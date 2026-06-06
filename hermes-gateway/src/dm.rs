//! DM (Direct Message) pairing mechanism (Requirement 7.9).
//!
//! Handles authorization decisions when an unregistered user sends a
//! direct message to the bot. Supports configurable behaviors:
//! - Pair: Create a session and request admin approval
//! - Ignore: Silently discard the message

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;

use hermes_config::UnauthorizedDmBehavior;

// ---------------------------------------------------------------------------
// DmDecision
// ---------------------------------------------------------------------------

/// Decision outcome for a DM from an unregistered/unauthorized user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DmDecision {
    /// Allow the DM through (user is authorized).
    Allow,
    /// Pair the user: create a session and request admin approval.
    Pair {
        /// A message to show the user while awaiting approval.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Deny the DM entirely.
    Deny,
}

// ---------------------------------------------------------------------------
// DmManager
// ---------------------------------------------------------------------------

/// Manages DM authorization decisions for incoming messages.
pub struct DmManager {
    /// Set of user IDs that are explicitly authorized to DM the bot.
    authorized_users: HashSet<String>,

    /// Set of user IDs that have admin privileges.
    admin_users: HashSet<String>,

    /// Per-platform user IDs from config/env allowlists.
    platform_authorized_users: HashMap<String, HashSet<String>>,

    /// Per-platform admin user IDs from config/env allowlists.
    platform_admin_users: HashMap<String, HashSet<String>>,

    /// Per-platform group sender allowlists. These never authorize DMs.
    platform_group_authorized_users: HashMap<String, HashSet<String>>,

    /// Per-platform group chat allowlists. These never authorize DMs.
    platform_group_authorized_chats: HashMap<String, HashSet<String>>,

    /// Per-platform aliases such as WhatsApp LID <-> phone mappings.
    platform_user_aliases: HashMap<String, HashMap<String, HashSet<String>>>,

    /// How to handle DMs from unauthorized users.
    unauthorized_dm_behavior: UnauthorizedDmBehavior,

    /// Per-platform unauthorized-DM policy. This preserves Python behavior
    /// where strict allowlists default to ignoring strangers instead of pairing.
    platform_unauthorized_dm_behavior: HashMap<String, UnauthorizedDmBehavior>,

    /// Pending pairing codes keyed by platform and user.
    pairing_codes: Mutex<HashMap<DmPairingKey, String>>,

    /// Users whose pairing requests are currently rate-limited.
    pairing_rate_limited: Mutex<HashSet<DmPairingKey>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DmPairingKey {
    platform: String,
    user_id: String,
}

impl DmManager {
    /// Create a new `DmManager`.
    pub fn new(
        authorized_users: HashSet<String>,
        admin_users: HashSet<String>,
        unauthorized_dm_behavior: UnauthorizedDmBehavior,
    ) -> Self {
        Self {
            authorized_users,
            admin_users,
            platform_authorized_users: HashMap::new(),
            platform_admin_users: HashMap::new(),
            platform_group_authorized_users: HashMap::new(),
            platform_group_authorized_chats: HashMap::new(),
            platform_user_aliases: HashMap::new(),
            unauthorized_dm_behavior,
            platform_unauthorized_dm_behavior: HashMap::new(),
            pairing_codes: Mutex::new(HashMap::new()),
            pairing_rate_limited: Mutex::new(HashSet::new()),
        }
    }

    /// Create a `DmManager` with the Pair behavior and no pre-authorized users.
    pub fn with_pair_behavior() -> Self {
        Self {
            authorized_users: HashSet::new(),
            admin_users: HashSet::new(),
            platform_authorized_users: HashMap::new(),
            platform_admin_users: HashMap::new(),
            platform_group_authorized_users: HashMap::new(),
            platform_group_authorized_chats: HashMap::new(),
            platform_user_aliases: HashMap::new(),
            unauthorized_dm_behavior: UnauthorizedDmBehavior::Pair,
            platform_unauthorized_dm_behavior: HashMap::new(),
            pairing_codes: Mutex::new(HashMap::new()),
            pairing_rate_limited: Mutex::new(HashSet::new()),
        }
    }

