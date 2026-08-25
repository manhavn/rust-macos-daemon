use crate::model::{read_plist_from_file, LaunchdPlist, ServiceScope};
use crate::privilege::{current_user_id, run_command};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonItem {
    pub label: String,
    pub scope: ServiceScope,
    pub plist_path: PathBuf,
    pub pid: Option<i32>,
    pub last_exit_status: Option<i32>,
    pub is_loaded: bool,
    pub is_enabled: bool,
    pub plist_data: Option<LaunchdPlist>,
}

pub struct LaunchdManager;

impl LaunchdManager {
    /// Domain target string for modern launchctl (gui/<uid> or system)
    pub fn domain_target(scope: ServiceScope) -> String {
        match scope {
            ServiceScope::User => format!("gui/{}", current_user_id()),
            ServiceScope::GlobalAgent => format!("gui/{}", current_user_id()),
            ServiceScope::SystemDaemon => "system".to_string(),
        }
    }

    /// Query launchctl list for running services and PIDs
    pub fn get_launchctl_list() -> HashMap<String, (Option<i32>, Option<i32>)> {
        let mut map = HashMap::new();
        let output = match std::process::Command::new("launchctl").arg("list").output() {
            Ok(out) => out,
            Err(_) => return map,
        };

        if !output.status.success() {
            return map;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let pid = parts[0].parse::<i32>().ok();
                let status = parts[1].parse::<i32>().ok();
                let label = parts[2].to_string();
                map.insert(label, (pid, status));
            }
        }
        map
    }

    /// List all services across requested scopes
    pub fn list_services(filter_scope: Option<ServiceScope>) -> Result<Vec<DaemonItem>> {
        let launchctl_map = Self::get_launchctl_list();
        let scopes = match filter_scope {
            Some(s) => vec![s],
            None => vec![
                ServiceScope::User,
                ServiceScope::GlobalAgent,
                ServiceScope::SystemDaemon,
            ],
        };

        let mut items = Vec::new();

        for scope in scopes {
            let dir = scope.directory_path();
            if !dir.exists() {
                continue;
            }

            let entries = match std::fs::read_dir(&dir) {
                Ok(rd) => rd,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "plist") {
                    let plist_data = read_plist_from_file(&path).ok();
                    let label = plist_data
                        .as_ref()
                        .map(|p| p.label.clone())
                        .unwrap_or_else(|| {
                            path.file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned()
                        });

                    let (pid, last_exit_status) =
                        launchctl_map.get(&label).cloned().unwrap_or((None, None));

                    let is_loaded = launchctl_map.contains_key(&label);
                    let is_enabled = plist_data.as_ref().is_none_or(|p| p.disabled != Some(true));

                    items.push(DaemonItem {
                        label,
                        scope,
                        plist_path: path,
                        pid,
                        last_exit_status,
                        is_loaded,
                        is_enabled,
                        plist_data,
                    });
                }
            }
        }

        items.sort_by(|a, b| a.label.cmp(&b.label));
        Ok(items)
    }

    /// Find a service by label
    pub fn find_service(label: &str, scope: Option<ServiceScope>) -> Result<Option<DaemonItem>> {
        let all = Self::list_services(scope)?;
        Ok(all.into_iter().find(|d| d.label == label))
    }

    /// Load / bootstrap a service
    pub fn load_service(scope: ServiceScope, plist_path: &Path) -> Result<()> {
        let path_str = plist_path.to_string_lossy();
        let target = Self::domain_target(scope);
        let req_root = scope.requires_root();

        // Modern launchctl bootstrap
        let boot_out = run_command("launchctl", &["bootstrap", &target, &path_str], req_root);
        if boot_out.as_ref().is_ok_and(|o| o.status.success()) {
            return Ok(());
        }

        // Fallback to legacy launchctl load -w
        let load_out = run_command("launchctl", &["load", "-w", &path_str], req_root)?;
        if !load_out.status.success() {
            let err = String::from_utf8_lossy(&load_out.stderr);
            return Err(anyhow::anyhow!(
                "Failed to load service {}: {}",
                path_str,
                err
            ));
        }

        Ok(())
    }

    /// Unload / bootout a service
    pub fn unload_service(scope: ServiceScope, plist_path: &Path, label: &str) -> Result<()> {
        let path_str = plist_path.to_string_lossy();
        let target = Self::domain_target(scope);
        let service_target = format!("{}/{}", target, label);
        let req_root = scope.requires_root();

        // Modern launchctl bootout
        let boot_out = run_command("launchctl", &["bootout", &service_target], req_root);
        if boot_out.as_ref().is_ok_and(|o| o.status.success()) {
            return Ok(());
        }

        // Fallback to legacy launchctl unload -w
        let unload_out = run_command("launchctl", &["unload", "-w", &path_str], req_root)?;
        if !unload_out.status.success() {
            let err = String::from_utf8_lossy(&unload_out.stderr);
            return Err(anyhow::anyhow!(
                "Failed to unload service {}: {}",
                label,
                err
            ));
        }

        Ok(())
    }

    /// Enable a service
    pub fn enable_service(scope: ServiceScope, label: &str) -> Result<()> {
        let target = Self::domain_target(scope);
        let service_target = format!("{}/{}", target, label);
        let req_root = scope.requires_root();

        let out = run_command("launchctl", &["enable", &service_target], req_root)?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(anyhow::anyhow!(
                "Failed to enable service {}: {}",
                label,
                err
            ));
        }

        Ok(())
    }

    /// Disable a service
    pub fn disable_service(scope: ServiceScope, label: &str) -> Result<()> {
        let target = Self::domain_target(scope);
        let service_target = format!("{}/{}", target, label);
        let req_root = scope.requires_root();

        let out = run_command("launchctl", &["disable", &service_target], req_root)?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(anyhow::anyhow!(
                "Failed to disable service {}: {}",
                label,
                err
            ));
        }

        Ok(())
    }

    /// Kickstart / start a service
    pub fn start_service(scope: ServiceScope, label: &str) -> Result<()> {
        let target = Self::domain_target(scope);
        let service_target = format!("{}/{}", target, label);
        let req_root = scope.requires_root();

        // Modern kickstart -k
        let out = run_command("launchctl", &["kickstart", "-k", &service_target], req_root);
        if out.as_ref().is_ok_and(|o| o.status.success()) {
            return Ok(());
        }

        // Legacy launchctl start
        let legacy_out = run_command("launchctl", &["start", label], req_root)?;
        if !legacy_out.status.success() {
            let err = String::from_utf8_lossy(&legacy_out.stderr);
            return Err(anyhow::anyhow!(
                "Failed to start service {}: {}",
                label,
                err
            ));
        }

        Ok(())
    }

    /// Stop a service
    pub fn stop_service(scope: ServiceScope, label: &str) -> Result<()> {
        let req_root = scope.requires_root();
        let legacy_out = run_command("launchctl", &["stop", label], req_root)?;
        if !legacy_out.status.success() {
            let err = String::from_utf8_lossy(&legacy_out.stderr);
            return Err(anyhow::anyhow!("Failed to stop service {}: {}", label, err));
        }

        Ok(())
    }
}
