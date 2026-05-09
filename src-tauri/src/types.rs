use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RarityTier {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl RarityTier {
    pub fn from_str(s: &str) -> Self {
        match s {
            "uncommon" => Self::Uncommon,
            "rare" => Self::Rare,
            "legendary" => Self::Legendary,
            _ => Self::Common,
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Legendary => "legendary",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CreatureStatus {
    Egg,
    Alive,
    Retired,
    Dead,
}

impl CreatureStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "egg" => Self::Egg,
            "retired" => Self::Retired,
            "dead" => Self::Dead,
            _ => Self::Alive,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    pub id: String,
    pub name: String,
    pub rarity_tier: RarityTier,
    pub ascii_idle: String,
    pub ascii_happy: String,
    pub ascii_hungry: String,
    pub ascii_special: Option<String>,
    pub description: String,
    pub codex_flavor_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creature {
    pub id: i64,
    pub species_id: String,
    pub nickname: Option<String>,
    pub hatched_at: i64,
    pub retired_at: Option<i64>,
    pub is_active: bool,
    pub age_ticks: i64,
    pub status: CreatureStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureStats {
    pub creature_id: i64,
    pub hunger: i32,
    pub happiness: i32,
    pub energy: i32,
    pub xp: i64,
    pub milestone_count: i32,
    pub starving_ticks: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureState {
    pub creature: Creature,
    pub species: Species,
    pub stats: CreatureStats,
    pub egg_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexEntry {
    pub species: Species,
    pub discovered: bool,
    pub first_hatched_at: Option<i64>,
    pub times_hatched: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub always_on_top: bool,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub tick_interval_secs: u64,
    pub show_tray_tooltip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsPatch {
    pub always_on_top: Option<bool>,
    pub window_x: Option<Option<i32>>,
    pub window_y: Option<Option<i32>>,
    pub tick_interval_secs: Option<u64>,
    pub show_tray_tooltip: Option<bool>,
}

/// Stub for future APM integration. v1 always passes None.
#[derive(Debug, Clone)]
pub struct ActivitySnapshot {
    pub apm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickEvent {
    pub creature_state: Option<CreatureState>,
    pub egg_count: i32,
}