    /// Create a `DmManager` with the Ignore behavior and no pre-authorized users.
    pub fn with_ignore_behavior() -> Self {
        Self {
            authorized_users: HashSet::new(),
            admin_users: HashSet::new(),
            platform_authorized_users: HashMap::new(),
            platform_admin_users: HashMap::new(),
            platform_group_authorized_users: HashMap::new(),
            platform_group_authorized_chats: HashMap::new(),
            platform_user_aliases: HashMap::new(),
            unauthorized_dm_behavior: UnauthorizedDmBehavior::Ignore,
            platform_unauthorized_dm_behavior: HashMap::new(),
            pairing_codes: Mutex::new(HashMap::new()),
            pairing_rate_limited: Mutex::new(HashSet::new()),
        }
    }

    fn platform_key(platform: &str) -> String {
        let key = platform.trim().to_ascii_lowercase();
        match key.as_str() {
            "qq" | "qq_bot" => "qqbot".to_string(),
            _ => key,
        }
    }

    fn user_matches_any(user_id: &str, set: &HashSet<String>) -> bool {
        let candidate = user_id.trim();
        if candidate.is_empty() {
            return false;
        }
        let candidate_no_at = candidate.strip_prefix('@').unwrap_or(candidate);
        set.iter().any(|entry| {
            let allowed = entry.trim();
            if allowed.is_empty() {
                return false;
            }
            if allowed == "*" {
                return true;
            }
            let allowed_no_at = allowed.strip_prefix('@').unwrap_or(allowed);
            allowed.eq_ignore_ascii_case(candidate)
                || allowed.eq_ignore_ascii_case(candidate_no_at)
                || allowed_no_at.eq_ignore_ascii_case(candidate)
                || allowed_no_at.eq_ignore_ascii_case(candidate_no_at)
        })
    }

    fn user_variants_for_platform(platform: &str, user_id: &str) -> HashSet<String> {
        let mut variants = HashSet::new();
        let candidate = user_id.trim();
        if candidate.is_empty() {
            return variants;
        }

        variants.insert(candidate.to_string());
        variants.insert(candidate.strip_prefix('@').unwrap_or(candidate).to_string());

        if Self::platform_key(platform) == "whatsapp" {
            let base = candidate
                .strip_suffix("@lid")
                .or_else(|| candidate.strip_suffix("@s.whatsapp.net"))
                .unwrap_or(candidate)
                .trim();
            if !base.is_empty() {
                variants.insert(base.to_string());
                if let Some(no_plus) = base.strip_prefix('+') {
                    if !no_plus.is_empty() {
                        variants.insert(no_plus.to_string());
                    }
                } else if base.chars().all(|c| c.is_ascii_digit()) {
                    variants.insert(format!("+{base}"));
                }
            }
        }

        variants
    }

    fn alias_key(value: &str) -> String {
        value.trim().to_ascii_lowercase()
    }

    fn insert_alias(
        aliases: &mut HashMap<String, HashSet<String>>,
        platform: &str,
        key: &str,
        alias: &str,
    ) {
        for key_variant in Self::user_variants_for_platform(platform, key) {
            let key_variant = Self::alias_key(&key_variant);
            if key_variant.is_empty() {
                continue;
            }
            let entry = aliases.entry(key_variant).or_default();
            for alias_variant in Self::user_variants_for_platform(platform, alias) {
                let alias_variant = alias_variant.trim();
                if !alias_variant.is_empty() {
                    entry.insert(alias_variant.to_string());
                }
            }
        }
    }

    fn user_matches_platform_set(
        &self,
        platform: &str,
        user_id: &str,
        set: &HashSet<String>,
    ) -> bool {
        if Self::user_matches_any(user_id, set) {
            return true;
        }

        let platform = Self::platform_key(platform);
        let variants = Self::user_variants_for_platform(&platform, user_id);
        if variants
            .iter()
            .any(|variant| Self::user_matches_any(variant, set))
        {
            return true;
        }

        let Some(aliases) = self.platform_user_aliases.get(&platform) else {
            return false;
        };
        variants.iter().any(|variant| {
            aliases
                .get(&Self::alias_key(variant))
                .is_some_and(|mapped| {
                    mapped
                        .iter()
                        .any(|alias| Self::user_matches_any(alias, set))
                })
        })
    }

    fn chat_matches_any(chat_id: &str, set: &HashSet<String>) -> bool {
        let candidate = chat_id.trim();
        if candidate.is_empty() {
            return false;
        }
        set.iter().any(|entry| {
            let allowed = entry.trim();
            allowed == "*" || allowed.eq_ignore_ascii_case(candidate)
        })
    }

