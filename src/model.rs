use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceScope {
    /// User specific LaunchAgents (~/Library/LaunchAgents)
    User,
    /// Global system LaunchAgents (/Library/LaunchAgents)
    GlobalAgent,
    /// System Root LaunchDaemons (/Library/LaunchDaemons)
    SystemDaemon,
}

impl ServiceScope {
    pub fn display_name(&self) -> &'static str {
        match self {
            ServiceScope::User => "User Agent (~/Library/LaunchAgents)",
            ServiceScope::GlobalAgent => "Global Agent (/Library/LaunchAgents)",
            ServiceScope::SystemDaemon => "System Daemon (/Library/LaunchDaemons)",
        }
    }

    pub fn directory_path(&self) -> PathBuf {
        match self {
            ServiceScope::User => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
                PathBuf::from(home).join("Library/LaunchAgents")
            }
            ServiceScope::GlobalAgent => PathBuf::from("/Library/LaunchAgents"),
            ServiceScope::SystemDaemon => PathBuf::from("/Library/LaunchDaemons"),
        }
    }

    pub fn requires_root(&self) -> bool {
        matches!(self, ServiceScope::SystemDaemon | ServiceScope::GlobalAgent)
    }
}

impl std::fmt::Display for ServiceScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for ServiceScope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" | "u" => Ok(ServiceScope::User),
            "global" | "global-agent" | "global_agent" | "g" => Ok(ServiceScope::GlobalAgent),
            "system" | "system-daemon" | "system_daemon" | "sys" | "s" | "root" => Ok(ServiceScope::SystemDaemon),
            _ => Err(anyhow::anyhow!("Unknown scope: '{}'. Valid scopes: user, global, system", s)),
        }
    }
}

/// Represents the contents of a macOS Launchd Plist file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LaunchdPlist {
    pub label: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_arguments: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_at_load: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_out_path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_error_path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_variables: Option<BTreeMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_interval: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttle_interval: Option<u64>,
}

impl LaunchdPlist {
    pub fn new(label: impl Into<String>, program_arguments: Vec<String>) -> Self {
        Self {
            label: label.into(),
            program: None,
            program_arguments: Some(program_arguments),
            run_at_load: Some(true),
            keep_alive: Some(serde_json::Value::Bool(true)),
            standard_out_path: None,
            standard_error_path: None,
            working_directory: None,
            environment_variables: None,
            start_interval: None,
            process_type: None,
            disabled: None,
            throttle_interval: None,
        }
    }
}

/// Helper functions for reading/writing plist files and raw XML
pub fn read_plist_from_file(path: &Path) -> anyhow::Result<LaunchdPlist> {
    let value: LaunchdPlist = plist::from_file(path)?;
    Ok(value)
}

#[allow(dead_code)]
pub fn write_plist_to_file(path: &Path, plist_data: &LaunchdPlist) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    plist::to_file_xml(path, plist_data)?;
    Ok(())
}

pub fn read_raw_xml(path: &Path) -> anyhow::Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

pub fn write_raw_xml(path: &Path, xml_content: &str) -> anyhow::Result<()> {
    validate_raw_xml(xml_content)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, xml_content)?;
    Ok(())
}

pub fn validate_raw_xml(xml_content: &str) -> anyhow::Result<()> {
    // Attempt to parse XML via plist crate
    let cursor = std::io::Cursor::new(xml_content.as_bytes());
    let _value: plist::Value = plist::Value::from_reader(cursor)
        .map_err(|e| anyhow::anyhow!("Invalid Plist XML syntax: {}", e))?;
    Ok(())
}
