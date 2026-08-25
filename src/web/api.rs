use crate::launchd::{DaemonItem, LaunchdManager};
use crate::model::{read_raw_xml, validate_raw_xml, LaunchdPlist, ServiceScope};
use crate::privilege::{current_user_name, is_root, remove_file_elevated, write_file_elevated};
use axum::{
    extract::{Path as AxumPath, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct SystemStatusResponse {
    pub is_root: bool,
    pub user_name: String,
    pub user_id: u32,
    pub os: String,
}

pub async fn get_system_status() -> impl IntoResponse {
    let status = SystemStatusResponse {
        is_root: is_root(),
        user_name: current_user_name(),
        user_id: crate::privilege::current_user_id(),
        os: "macOS".to_string(),
    };
    Json(status)
}

#[derive(Deserialize)]
pub struct ListServicesQuery {
    pub scope: Option<String>,
}

pub async fn list_services(Query(query): Query<ListServicesQuery>) -> impl IntoResponse {
    let scope_filter = query
        .scope
        .as_deref()
        .map(|s| s.parse::<ServiceScope>())
        .transpose();

    match scope_filter {
        Ok(filter) => match LaunchdManager::list_services(filter) {
            Ok(items) => (
                StatusCode::OK,
                Json(serde_json::json!({ "services": items })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            ),
        },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
pub struct ServiceDetailQuery {
    pub scope: Option<String>,
}

#[derive(Serialize)]
pub struct ServiceDetailResponse {
    pub item: DaemonItem,
    pub raw_xml: String,
}

pub async fn get_service_detail(
    AxumPath(label): AxumPath<String>,
    Query(query): Query<ServiceDetailQuery>,
) -> impl IntoResponse {
    let scope_filter = query
        .scope
        .as_deref()
        .map(|s| s.parse::<ServiceScope>())
        .transpose();

    let scope = match scope_filter {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    match LaunchdManager::find_service(&label, scope) {
        Ok(Some(item)) => {
            let raw_xml = read_raw_xml(&item.plist_path).unwrap_or_default();
            (
                StatusCode::OK,
                Json(serde_json::json!(ServiceDetailResponse { item, raw_xml })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Service '{}' not found", label) })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
pub struct CreateServiceRequest {
    pub label: String,
    pub scope: String,
    pub exec: String,
    pub args: Option<Vec<String>>,
    pub run_at_load: Option<bool>,
    pub keep_alive: Option<bool>,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub workdir: Option<String>,
    pub interval: Option<u64>,
    pub env: Option<HashMap<String, String>>,
}

pub async fn create_or_update_service(Json(req): Json<CreateServiceRequest>) -> impl IntoResponse {
    let scope: ServiceScope = match req.scope.parse() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    let target_dir = scope.directory_path();
    let plist_path = target_dir.join(format!("{}.plist", req.label));

    let mut cmd_args = vec![req.exec];
    if let Some(extra) = req.args {
        cmd_args.extend(extra);
    }

    let mut plist_obj = LaunchdPlist::new(req.label.clone(), cmd_args);
    plist_obj.run_at_load = req.run_at_load;
    if let Some(ka) = req.keep_alive {
        plist_obj.keep_alive = Some(serde_json::Value::Bool(ka));
    }
    plist_obj.standard_out_path = req.stdout_path;
    plist_obj.standard_error_path = req.stderr_path;
    plist_obj.working_directory = req.workdir;
    plist_obj.start_interval = req.interval;
    if let Some(env_map) = req.env {
        plist_obj.environment_variables = Some(env_map.into_iter().collect());
    }

    let mut xml_bytes = Vec::new();
    if let Err(e) = plist::to_writer_xml(&mut xml_bytes, &plist_obj) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Failed to generate plist XML: {}", e) })),
        );
    }

    let xml_str = String::from_utf8_lossy(&xml_bytes);

    if let Err(e) = write_file_elevated(&plist_path, &xml_str, scope.requires_root()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to write plist file: {}", e) })),
        );
    }

    // Attempt to reload service
    let _ = LaunchdManager::unload_service(scope, &plist_path, &req.label);
    let load_res = LaunchdManager::load_service(scope, &plist_path);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": format!("Service '{}' saved successfully", req.label),
            "load_status": load_res.is_ok(),
            "load_error": load_res.err().map(|e| e.to_string())
        })),
    )
}

#[derive(Deserialize)]
pub struct SaveRawServiceRequest {
    pub label: String,
    pub scope: String,
    pub xml_content: String,
}

pub async fn save_raw_service(Json(req): Json<SaveRawServiceRequest>) -> impl IntoResponse {
    let scope: ServiceScope = match req.scope.parse() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    if let Err(e) = validate_raw_xml(&req.xml_content) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("XML Validation Error: {}", e) })),
        );
    }

    let target_dir = scope.directory_path();
    let plist_path = target_dir.join(format!("{}.plist", req.label));

    if let Err(e) = write_file_elevated(&plist_path, &req.xml_content, scope.requires_root()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to write plist file: {}", e) })),
        );
    }

    let _ = LaunchdManager::unload_service(scope, &plist_path, &req.label);
    let load_res = LaunchdManager::load_service(scope, &plist_path);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": format!("Raw Plist XML for '{}' saved successfully", req.label),
            "load_status": load_res.is_ok(),
            "load_error": load_res.err().map(|e| e.to_string())
        })),
    )
}