    fn pairing_key(platform: &str, user_id: &str) -> Option<DmPairingKey> {
        let platform = Self::platform_key(platform);
        let user_id = user_id.trim();
        if platform.is_empty() || user_id.is_empty() {
            return None;
        }
        Some(DmPairingKey {
            platform,
            user_id: user_id.to_string(),
        })
    }

    fn new_pairing_code() -> String {
        uuid::Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
            .to_ascii_uppercase()
    }

    fn pairing_message(code: &str) -> String {
        format!(
            "Your pairing code is {code}. Send this code to an admin to approve this gateway session."
        )
    }

    fn pending_pairing_message(&self, platform: &str, user_id: &str) -> Option<String> {
        let key = Self::pairing_key(platform, user_id)?;
        if self
            .pairing_rate_limited
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .contains(&key)
        {
            return None;
        }
        let mut codes = self
            .pairing_codes
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let code = codes
            .entry(key)
            .or_insert_with(Self::new_pairing_code)
            .clone();
        Some(Self::pairing_message(&code))
    }

    /// Handle an incoming DM from a user on a platform.
    ///
    /// Returns a `DmDecision` indicating how to proceed:
    /// - `Allow` if the user is authorized or is an admin
    /// - `Pair` if unauthorized and behavior is Pair
    /// - `Deny` if unauthorized and behavior is Ignore
    pub async fn handle_dm(&self, user_id: &str, _platform: &str) -> DmDecision {
        let platform = Self::platform_key(_platform);
        if self.is_authorized_for_platform(&platform, user_id) {
            return DmDecision::Allow;
        }

        // Unauthorized user: apply the configured behavior
        let behavior = self
            .platform_unauthorized_dm_behavior
            .get(&platform)
            .copied()
            .unwrap_or(self.unauthorized_dm_behavior);
        match behavior {
            UnauthorizedDmBehavior::Pair => {
                match self.pending_pairing_message(&platform, user_id) {
                    Some(message) => DmDecision::Pair {
                        message: Some(message),
                    },
                    None => DmDecision::Deny,
                }
            }
            UnauthorizedDmBehavior::Ignore => DmDecision::Deny,
        }
    }

    /// Add a user to the authorized users set.
    pub fn authorize_user(&mut self, user_id: impl Into<String>) {
        self.authorized_users.insert(user_id.into());
    }

    /// Add a platform-scoped user to the authorized users set.
    pub fn authorize_user_for_platform(
        &mut self,
        platform: impl AsRef<str>,
        user_id: impl Into<String>,
    ) {
        self.platform_authorized_users
            .entry(Self::platform_key(platform.as_ref()))
            .or_default()
            .insert(user_id.into());
    }

    /// Add a platform-scoped group sender allowlist entry.
    pub fn authorize_group_user_for_platform(
        &mut self,
        platform: impl AsRef<str>,
        user_id: impl Into<String>,
    ) {
        self.platform_group_authorized_users
            .entry(Self::platform_key(platform.as_ref()))
            .or_default()
            .insert(user_id.into());
    }

    /// Add a platform-scoped group chat allowlist entry.
    pub fn authorize_group_chat_for_platform(
        &mut self,
        platform: impl AsRef<str>,
        chat_id: impl Into<String>,
    ) {
        self.platform_group_authorized_chats
            .entry(Self::platform_key(platform.as_ref()))
            .or_default()
            .insert(chat_id.into());
    }

    /// Add a platform-scoped user alias, for example WhatsApp LID <-> phone ID.
    pub fn add_user_alias_for_platform(
        &mut self,
        platform: impl AsRef<str>,
        user_id: impl AsRef<str>,
        alias: impl AsRef<str>,
    ) {
        let platform = Self::platform_key(platform.as_ref());
        let user_id = user_id.as_ref().trim();
        let alias = alias.as_ref().trim();
        if user_id.is_empty() || alias.is_empty() {
            return;
        }
        let aliases = self
            .platform_user_aliases
            .entry(platform.clone())
            .or_default();
        Self::insert_alias(aliases, &platform, user_id, alias);
        Self::insert_alias(aliases, &platform, alias, user_id);
    }

