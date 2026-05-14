use supertool_core::logic::CoreService;
#[allow(unused_imports)]
use supertool_core::logic::nginx_generator::NginxSubFile;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_nginx_presets(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_nginx_presets() called");
    let result = core.get_all_nginx_presets().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_preset(
    core: State<'_, CoreService>,
    preset: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_preset() called");
    let p: supertool_core::db::nginx::NginxPreset =
        serde_json::from_value(preset).map_err(|e| e.to_string())?;
    let result = core.add_nginx_preset(p).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_preset(
    core: State<'_, CoreService>,
    preset: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_preset() called");
    let p: supertool_core::db::nginx::NginxPreset =
        serde_json::from_value(preset).map_err(|e| e.to_string())?;
    let result = core.update_nginx_preset(p).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_preset(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_preset() called");
    let result = core.delete_nginx_preset(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn fetch_nginx_config(
    core: State<'_, CoreService>,
    server_id: String,
    config_path: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] fetch_nginx_config() called");
    let result = core.fetch_nginx_config(&server_id, &config_path).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn test_nginx_config(
    core: State<'_, CoreService>,
    server_id: String,
    config_path: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] test_nginx_config() called");
    let result = core.test_nginx_config(&server_id, &config_path).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn deploy_nginx_config(
    core: State<'_, CoreService>,
    server_id: String,
    config_path: String,
    content: String,
    _comment: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] deploy_nginx_config() called");
    let result = core
        .deploy_nginx_config(&server_id, &config_path, &content)
        .await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn deploy_nginx_config_decomposed(
    core: State<'_, CoreService>,
    server_id: String,
    config_path: String,
    main_content: String,
    sub_files: Vec<NginxSubFile>,
    _comment: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] deploy_nginx_config_decomposed() called");
    let result = core
        .deploy_nginx_config_decomposed(&server_id, &config_path, &main_content, sub_files)
        .await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_nginx_config_versions(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_nginx_config_versions() called");
    let result = core.get_nginx_config_versions(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_nginx_config_version(
    core: State<'_, CoreService>,
    version: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] save_nginx_config_version() called");
    let v: supertool_core::db::nginx::NginxConfigVersion =
        serde_json::from_value(version).map_err(|e| e.to_string())?;
    let result = core.add_nginx_config_version(v).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_active_nginx_version(
    core: State<'_, CoreService>,
    preset_id: String,
    version_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] set_active_nginx_version() called");
    let result = core.set_current_nginx_version(&preset_id, &version_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxServer Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_servers_by_preset(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_servers_by_preset() called");
    let result = core.get_servers_by_preset(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_server(
    core: State<'_, CoreService>,
    server: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_server() called");
    let s: supertool_core::db::nginx::NginxServer =
        serde_json::from_value(server).map_err(|e| e.to_string())?;
    let result = core.add_nginx_server(&s).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_server(
    core: State<'_, CoreService>,
    server: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_server() called");
    let s: supertool_core::db::nginx::NginxServer =
        serde_json::from_value(server).map_err(|e| e.to_string())?;
    let result = core.update_nginx_server(&s).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_server(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_server() called");
    let result = core.delete_nginx_server(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxLocation Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_locations_by_server(
    core: State<'_, CoreService>,
    server_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_locations_by_server() called");
    let result = core.get_locations_by_server(&server_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_location(
    core: State<'_, CoreService>,
    location: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_location() called");
    let loc: supertool_core::db::nginx::NginxLocation =
        serde_json::from_value(location).map_err(|e| e.to_string())?;
    let result = core.add_nginx_location(&loc).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_location(
    core: State<'_, CoreService>,
    location: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_location() called");
    let loc: supertool_core::db::nginx::NginxLocation =
        serde_json::from_value(location).map_err(|e| e.to_string())?;
    let result = core.update_nginx_location(&loc).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_location(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_location() called");
    let result = core.delete_nginx_location(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxUpstream Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_upstreams_by_preset(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_upstreams_by_preset() called");
    let result = core.get_upstreams_by_preset(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_upstream(
    core: State<'_, CoreService>,
    upstream: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_upstream() called");
    let u: supertool_core::db::nginx::NginxUpstream =
        serde_json::from_value(upstream).map_err(|e| e.to_string())?;
    let result = core.add_nginx_upstream(&u).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_upstream(
    core: State<'_, CoreService>,
    upstream: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_upstream() called");
    let u: supertool_core::db::nginx::NginxUpstream =
        serde_json::from_value(upstream).map_err(|e| e.to_string())?;
    let result = core.update_nginx_upstream(&u).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_upstream(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_upstream() called");
    let result = core.delete_nginx_upstream(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxUpstreamServer Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_upstream_servers(
    core: State<'_, CoreService>,
    upstream_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_upstream_servers() called");
    let result = core.get_upstream_servers(&upstream_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_upstream_server(
    core: State<'_, CoreService>,
    upstream_server: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_upstream_server() called");
    let us: supertool_core::db::nginx::NginxUpstreamServer =
        serde_json::from_value(upstream_server).map_err(|e| e.to_string())?;
    let result = core.add_nginx_upstream_server(&us).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_upstream_server(
    core: State<'_, CoreService>,
    upstream_server: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_upstream_server() called");
    let us: supertool_core::db::nginx::NginxUpstreamServer =
        serde_json::from_value(upstream_server).map_err(|e| e.to_string())?;
    let result = core.update_nginx_upstream_server(&us).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_upstream_server(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_upstream_server() called");
    let result = core.delete_nginx_upstream_server(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxHttpParam Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_http_params_by_preset(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_http_params_by_preset() called");
    let result = core.get_http_params_by_preset(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_http_param(
    core: State<'_, CoreService>,
    param: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_http_param() called");
    let p: supertool_core::db::nginx::NginxHttpParam =
        serde_json::from_value(param).map_err(|e| e.to_string())?;
    let result = core.add_nginx_http_param(&p).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_http_param(
    core: State<'_, CoreService>,
    param: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_http_param() called");
    let p: supertool_core::db::nginx::NginxHttpParam =
        serde_json::from_value(param).map_err(|e| e.to_string())?;
    let result = core.update_nginx_http_param(&p).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_http_param(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_http_param() called");
    let result = core.delete_nginx_http_param(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxStream Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_streams_by_preset(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_streams_by_preset() called");
    let result = core.get_streams_by_preset(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_stream(
    core: State<'_, CoreService>,
    stream: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_stream() called");
    let s: supertool_core::db::nginx::NginxStream =
        serde_json::from_value(stream).map_err(|e| e.to_string())?;
    let result = core.add_nginx_stream(&s).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_stream(
    core: State<'_, CoreService>,
    stream: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_stream() called");
    let s: supertool_core::db::nginx::NginxStream =
        serde_json::from_value(stream).map_err(|e| e.to_string())?;
    let result = core.update_nginx_stream(&s).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_stream(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_stream() called");
    let result = core.delete_nginx_stream(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxCert Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_certs_by_preset(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_certs_by_preset() called");
    let result = core.get_certs_by_preset(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_cert(
    core: State<'_, CoreService>,
    cert: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_cert() called");
    let c: supertool_core::db::nginx::NginxCert =
        serde_json::from_value(cert).map_err(|e| e.to_string())?;
    let result = core.add_nginx_cert(&c).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_cert(
    core: State<'_, CoreService>,
    cert: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_cert() called");
    let c: supertool_core::db::nginx::NginxCert =
        serde_json::from_value(cert).map_err(|e| e.to_string())?;
    let result = core.update_nginx_cert(&c).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_cert(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_cert() called");
    let result = core.delete_nginx_cert(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxTemplate Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_templates_by_preset(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_templates_by_preset() called");
    let result = core.get_templates_by_preset(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_template(
    core: State<'_, CoreService>,
    template: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_template() called");
    let t: supertool_core::db::nginx::NginxTemplate =
        serde_json::from_value(template).map_err(|e| e.to_string())?;
    let result = core.add_nginx_template(&t).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_template(
    core: State<'_, CoreService>,
    template: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_template() called");
    let t: supertool_core::db::nginx::NginxTemplate =
        serde_json::from_value(template).map_err(|e| e.to_string())?;
    let result = core.update_nginx_template(&t).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_template(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_template() called");
    let result = core.delete_nginx_template(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxBasicSetting Commands (key-value) ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_basic_settings(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_basic_settings() called");
    let result = core.get_basic_settings(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_basic_settings(
    core: State<'_, CoreService>,
    preset_id: String,
    settings: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] save_basic_settings() called");
    let items: Vec<supertool_core::db::nginx::NginxBasicSetting> = settings
        .into_iter()
        .map(|v| serde_json::from_value(v).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let result = core.save_basic_settings(&preset_id, &items).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxParam Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_params_by_preset(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_params_by_preset() called");
    let result = core.get_params_by_preset(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_param(
    core: State<'_, CoreService>,
    param: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_param() called");
    let p: supertool_core::db::nginx::NginxParam =
        serde_json::from_value(param).map_err(|e| e.to_string())?;
    let result = core.add_nginx_param(&p).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_param(
    core: State<'_, CoreService>,
    param: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_param() called");
    let p: supertool_core::db::nginx::NginxParam =
        serde_json::from_value(param).map_err(|e| e.to_string())?;
    let result = core.update_nginx_param(&p).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_param(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_param() called");
    let result = core.delete_nginx_param(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxDenyAllow Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_deny_allows_by_preset(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_deny_allows_by_preset() called");
    let result = core.get_deny_allows_by_preset(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_deny_allow(
    core: State<'_, CoreService>,
    deny_allow: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_deny_allow() called");
    let d: supertool_core::db::nginx::NginxDenyAllow =
        serde_json::from_value(deny_allow).map_err(|e| e.to_string())?;
    let result = core.add_nginx_deny_allow(&d).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_deny_allow(
    core: State<'_, CoreService>,
    deny_allow: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_deny_allow() called");
    let d: supertool_core::db::nginx::NginxDenyAllow =
        serde_json::from_value(deny_allow).map_err(|e| e.to_string())?;
    let result = core.update_nginx_deny_allow(&d).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_deny_allow(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_deny_allow() called");
    let result = core.delete_nginx_deny_allow(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ NginxPassword Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn get_passwords_by_preset(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_passwords_by_preset() called");
    let result = core.get_passwords_by_preset(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_password(
    core: State<'_, CoreService>,
    password: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_password() called");
    let pw: supertool_core::db::nginx::NginxPassword =
        serde_json::from_value(password).map_err(|e| e.to_string())?;
    let result = core.add_nginx_password(&pw).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_password(
    core: State<'_, CoreService>,
    password: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_password() called");
    let pw: supertool_core::db::nginx::NginxPassword =
        serde_json::from_value(password).map_err(|e| e.to_string())?;
    let result = core.update_nginx_password(&pw).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_password(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_password() called");
    let result = core.delete_nginx_password(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ Config Generation ============

#[tauri::command(rename_all = "camelCase")]
pub async fn generate_nginx_config(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] generate_nginx_config() called");
    let result = core.generate_nginx_config(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn generate_nginx_config_decomposed(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] generate_nginx_config_decomposed() called");
    let result = core.generate_nginx_config_decomposed(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

// ============ Config Import ============

#[tauri::command(rename_all = "camelCase")]
pub async fn import_nginx_config(
    core: State<'_, CoreService>,
    preset_id: String,
    config_text: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] import_nginx_config() called");
    let result = core.import_nginx_config(&preset_id, &config_text).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}