#[derive(Deserialize)]
pub struct DeleteServiceQuery {
    pub scope: String,
}

pub async fn delete_service(
    AxumPath(label): AxumPath<String>,
    Query(query): Query<DeleteServiceQuery>,
) -> impl IntoResponse {
    let scope: ServiceScope = match query.scope.parse() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    let plist_path = scope.directory_path().join(format!("{}.plist", label));

    let _ = LaunchdManager::unload_service(scope, &plist_path, &label);

    if let Err(e) = remove_file_elevated(&plist_path, scope.requires_root()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to remove file: {}", e) })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "message": format!("Service '{}' deleted successfully", label) })),
    )
}

#[derive(Deserialize)]
pub struct ActionRequest {
    pub action: String,
    pub scope: String,
}

pub async fn service_action(
    AxumPath(label): AxumPath<String>,
    Json(req): Json<ActionRequest>,
) -> impl IntoResponse {
    let scope: ServiceScope = match req.scope.parse() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    let plist_path = scope.directory_path().join(format!("{}.plist", label));

    let res = match req.action.to_lowercase().as_str() {
        "load" => LaunchdManager::load_service(scope, &plist_path),
        "unload" => LaunchdManager::unload_service(scope, &plist_path, &label),
        "enable" => LaunchdManager::enable_service(scope, &label),
        "disable" => LaunchdManager::disable_service(scope, &label),
        "start" => LaunchdManager::start_service(scope, &label),
        "stop" => LaunchdManager::stop_service(scope, &label),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unknown action '{}'", req.action) })),
            )
        }
    };

    match res {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": format!("Action '{}' on service '{}' succeeded", req.action, label)
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
pub struct LogQuery {
    pub path: String,
    pub lines: Option<usize>,
}

pub async fn get_service_log(Query(query): Query<LogQuery>) -> impl IntoResponse {
    let path = PathBuf::from(&query.path);
    if !path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Log file not found: {}", query.path) })),
        );
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let max_lines = query.lines.unwrap_or(200);
            let line_vec: Vec<&str> = content.lines().collect();
            let start = if line_vec.len() > max_lines {
                line_vec.len() - max_lines
            } else {
                0
            };
            let tail_content = line_vec[start..].join("\n");

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "path": query.path,
                    "total_lines": line_vec.len(),
                    "content": tail_content
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to read log file: {}", e) })),
        ),
    }
}

#[derive(Deserialize)]
pub struct ReadFileQuery {
    pub path: String,
}