    /// Load WhatsApp LID mapping files from `$HERMES_HOME/whatsapp/session`.
    pub fn load_whatsapp_lid_mappings_from_home(&mut self, home: impl AsRef<Path>) -> usize {
        let session_dir = home.as_ref().join("whatsapp").join("session");
        let Ok(entries) = std::fs::read_dir(session_dir) else {
            return 0;
        };
        let mut loaded = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Some(raw_key) = file_name
                .strip_prefix("lid-mapping-")
                .and_then(|s| s.strip_suffix(".json"))
            else {
                continue;
            };
            let key = raw_key.strip_suffix("_reverse").unwrap_or(raw_key);
            let Ok(raw) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<String>(&raw) else {
                continue;
            };
            self.add_user_alias_for_platform("whatsapp", key, value);
            loaded += 1;
        }
        loaded
    }

    /// Remove a user from the authorized users set.
    pub fn deauthorize_user(&mut self, user_id: &str) {
        self.authorized_users.remove(user_id);
    }

    /// Add a user to the admin users set.
    pub fn add_admin(&mut self, user_id: impl Into<String>) {
        self.admin_users.insert(user_id.into());
    }

    /// Add a platform-scoped admin user.
    pub fn add_admin_for_platform(
        &mut self,
        platform: impl AsRef<str>,
        user_id: impl Into<String>,
    ) {
        self.platform_admin_users
            .entry(Self::platform_key(platform.as_ref()))
            .or_default()
            .insert(user_id.into());
    }

    /// Remove a user from the admin users set.
    pub fn remove_admin(&mut self, user_id: &str) {
        self.admin_users.remove(user_id);
    }

    /// Check if a user is authorized.
    pub fn is_authorized(&self, user_id: &str) -> bool {
        Self::user_matches_any(user_id, &self.authorized_users)
            || Self::user_matches_any(user_id, &self.admin_users)
    }

    /// Check if a user is authorized globally or for the given platform.
    pub fn is_authorized_for_platform(&self, platform: &str, user_id: &str) -> bool {
        let platform = Self::platform_key(platform);
        self.platform_authorized_users
            .get(&platform)
            .is_some_and(|users| self.user_matches_platform_set(&platform, user_id, users))
            || self
                .platform_admin_users
                .get(&platform)
                .is_some_and(|admins| self.user_matches_platform_set(&platform, user_id, admins))
            || self.user_matches_platform_set(&platform, user_id, &self.authorized_users)
            || self.user_matches_platform_set(&platform, user_id, &self.admin_users)
    }

    /// Check if a DM or group source is authorized for the given platform.
    pub fn is_authorized_source(
        &self,
        platform: &str,
        user_id: &str,
        chat_id: &str,
        is_dm: bool,
    ) -> bool {
        if is_dm {
            return self.is_authorized_for_platform(platform, user_id);
        }

        let platform = Self::platform_key(platform);
        self.platform_group_authorized_users
            .get(&platform)
            .is_some_and(|users| self.user_matches_platform_set(&platform, user_id, users))
            || self
                .platform_group_authorized_chats
                .get(&platform)
                .is_some_and(|chats| Self::chat_matches_any(chat_id, chats))
            || self.is_authorized_for_platform(&platform, user_id)
    }

    /// Check if a user is an admin.
    pub fn is_admin(&self, user_id: &str) -> bool {
        Self::user_matches_any(user_id, &self.admin_users)
    }

    /// Get the number of authorized users.
    pub fn authorized_user_count(&self) -> usize {
        self.authorized_users.len()
    }

    /// Get the number of admin users.
    pub fn admin_user_count(&self) -> usize {
        self.admin_users.len()
    }

    /// Override unauthorized-DM behavior for a platform.
    pub fn set_platform_unauthorized_behavior(
        &mut self,
        platform: impl AsRef<str>,
        behavior: UnauthorizedDmBehavior,
    ) {
        self.platform_unauthorized_dm_behavior
            .insert(Self::platform_key(platform.as_ref()), behavior);
    }

