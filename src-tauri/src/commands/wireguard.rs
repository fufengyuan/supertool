use crate::core::wireguard::WireGuardManager;
use crate::db::wireguard as db_wg;


#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_get_all(
    core: tauri::State<'_, crate::core::CoreService>,
) -> Result<serde_json::Value, String> {
    let configs = core.db_read(|conn| db_wg::get_all(conn).map_err(|e| e.to_string()))?;
    Ok(serde_json::to_value(configs).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_get_by_id(
    core: tauri::State<'_, crate::core::CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    let config = core.db_read(|conn| db_wg::get_by_id(conn, &id).map_err(|e| e.to_string()))?;
    Ok(serde_json::to_value(config).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_add(
    core: tauri::State<'_, crate::core::CoreService>,
    data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let name = data["name"].as_str().unwrap_or("").to_string();
    let private_key = data["privateKey"].as_str().unwrap_or("").to_string();
    let public_key = data["publicKey"].as_str().unwrap_or("").to_string();
    let address = data["address"].as_str().unwrap_or("10.0.0.2/32").to_string();
    let dns = data["dns"].as_str().map(|s| s.to_string());
    let mtu = data["mtu"].as_i64();
    let peer_public_key = data["peerPublicKey"].as_str().unwrap_or("").to_string();
    let peer_endpoint = data["peerEndpoint"].as_str().unwrap_or("").to_string();
    let peer_allowed_ips = data["peerAllowedIPs"].as_str().unwrap_or("0.0.0.0/0").to_string();
    let peer_keepalive = data["peerPersistentKeepalive"].as_i64();
    let preshared_key = data["presharedKey"].as_str().map(|s| s.to_string());

    let id = core.db_write(|conn| {
        db_wg::add(
            conn, &name, &private_key, &public_key, &address,
            dns.as_deref(), mtu,
            &peer_public_key, &peer_endpoint, &peer_allowed_ips,
            peer_keepalive, preshared_key.as_deref(),
        ).map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "id": id }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_update(
    core: tauri::State<'_, crate::core::CoreService>,
    data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let id = data["id"].as_str().unwrap_or("").to_string();
    let name = data["name"].as_str().unwrap_or("").to_string();
    let private_key = data["privateKey"].as_str().unwrap_or("").to_string();
    let public_key = data["publicKey"].as_str().unwrap_or("").to_string();
    let address = data["address"].as_str().unwrap_or("").to_string();
    let dns = data["dns"].as_str().map(|s| s.to_string());
    let mtu = data["mtu"].as_i64();
    let peer_public_key = data["peerPublicKey"].as_str().unwrap_or("").to_string();
    let peer_endpoint = data["peerEndpoint"].as_str().unwrap_or("").to_string();
    let peer_allowed_ips = data["peerAllowedIPs"].as_str().unwrap_or("").to_string();
    let peer_keepalive = data["peerPersistentKeepalive"].as_i64();
    let preshared_key = data["presharedKey"].as_str().map(|s| s.to_string());

    core.db_write(|conn| {
        db_wg::update(
            conn, &id, &name, &private_key, &public_key, &address,
            dns.as_deref(), mtu,
            &peer_public_key, &peer_endpoint, &peer_allowed_ips,
            peer_keepalive, preshared_key.as_deref(),
        ).map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_delete(
    core: tauri::State<'_, crate::core::CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    core.db_write(|conn| db_wg::delete(conn, &id).map_err(|e| e.to_string()))?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_connect(
    wg: tauri::State<'_, WireGuardManager>,
    config_id: String,
    config_name: String,
    private_key: String,
    peer_public_key: String,
    peer_endpoint: String,
    preshared_key: Option<String>,
) -> Result<serde_json::Value, String> {
    wg.connect(&config_id, &config_name, &private_key, &peer_public_key, &peer_endpoint, preshared_key.as_deref())
        .await
        .map(|_| serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_disconnect(
    wg: tauri::State<'_, WireGuardManager>,
) -> Result<serde_json::Value, String> {
    wg.disconnect().await.map(|_| serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_get_status(
    wg: tauri::State<'_, WireGuardManager>,
) -> Result<serde_json::Value, String> {
    let status = wg.get_status();
    Ok(serde_json::to_value(status).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_generate_keypair() -> Result<serde_json::Value, String> {
    let mgr = WireGuardManager::new();
    let (private, public) = mgr.generate_keypair()?;
    Ok(serde_json::json!({ "privateKey": private, "publicKey": public }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn wireguard_derive_public_key(private_key: String) -> Result<serde_json::Value, String> {
    let public = WireGuardManager::public_key_from_private(&private_key)?;
    Ok(serde_json::json!({ "publicKey": public }))
}