pub async fn read_raw_file(Query(query): Query<ReadFileQuery>) -> impl IntoResponse {
    let path = PathBuf::from(&query.path);
    let exists = path.exists();

    if !exists {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "path": query.path,
                "exists": false,
                "content": "",
                "requires_root": query.path.starts_with("/Library") || query.path.starts_with("/etc") || query.path.starts_with("/System") || query.path.starts_with("/var")
            })),
        );
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "path": query.path,
                "exists": true,
                "content": content,
                "requires_root": query.path.starts_with("/Library") || query.path.starts_with("/etc") || query.path.starts_with("/System") || query.path.starts_with("/var")
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": format!("Failed to read file '{}': {}", query.path, e) }),
            ),
        ),
    }
}

#[derive(Deserialize)]
pub struct SaveFileRequest {
    pub path: String,
    pub content: String,
    pub require_root: Option<bool>,
}

pub async fn save_raw_file(Json(req): Json<SaveFileRequest>) -> impl IntoResponse {
    let path = PathBuf::from(&req.path);
    let require_root = req.require_root.unwrap_or_else(|| {
        req.path.starts_with("/Library")
            || req.path.starts_with("/etc")
            || req.path.starts_with("/System")
            || req.path.starts_with("/var")
    });

    match write_file_elevated(&path, &req.content, require_root) {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": format!("File '{}' saved successfully", req.path),
                "path": req.path
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to save file: {}", e) })),
        ),
    }
}

#[derive(Deserialize)]
pub struct PermissionRequest {
    pub path: String,
    pub action: String,
    pub value: Option<String>,
    pub require_root: Option<bool>,
}

pub async fn manage_permissions(Json(req): Json<PermissionRequest>) -> impl IntoResponse {
    let path = PathBuf::from(&req.path);
    let require_root = req.require_root.unwrap_or_else(|| {
        req.path.starts_with("/Library")
            || req.path.starts_with("/etc")
            || req.path.starts_with("/System")
            || req.path.starts_with("/var")
    });

    match req.action.to_lowercase().as_str() {
        "load" => {
            if !path.exists() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(
                        serde_json::json!({ "error": format!("Path '{}' does not exist", req.path) }),
                    ),
                );
            }

            let out = match std::process::Command::new("ls")
                .args(["-ld", &req.path])
                .output()
            {
                Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
                Err(e) => format!("Error: {}", e),
            };

            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let chmod_mode = match std::fs::metadata(&req.path) {
                Ok(m) => format!("{:o}", m.permissions().mode() & 0o777),
                Err(_) => "".to_string(),
            };

            let chown_val = match std::fs::metadata(&req.path) {
                Ok(m) => {
                    let uid = m.uid();
                    let gid = m.gid();
                    let u = users::get_user_by_uid(uid)
                        .map(|u| u.name().to_string_lossy().into_owned())
                        .unwrap_or_else(|| uid.to_string());
                    let g = users::get_group_by_gid(gid)
                        .map(|g| g.name().to_string_lossy().into_owned())
                        .unwrap_or_else(|| gid.to_string());
                    format!("{}:{}", u, g)
                }
                Err(_) => "".to_string(),
            };

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "path": req.path,
                    "info": out.trim(),
                    "chmod": chmod_mode,
                    "chown": chown_val,
                    "requires_root": require_root
                })),
            )
        }
        "chmod" => {
            let mode = match req.value {
                Some(ref m) if !m.trim().is_empty() => m.trim().to_string(),
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(
                            serde_json::json!({ "error": "Chmod permissions value required (e.g. 755)" }),
                        ),
                    )
                }
            };

            match crate::privilege::run_command("chmod", &[&mode, &req.path], require_root) {
                Ok(out) if out.status.success() => (
                    StatusCode::OK,
                    Json(
                        serde_json::json!({ "message": format!("Chmod {} applied to '{}'", mode, req.path) }),
                    ),
                ),
                Ok(out) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        serde_json::json!({ "error": format!("Chmod failed: {}", String::from_utf8_lossy(&out.stderr)) }),
                    ),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                ),
            }
        }
        "chown" => {
            let owner = match req.value {
                Some(ref o) if !o.trim().is_empty() => o.trim().to_string(),
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(
                            serde_json::json!({ "error": "Chown owner:group value required (e.g. root:wheel)" }),
                        ),
                    )
                }
            };

            match crate::privilege::run_command("chown", &[&owner, &req.path], require_root) {
                Ok(out) if out.status.success() => (
                    StatusCode::OK,
                    Json(
                        serde_json::json!({ "message": format!("Chown {} applied to '{}'", owner, req.path) }),
                    ),
                ),
                Ok(out) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        serde_json::json!({ "error": format!("Chown failed: {}", String::from_utf8_lossy(&out.stderr)) }),
                    ),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                ),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({ "error": format!("Unknown permission action '{}'", req.action) }),
            ),
        ),
    }
}