    /// Mark a user's pairing flow as rate-limited.
    pub fn record_pairing_rate_limit(&self, platform: &str, user_id: &str) {
        if let Some(key) = Self::pairing_key(platform, user_id) {
            self.pairing_rate_limited
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .insert(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dm_manager_allows_authorized_user() {
        let mut dm = DmManager::with_ignore_behavior();
        dm.authorize_user("user1");

        let decision = dm.handle_dm("user1", "telegram").await;
        assert_eq!(decision, DmDecision::Allow);
    }

    #[tokio::test]
    async fn dm_manager_allows_admin() {
        let mut dm = DmManager::with_ignore_behavior();
        dm.add_admin("admin1");

        let decision = dm.handle_dm("admin1", "discord").await;
        assert_eq!(decision, DmDecision::Allow);
    }

    #[tokio::test]
    async fn dm_manager_pair_behavior() {
        let dm = DmManager::with_pair_behavior();
        let decision = dm.handle_dm("unknown_user", "telegram").await;
        let DmDecision::Pair { message } = decision else {
            panic!("expected pairing decision");
        };
        let message = message.expect("pairing message");
        assert!(message.contains("pairing code"));
    }

    #[tokio::test]
    async fn dm_manager_ignore_behavior() {
        let dm = DmManager::with_ignore_behavior();
        let decision = dm.handle_dm("unknown_user", "telegram").await;
        assert_eq!(decision, DmDecision::Deny);
    }

    #[tokio::test]
    async fn dm_manager_authorize_and_deauthorize() {
        let mut dm = DmManager::with_ignore_behavior();
        dm.authorize_user("user1");
        assert!(dm.is_authorized("user1"));

        dm.deauthorize_user("user1");
        assert!(!dm.is_authorized("user1"));
    }

    #[tokio::test]
    async fn dm_manager_wildcard_authorizes_any_non_empty_user() {
        let mut dm = DmManager::with_ignore_behavior();
        dm.authorize_user("*");

        assert_eq!(dm.handle_dm("user1", "telegram").await, DmDecision::Allow);
        assert_eq!(dm.handle_dm("999", "discord").await, DmDecision::Allow);
        assert_eq!(dm.handle_dm("", "discord").await, DmDecision::Deny);
    }

    #[tokio::test]
    async fn dm_manager_platform_scoped_allowlist_does_not_cross_authorize() {
        let mut dm = DmManager::with_pair_behavior();
        dm.authorize_user_for_platform("telegram", "123");
        dm.set_platform_unauthorized_behavior("telegram", UnauthorizedDmBehavior::Ignore);

        assert_eq!(dm.handle_dm("123", "telegram").await, DmDecision::Allow);
        assert!(matches!(
            dm.handle_dm("123", "discord").await,
            DmDecision::Pair { .. }
        ));
        assert_eq!(dm.handle_dm("999", "telegram").await, DmDecision::Deny);
    }

    #[tokio::test]
    async fn dm_manager_group_allowlists_do_not_authorize_dms() {
        let mut dm = DmManager::with_ignore_behavior();
        dm.authorize_group_user_for_platform("telegram", "999");
        dm.authorize_group_chat_for_platform("telegram", "-1001878443972");

        assert!(dm.is_authorized_source("telegram", "999", "-1001878443972", false));
        assert!(dm.is_authorized_source("telegram", "123", "-1001878443972", false));
        assert!(!dm.is_authorized_source("telegram", "999", "999", true));
    }

    #[tokio::test]
    async fn dm_manager_whatsapp_lid_alias_authorizes_phone_allowlist() {
        let mut dm = DmManager::with_ignore_behavior();
        dm.authorize_user_for_platform("whatsapp", "15550000001");
        dm.add_user_alias_for_platform("whatsapp", "900000000000001", "15550000001");

        assert_eq!(
            dm.handle_dm("900000000000001@lid", "whatsapp").await,
            DmDecision::Allow
        );
    }

    #[tokio::test]
    async fn dm_manager_rate_limited_pairing_gets_no_response() {
        let dm = DmManager::with_pair_behavior();
        let first = dm.handle_dm("15551234567@s.whatsapp.net", "whatsapp").await;
        let DmDecision::Pair { message } = first else {
            panic!("expected pairing decision");
        };
        assert!(message.expect("pairing message").contains("pairing code"));

        dm.record_pairing_rate_limit("whatsapp", "15551234567@s.whatsapp.net");
        assert_eq!(
            dm.handle_dm("15551234567@s.whatsapp.net", "whatsapp").await,
            DmDecision::Deny
        );
    }

    #[test]
    fn dm_decision_serde() {
        let allow = DmDecision::Allow;
        let json = serde_json::to_string(&allow).unwrap();
        assert!(json.contains("allow"));

        let pair = DmDecision::Pair {
            message: Some("pending".to_string()),
        };
        let json = serde_json::to_string(&pair).unwrap();
        assert!(json.contains("pair"));

        let deny = DmDecision::Deny;
        let json = serde_json::to_string(&deny).unwrap();
        assert!(json.contains("deny"));
    }
}
