export type RarityTier = 'common' | 'uncommon' | 'rare' | 'legendary';
export type CreatureStatus = 'egg' | 'alive' | 'retired' | 'dead';

export interface Species {
  id: string;
  name: string;
  rarity_tier: RarityTier;
  ascii_idle: string;
  ascii_happy: string;
  ascii_hungry: string;
  ascii_special: string | null;
  description: string;
  codex_flavor_text: string;
}

export interface Creature {
  id: number;
  species_id: string;
  nickname: string | null;
  hatched_at: number;
  retired_at: number | null;
  is_active: boolean;
  age_ticks: number;
  status: CreatureStatus;
}

export interface CreatureStats {
  creature_id: number;
  hunger: number;
  happiness: number;
  energy: number;
  xp: number;
  milestone_count: number;
  starving_ticks: number;
}

export interface CreatureState {
  creature: Creature;
  species: Species;
  stats: CreatureStats;
  egg_count: number;
}

export interface CodexEntry {
  species: Species;
  discovered: boolean;
  first_hatched_at: number | null;
  times_hatched: number;
}

export interface Settings {
  always_on_top: boolean;
  window_x: number | null;
  window_y: number | null;
  tick_interval_secs: number;
  show_tray_tooltip: boolean;
}

export interface SettingsPatch {
  always_on_top?: boolean;
  window_x?: number | null;
  window_y?: number | null;
  tick_interval_secs?: number;
  show_tray_tooltip?: boolean;
}

export interface TickEvent {
  creature_state: CreatureState | null;
  egg_count: number;
}

export interface CollectionEntry {
  creature: Creature;
  species: Species;
}