#[derive(Deserialize)]
pub struct CopyRequest {
    pub src: String,
    pub dest: String,
    pub require_root: Option<bool>,
}

pub async fn copy_path(Json(req): Json<CopyRequest>) -> impl IntoResponse {
    let src_path = PathBuf::from(&req.src);
    if !src_path.exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({ "error": format!("Source path '{}' does not exist", req.src) }),
            ),
        );
    }

    let require_root = req.require_root.unwrap_or_else(|| {
        req.src.starts_with("/Library")
            || req.dest.starts_with("/Library")
            || req.src.starts_with("/etc")
            || req.dest.starts_with("/etc")
            || req.src.starts_with("/System")
            || req.dest.starts_with("/System")
            || req.src.starts_with("/var")
            || req.dest.starts_with("/var")
    });

    match crate::privilege::run_command("cp", &["-R", &req.src, &req.dest], require_root) {
        Ok(out) if out.status.success() => (
            StatusCode::OK,
            Json(
                serde_json::json!({ "message": format!("Successfully copied '{}' to '{}'", req.src, req.dest) }),
            ),
        ),
        Ok(out) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": format!("Copy failed: {}", String::from_utf8_lossy(&out.stderr)) }),
            ),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
pub struct DeletePathRequest {
    pub path: String,
    pub require_root: Option<bool>,
}

pub async fn delete_path(Json(req): Json<DeletePathRequest>) -> impl IntoResponse {
    let path = PathBuf::from(&req.path);
    if !path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Path '{}' does not exist", req.path) })),
        );
    }

    let require_root = req.require_root.unwrap_or_else(|| {
        req.path.starts_with("/Library")
            || req.path.starts_with("/etc")
            || req.path.starts_with("/System")
            || req.path.starts_with("/var")
    });

    match crate::privilege::run_command("rm", &["-rf", &req.path], require_root) {
        Ok(out) if out.status.success() => (
            StatusCode::OK,
            Json(serde_json::json!({ "message": format!("Successfully deleted '{}'", req.path) })),
        ),
        Ok(out) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": format!("Delete failed: {}", String::from_utf8_lossy(&out.stderr)) }),
            ),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub src: String,
    pub dest: String,
    pub require_root: Option<bool>,
}

pub async fn move_path(Json(req): Json<MoveRequest>) -> impl IntoResponse {
    let src_path = PathBuf::from(&req.src);
    if !src_path.exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({ "error": format!("Source path '{}' does not exist", req.src) }),
            ),
        );
    }

    let require_root = req.require_root.unwrap_or_else(|| {
        req.src.starts_with("/Library")
            || req.dest.starts_with("/Library")
            || req.src.starts_with("/etc")
            || req.dest.starts_with("/etc")
            || req.src.starts_with("/System")
            || req.dest.starts_with("/System")
            || req.src.starts_with("/var")
            || req.dest.starts_with("/var")
    });

    match crate::privilege::run_command("mv", &[&req.src, &req.dest], require_root) {
        Ok(out) if out.status.success() => (
            StatusCode::OK,
            Json(
                serde_json::json!({ "message": format!("Successfully moved '{}' to '{}'", req.src, req.dest) }),
            ),
        ),
        Ok(out) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": format!("Move failed: {}", String::from_utf8_lossy(&out.stderr)) }),
            ),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
