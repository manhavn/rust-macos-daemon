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
            Ok(items) => (StatusCode::OK, Json(serde_json::json!({ "services": items }))),
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
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))),
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

pub async fn create_or_update_service(
    Json(req): Json<CreateServiceRequest>,
) -> impl IntoResponse {
    let scope: ServiceScope = match req.scope.parse() {
        Ok(s) => s,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))),
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
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))),
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
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))),
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
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))),
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
