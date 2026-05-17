use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WireGuardConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "address")]
    pub address: String,
    #[serde(rename = "dns")]
    pub dns: Option<String>,
    #[serde(rename = "mtu")]
    pub mtu: Option<i64>,
    // Peer info
    #[serde(rename = "peerPublicKey")]
    pub peer_public_key: String,
    #[serde(rename = "peerEndpoint")]
    pub peer_endpoint: String,
    #[serde(rename = "peerAllowedIPs")]
    pub peer_allowed_ips: String,
    #[serde(rename = "peerPersistentKeepalive")]
    pub peer_persistent_keepalive: Option<i64>,
    #[serde(rename = "presharedKey")]
    pub preshared_key: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

pub fn get_all(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<WireGuardConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, privateKey, publicKey, address, dns, mtu, peerPublicKey, peerEndpoint, peerAllowedIPs, peerPersistentKeepalive, presharedKey, createdAt, updatedAt FROM wireguard_configs ORDER BY name COLLATE NOCASE"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(WireGuardConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            private_key: row.get(2)?,
            public_key: row.get(3)?,
            address: row.get(4)?,
            dns: row.get(5)?,
            mtu: row.get(6)?,
            peer_public_key: row.get(7)?,
            peer_endpoint: row.get(8)?,
            peer_allowed_ips: row.get(9)?,
            peer_persistent_keepalive: row.get(10)?,
            preshared_key: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?;
    rows.collect()
}

pub fn get_by_id(
    conn: &rusqlite::Connection,
    id: &str,
) -> rusqlite::Result<Option<WireGuardConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, privateKey, publicKey, address, dns, mtu, peerPublicKey, peerEndpoint, peerAllowedIPs, peerPersistentKeepalive, presharedKey, createdAt, updatedAt FROM wireguard_configs WHERE id = ?"
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(WireGuardConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            private_key: row.get(2)?,
            public_key: row.get(3)?,
            address: row.get(4)?,
            dns: row.get(5)?,
            mtu: row.get(6)?,
            peer_public_key: row.get(7)?,
            peer_endpoint: row.get(8)?,
            peer_allowed_ips: row.get(9)?,
            peer_persistent_keepalive: row.get(10)?,
            preshared_key: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?;
    match rows.next() {
        Some(Ok(cfg)) => Ok(Some(cfg)),
        _ => Ok(None),
    }
}

pub fn add(
    conn: &rusqlite::Connection,
    name: &str,
    private_key: &str,
    public_key: &str,
    address: &str,
    dns: Option<&str>,
    mtu: Option<i64>,
    peer_public_key: &str,
    peer_endpoint: &str,
    peer_allowed_ips: &str,
    peer_persistent_keepalive: Option<i64>,
    preshared_key: Option<&str>,
) -> rusqlite::Result<String> {
    let id = format!("wg_{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO wireguard_configs (id, name, privateKey, publicKey, address, dns, mtu, peerPublicKey, peerEndpoint, peerAllowedIPs, peerPersistentKeepalive, presharedKey, createdAt, updatedAt) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        params![id, name, private_key, public_key, address, dns, mtu, peer_public_key, peer_endpoint, peer_allowed_ips, peer_persistent_keepalive, preshared_key, now, now],
    )?;
    Ok(id)
}

pub fn update(
    conn: &rusqlite::Connection,
    id: &str,
    name: &str,
    private_key: &str,
    public_key: &str,
    address: &str,
    dns: Option<&str>,
    mtu: Option<i64>,
    peer_public_key: &str,
    peer_endpoint: &str,
    peer_allowed_ips: &str,
    peer_persistent_keepalive: Option<i64>,
    preshared_key: Option<&str>,
) -> rusqlite::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE wireguard_configs SET name=?2, privateKey=?3, publicKey=?4, address=?5, dns=?6, mtu=?7, peerPublicKey=?8, peerEndpoint=?9, peerAllowedIPs=?10, peerPersistentKeepalive=?11, presharedKey=?12, updatedAt=?13 WHERE id=?1",
        params![id, name, private_key, public_key, address, dns, mtu, peer_public_key, peer_endpoint, peer_allowed_ips, peer_persistent_keepalive, preshared_key, now],
    )?;
    Ok(())
}

pub fn delete(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM wireguard_configs WHERE id = ?", params![id])?;
    Ok(())
}
