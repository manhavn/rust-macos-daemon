use crate::launchd::LaunchdManager;
use crate::model::{read_raw_xml, write_raw_xml, LaunchdPlist, ServiceScope};
use crate::privilege::{is_root, remove_file_elevated, write_file_elevated};
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser, Debug)]
#[command(name = "macdaemon")]
#[command(about = "macOS Launchd Daemon & Agent Manager with Sudo, CLI & Web UI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all registered LaunchAgents and LaunchDaemons
    List {
        /// Scope filter: user, global, system, or all
        #[arg(short, long)]
        scope: Option<String>,

        /// Output format as JSON
        #[arg(long)]
        json: bool,
    },

    /// Display detailed information and plist content of a service
    Info {
        /// Service label (e.g. com.example.myservice)
        label: String,

        /// Scope: user, global, system
        #[arg(short, long)]
        scope: Option<String>,
    },

    /// Register a new daemon or agent auto-start service
    Add {
        /// Unique service label (e.g. com.user.mydaemon)
        #[arg(short, long)]
        label: String,

        /// Program executable or shell command
        #[arg(short, long)]
        exec: String,

        /// Additional arguments (comma separated or multiple flags)
        #[arg(short, long, value_delimiter = ',')]
        args: Option<Vec<String>>,

        /// Scope: user (default), global, or system
        #[arg(short, long, default_value = "user")]
        scope: String,

        /// Run automatically when loaded / system boots
        #[arg(long, default_value = "true")]
        run_at_load: bool,

        /// Keep process alive automatically if it crashes/exits
        #[arg(long, default_value = "true")]
        keep_alive: bool,

        /// Standard output log file path
        #[arg(long)]
        stdout: Option<String>,

        /// Standard error log file path
        #[arg(long)]
        stderr: Option<String>,

        /// Working directory
        #[arg(long)]
        workdir: Option<String>,

        /// Start interval in seconds
        #[arg(long)]
        interval: Option<u64>,
    },

    /// Unload and remove a registered daemon/agent plist file
    Remove {
        /// Service label
        label: String,

        /// Scope: user, global, system
        #[arg(short, long, default_value = "user")]
        scope: String,
    },

    /// Load (bootstrap) a service via launchctl
    Load {
        /// Service label
        label: String,

        /// Scope: user, global, system
        #[arg(short, long, default_value = "user")]
        scope: String,
    },

    /// Unload (bootout) a service via launchctl
    Unload {
        /// Service label
        label: String,

        /// Scope: user, global, system
        #[arg(short, long, default_value = "user")]
        scope: String,
    },

    /// Enable a service in launchctl
    Enable {
        /// Service label
        label: String,

        /// Scope: user, global, system
        #[arg(short, long, default_value = "user")]
        scope: String,
    },

    /// Disable a service in launchctl
    Disable {
        /// Service label
        label: String,

        /// Scope: user, global, system
        #[arg(short, long, default_value = "user")]
        scope: String,
    },

    /// Start (kickstart) a service immediately
    Start {
        /// Service label
        label: String,

        /// Scope: user, global, system
        #[arg(short, long, default_value = "user")]
        scope: String,
    },

    /// Stop a running service
    Stop {
        /// Service label
        label: String,

        /// Scope: user, global, system
        #[arg(short, long, default_value = "user")]
        scope: String,
    },

    /// Edit or view raw XML plist file
    RawEdit {
        /// Service label
        label: String,

        /// Scope: user, global, system
        #[arg(short, long, default_value = "user")]
        scope: String,
    },

    /// Start the Web UI remote management server
    Web {
        /// Host address to bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value_t = 8990)]
        port: u16,

        /// Open web app in default browser
        #[arg(long, default_value_t = true)]
        open: bool,
    },
}

#[derive(Debug)]
pub enum CliAction {
    Executed,
    StartWeb {
        host: String,
        port: u16,
        open: bool,
    },
}

