use anyhow::{anyhow, Result};
use std::process::{Command, Output};

pub fn is_root() -> bool {
    users::get_current_uid() == 0
}

pub fn current_user_id() -> u32 {
    users::get_current_uid()
}

pub fn current_user_name() -> String {
    users::get_current_username()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Execute command with elevation if required
pub fn run_command(command: &str, args: &[&str], require_root: bool) -> Result<Output> {
    if !require_root || is_root() {
        let output = Command::new(command).args(args).output()?;
        Ok(output)
    } else {
        // Try sudo first
        let mut sudo_args = vec![command];
        sudo_args.extend(args.iter().cloned());

        let output = Command::new("sudo").args(&sudo_args).output();

        match output {
            Ok(out) if out.status.success() => Ok(out),
            _ => {
                // Fallback to osascript for GUI / Web UI prompt if sudo failed due to TTY/password
                let full_cmd = format!("{} {}", command, args.join(" "));
                let escaped_cmd = full_cmd.replace('\\', "\\\\").replace('"', "\\\"");
                let osa_script = format!(
                    "do shell script \"{}\" with administrator privileges",
                    escaped_cmd
                );

                let osa_output = Command::new("osascript")
                    .arg("-e")
                    .arg(&osa_script)
                    .output()?;

                if !osa_output.status.success() {
                    let stderr = String::from_utf8_lossy(&osa_output.stderr);
                    return Err(anyhow!("Privilege escalation failed: {}", stderr));
                }

                Ok(osa_output)
            }
        }
    }
}

/// Write file with privilege escalation if necessary
pub fn write_file_elevated(
    path: &std::path::Path,
    content: &str,
    require_root: bool,
) -> Result<()> {
    if !require_root || is_root() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    } else {
        // Write to temporary file first, then sudo mv to destination
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("macdaemon_tmp_{}.xml", std::process::id()));
        std::fs::write(&temp_path, content)?;

        let dest_str = path.to_string_lossy();
        let temp_str = temp_path.to_string_lossy();

        let parent_str = path
            .parent()
            .map(|p| p.to_string_lossy())
            .unwrap_or_default();
        let mkdir_out = run_command("mkdir", &["-p", &parent_str], true)?;
        if !mkdir_out.status.success() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(anyhow!(
                "Failed to create directory {}: {}",
                parent_str,
                String::from_utf8_lossy(&mkdir_out.stderr)
            ));
        }

        let mv_out = run_command("mv", &[&temp_str, &dest_str], true)?;
        let _ = std::fs::remove_file(&temp_path);

        if !mv_out.status.success() {
            return Err(anyhow!(
                "Failed to move file to {}: {}",
                dest_str,
                String::from_utf8_lossy(&mv_out.stderr)
            ));
        }

        // Set proper permissions for system daemons (root:wheel 644)
        let chmod_out = run_command("chmod", &["644", &dest_str], true)?;
        if !chmod_out.status.success() {
            tracing::warn!("Failed to chmod 644 on {}", dest_str);
        }

        Ok(())
    }
}

/// Delete file with privilege escalation if necessary
pub fn remove_file_elevated(path: &std::path::Path, require_root: bool) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if !require_root || is_root() {
        std::fs::remove_file(path)?;
        Ok(())
    } else {
        let dest_str = path.to_string_lossy();
        let rm_out = run_command("rm", &["-f", &dest_str], true)?;
        if !rm_out.status.success() {
            return Err(anyhow!(
                "Failed to remove file {}: {}",
                dest_str,
                String::from_utf8_lossy(&rm_out.stderr)
            ));
        }
        Ok(())
    }
}

/// Send macOS native system desktop notification
pub fn send_macos_notification(title: &str, message: &str) {
    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        message.replace('"', "\\\""),
        title.replace('"', "\\\"")
    );
    let _ = Command::new("osascript").arg("-e").arg(script).output();
}