pub fn handle_cli(cli: Cli) -> Result<CliAction> {
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            return Ok(CliAction::StartWeb {
                host: "127.0.0.1".to_string(),
                port: 8990,
                open: true,
            })
        }
    };

    match command {
        Commands::List { scope, json } => {
            let filter_scope = scope.as_deref().map(|s| s.parse::<ServiceScope>()).transpose()?;
            let items = LaunchdManager::list_services(filter_scope)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&items)?);
            } else {
                println!("{}", "=== macOS Launchd Services ===".bold().cyan());
                if is_root() {
                    println!("{}", "Running as ROOT (Full System Access)".bold().green());
                } else {
                    println!("{}", "Running as User (System Daemons may require sudo)".yellow());
                }
                println!();

                for item in &items {
                    let status_str = if let Some(pid) = item.pid {
                        format!("RUNNING (PID: {})", pid).bold().green()
                    } else if item.is_loaded {
                        "LOADED (Stopped)".bold().yellow()
                    } else {
                        "UNLOADED".dimmed()
                    };

                    let scope_badge = match item.scope {
                        ServiceScope::User => "[User]".green(),
                        ServiceScope::GlobalAgent => "[Global]".blue(),
                        ServiceScope::SystemDaemon => "[System Daemon]".magenta(),
                    };

                    println!("{} {} {}", scope_badge, item.label.bold(), status_str);
                    println!("   Path: {}", item.plist_path.display());
                    if let Some(ref plist) = item.plist_data {
                        if let Some(ref args) = plist.program_arguments {
                            println!("   Cmd:  {}", args.join(" ").dimmed());
                        } else if let Some(ref prog) = plist.program {
                            println!("   Prog: {}", prog.dimmed());
                        }
                    }
                    println!();
                }
                println!("Total: {} service(s)", items.len());
            }
        }

        Commands::Info { label, scope } => {
            let filter_scope = scope.as_deref().map(|s| s.parse::<ServiceScope>()).transpose()?;
            let item = LaunchdManager::find_service(&label, filter_scope)?
                .ok_or_else(|| anyhow!("Service '{}' not found", label))?;

            println!("{}", format!("=== Service Info: {} ===", item.label).bold().cyan());
            println!("Scope:         {:?}", item.scope);
            println!("Plist Path:    {}", item.plist_path.display());
            println!("Is Loaded:     {}", item.is_loaded);
            println!("PID:           {:?}", item.pid);
            println!("Last Exit Code: {:?}", item.last_exit_status);
            println!();

            println!("{}", "--- Raw Plist Content ---".bold().yellow());
            let raw_xml = read_raw_xml(&item.plist_path)?;
            println!("{}", raw_xml);
        }

        Commands::Add {
            label,
            exec,
            args,
            scope,
            run_at_load,
            keep_alive,
            stdout,
            stderr,
            workdir,
            interval,
        } => {
            let parsed_scope: ServiceScope = scope.parse()?;
            let target_dir = parsed_scope.directory_path();
            let plist_path = target_dir.join(format!("{}.plist", label));

            let mut cmd_args = vec![exec.clone()];
            if let Some(extra_args) = args {
                cmd_args.extend(extra_args);
            }

            let mut plist_obj = LaunchdPlist::new(label.clone(), cmd_args);
            plist_obj.run_at_load = Some(run_at_load);
            plist_obj.keep_alive = Some(serde_json::Value::Bool(keep_alive));
            plist_obj.standard_out_path = stdout;
            plist_obj.standard_error_path = stderr;
            plist_obj.working_directory = workdir;
            plist_obj.start_interval = interval;

            let mut xml_bytes = Vec::new();
            plist::to_writer_xml(&mut xml_bytes, &plist_obj)?;
            let xml_str = String::from_utf8(xml_bytes)?;

            println!("Registering service '{}' in {}...", label.bold(), parsed_scope);
            write_file_elevated(&plist_path, &xml_str, parsed_scope.requires_root())?;
            println!("{} Created plist at {}", "SUCCESS".green().bold(), plist_path.display());

            println!("Loading service via launchctl...");
            match LaunchdManager::load_service(parsed_scope, &plist_path) {
                Ok(_) => println!("{} Service loaded and running!", "SUCCESS".green().bold()),
                Err(e) => println!("{} Service created but load warning: {}", "WARNING".yellow(), e),
            }
        }

        Commands::Remove { label, scope } => {
            let parsed_scope: ServiceScope = scope.parse()?;
            let plist_path = parsed_scope.directory_path().join(format!("{}.plist", label));

            println!("Unloading service '{}'...", label.bold());
            let _ = LaunchdManager::unload_service(parsed_scope, &plist_path, &label);

            println!("Removing plist file...");
            remove_file_elevated(&plist_path, parsed_scope.requires_root())?;
            println!("{} Removed service '{}'", "SUCCESS".green().bold(), label);
        }

        Commands::Load { label, scope } => {
            let parsed_scope: ServiceScope = scope.parse()?;
            let item = LaunchdManager::find_service(&label, Some(parsed_scope))?
                .ok_or_else(|| anyhow!("Service '{}' not found", label))?;

            LaunchdManager::load_service(parsed_scope, &item.plist_path)?;
            println!("{} Service '{}' loaded successfully", "SUCCESS".green().bold(), label);
        }

        Commands::Unload { label, scope } => {
            let parsed_scope: ServiceScope = scope.parse()?;
            let item = LaunchdManager::find_service(&label, Some(parsed_scope))?
                .ok_or_else(|| anyhow!("Service '{}' not found", label))?;

            LaunchdManager::unload_service(parsed_scope, &item.plist_path, &label)?;
            println!("{} Service '{}' unloaded successfully", "SUCCESS".green().bold(), label);
        }

        Commands::Enable { label, scope } => {
            let parsed_scope: ServiceScope = scope.parse()?;
            LaunchdManager::enable_service(parsed_scope, &label)?;
            println!("{} Service '{}' enabled", "SUCCESS".green().bold(), label);
        }

        Commands::Disable { label, scope } => {
            let parsed_scope: ServiceScope = scope.parse()?;
            LaunchdManager::disable_service(parsed_scope, &label)?;
            println!("{} Service '{}' disabled", "SUCCESS".green().bold(), label);
        }

        Commands::Start { label, scope } => {
            let parsed_scope: ServiceScope = scope.parse()?;
            LaunchdManager::start_service(parsed_scope, &label)?;
            println!("{} Service '{}' started", "SUCCESS".green().bold(), label);
        }

        Commands::Stop { label, scope } => {
            let parsed_scope: ServiceScope = scope.parse()?;
            LaunchdManager::stop_service(parsed_scope, &label)?;
            println!("{} Service '{}' stopped", "SUCCESS".green().bold(), label);
        }

        Commands::RawEdit { label, scope } => {
            let parsed_scope: ServiceScope = scope.parse()?;
            let item = LaunchdManager::find_service(&label, Some(parsed_scope))?
                .ok_or_else(|| anyhow!("Service '{}' not found", label))?;

            let xml_content = read_raw_xml(&item.plist_path)?;
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

            let temp_file = std::env::temp_dir().join(format!("{}.plist", label));
            std::fs::write(&temp_file, &xml_content)?;

            let status = std::process::Command::new(&editor)
                .arg(&temp_file)
                .status()?;

            if status.success() {
                let edited_xml = std::fs::read_to_string(&temp_file)?;
                write_raw_xml(&temp_file, &edited_xml)?; // Validate syntax
                write_file_elevated(&item.plist_path, &edited_xml, parsed_scope.requires_root())?;
                println!("{} Updated raw plist for '{}'", "SUCCESS".green().bold(), label);
            }
            let _ = std::fs::remove_file(&temp_file);
        }

        Commands::Web { host, port, open } => {
            return Ok(CliAction::StartWeb { host, port, open });
        }
    }

    Ok(CliAction::Executed)
}
