use crate::types::*;
use rand::Rng;

// ── Stat constants ──────────────────────────────────────────────────────────
pub const HUNGER_DECAY: i32 = 2;
pub const HAPPINESS_DECAY: i32 = 1;
pub const ENERGY_REGEN: i32 = 1;

pub const FEED_HUNGER: i32 = 20;
pub const FEED_ENERGY: i32 = -5;
pub const FEED_XP: i64 = 2;

pub const PLAY_HAPPINESS: i32 = 15;
pub const PLAY_ENERGY: i32 = -10;
pub const PLAY_XP: i64 = 3;

pub const STARVATION_WARNING: i32 = 20;
pub const STARVATION_DEATH_TICKS: i32 = 3;
pub const MAX_EGGS: i32 = 3;
pub const TICK_XP: i64 = 1;

// XP thresholds: index 0 = first milestone, index 1 = second, etc.
// After index 2, every 500 XP beyond 500 is a milestone.
pub fn next_milestone_xp(milestone_count: i32) -> i64 {
    match milestone_count {
        0 => 50,
        1 => 200,
        n => 500 * (n as i64 - 1),
    }
}

// ── Rarity weights ──────────────────────────────────────────────────────────
pub fn roll_rarity_tier(rng: &mut impl Rng) -> RarityTier {
    let roll: u32 = rng.gen_range(0..100);
    match roll {
        0..=59 => RarityTier::Common,
        60..=84 => RarityTier::Uncommon,
        85..=96 => RarityTier::Rare,
        _ => RarityTier::Legendary,
    }
}

// ── Game logic ──────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum LifecycleEvent {
    StarvationWarning,
    Died,
    MilestoneUnlocked,
}

/// Returns updated stats and any lifecycle events that fired.
pub fn apply_tick(
    mut stats: CreatureStats,
    _activity: Option<&ActivitySnapshot>,
) -> (CreatureStats, Vec<LifecycleEvent>) {
    let mut events = Vec::new();

    // Decay / regen
    stats.hunger = (stats.hunger - HUNGER_DECAY).max(0);
    stats.happiness = (stats.happiness - HAPPINESS_DECAY).max(0);
    stats.energy = (stats.energy + ENERGY_REGEN).min(100);
    stats.xp += TICK_XP;

    // Starving ticks
    if stats.hunger == 0 {
        stats.starving_ticks += 1;
    } else {
        stats.starving_ticks = 0;
    }

    // Lifecycle checks
    if stats.starving_ticks >= STARVATION_DEATH_TICKS {
        events.push(LifecycleEvent::Died);
    } else if stats.hunger <= STARVATION_WARNING {
        events.push(LifecycleEvent::StarvationWarning);
    }

    // Milestone check
    let threshold = next_milestone_xp(stats.milestone_count);
    if stats.xp >= threshold {
        events.push(LifecycleEvent::MilestoneUnlocked);
        stats.milestone_count += 1;
    }

    (stats, events)
}

pub fn apply_feed(mut stats: CreatureStats) -> CreatureStats {
    stats.hunger = (stats.hunger + FEED_HUNGER).min(100);
    stats.energy = (stats.energy + FEED_ENERGY).max(0);
    stats.xp += FEED_XP;
    stats
}

pub fn apply_play(mut stats: CreatureStats) -> CreatureStats {
    stats.happiness = (stats.happiness + PLAY_HAPPINESS).min(100);
    stats.energy = (stats.energy + PLAY_ENERGY).max(0);
    stats.xp += PLAY_XP;
    stats
}

// ── Species definitions ─────────────────────────────────────────────────────

pub fn all_species() -> Vec<Species> {
    vec![
        // ── Common ──
        Species {
            id: "blob".into(),
            name: "Blob".into(),
            rarity_tier: RarityTier::Common,
            ascii_idle:   "  ___  \n /o o\\ \n| --- |\n \\___/ ".into(),
            ascii_happy:  "  ___  \n /^ ^\\ \n| ~~~ |\n \\___/ ".into(),
            ascii_hungry: "  ___  \n /- -\\ \n| ... |\n \\___/ ".into(),
            ascii_special: None,
            description: "A wobbly, amorphous companion.".into(),
            codex_flavor_text: "No one knows what Blob is made of, and Blob doesn't care.".into(),
        },
        Species {
            id: "kitten".into(),
            name: "Kitten".into(),
            rarity_tier: RarityTier::Common,
            ascii_idle:   "/\\ /\\\n(o . o)\n > w <\n(_____)\n      ".into(),
            ascii_happy:  "/\\ /\\\n(^ . ^)\n > U <\n(_____)\n      ".into(),
            ascii_hungry: "/\\ /\\\n(; . ;)\n > _ <\n(_____)\n      ".into(),
            ascii_special: None,
            description: "A tiny ASCII cat with very strong opinions.".into(),
            codex_flavor_text: "Reportedly domesticated. Evidence is inconclusive.".into(),
        },
        Species {
            id: "sparrow".into(),
            name: "Sparrow".into(),
            rarity_tier: RarityTier::Common,
            ascii_idle:   "  __  \n (oo) \n-/||\\ \n  \\/  ".into(),
            ascii_happy:  "  __  \n (^^) \n-/||\\ \n  \\/  ".into(),
            ascii_hungry: "  __  \n (..) \n-/||\\ \n  \\/  ".into(),
            ascii_special: None,
            description: "Chirpy and round.".into(),
            codex_flavor_text: "Eats approximately its own weight in seeds daily.".into(),
        },
        Species {
            id: "bearcub".into(),
            name: "Bearcub".into(),
            rarity_tier: RarityTier::Common,
            ascii_idle:   "  _   _\n (eYe)\n /|_|\\ \n  \\_/  ".into(),
            ascii_happy:  "  _   _\n (^Y^)\n /|_|\\ \n  \\_/  ".into(),
            ascii_hungry: "  _   _\n (;Y;)\n /|_|\\ \n  \\_/  ".into(),
            ascii_special: None,
            description: "Perpetually sleepy.".into(),
            codex_flavor_text: "Hibernates 14 hours a day. Judgemental about the other 10.".into(),
        },
        // ── Uncommon ──
        Species {
            id: "drakling".into(),
            name: "Drakling".into(),
            rarity_tier: RarityTier::Uncommon,
            ascii_idle:   "^ ^  \n(o.o)\n)|||(\n  V  ".into(),
            ascii_happy:  "^ ^  \n(^.^)\n)|||(\n  V  ".into(),
            ascii_hungry: "^ ^  \n(-.-)\n)|||(\n  V  ".into(),
            ascii_special: None,
            description: "A small dragon, mostly harmless.".into(),
            codex_flavor_text: "Can technically breathe fire. Mostly breathes heavy sighs.".into(),
        },
        Species {
            id: "specter".into(),
            name: "Specter".into(),
            rarity_tier: RarityTier::Uncommon,
            ascii_idle:   ".~~~.\n/o   o\\\n| ^ |\n \\   /\n~~\\/~~".into(),
            ascii_happy:  ".~~~.\n/^ ^ ^\\\n| w |\n \\   /\n~~\\/~~".into(),
            ascii_hungry: ".~~~.\n/. . .\\\n| _ |\n \\   /\n~~\\/~~".into(),
            ascii_special: None,
            description: "Floats. Occasionally vanishes mid-conversation.".into(),
            codex_flavor_text: "Technically not alive. Doesn't let that slow it down.".into(),
        },
        // ── Rare ──
        Species {
            id: "phoenix".into(),
            name: "Phoenix".into(),
            rarity_tier: RarityTier::Rare,
            ascii_idle:   " /\\ \n/oo\\\n|~~|\n\\^^/\n >>> ".into(),
            ascii_happy:  " /\\ \n/^^\\\n|~~|\n\\^^/\n >>>".into(),
            ascii_hungry: " /\\ \n/..\\\n|--|\n\\--/\n ... ".into(),
            ascii_special: None,
            description: "A flame bird. Self-renewing. A bit smug about it.".into(),
            codex_flavor_text: "Has died 0 times. (This figure is contested.)".into(),
        },
        // ── Legendary ──
        Species {
            id: "voidling".into(),
            name: "Voidling".into(),
            rarity_tier: RarityTier::Legendary,
            ascii_idle:   "*  *  *\n* \\|/ *\n (o_o)\n* /|\\ *\n*  *  *".into(),
            ascii_happy:  "*  *  *\n* \\|/ *\n (^_^)\n* /|\\ *\n*  *  *".into(),
            ascii_hungry: "*  *  *\n* \\|/ *\n (O_O)\n* /|\\ *\n*  *  *".into(),
            ascii_special: Some("+ . + .\n. \\|/ .\n(>^_^<)\n. /|\\ .\n+ . + .".into()),
            description: "A being of pure cosmic energy. Handle with care.".into(),
            codex_flavor_text: "Exists in all states simultaneously. Prefers not to discuss it.".into(),
        },
    ]
}

pub fn species_by_tier<'a>(all: &'a [Species], tier: &RarityTier) -> Vec<&'a Species> {
    all.iter().filter(|s| &s.rarity_tier == tier).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_stats() -> CreatureStats {
        CreatureStats {
            creature_id: 1,
            hunger: 50,
            happiness: 50,
            energy: 50,
            xp: 0,
            milestone_count: 0,
            starving_ticks: 0,
        }
    }

    #[test]
    fn apply_tick_decays_stats() {
        let stats = base_stats();
        let (result, events) = apply_tick(stats, None);
        assert_eq!(result.hunger, 48);
        assert_eq!(result.happiness, 49);
        assert_eq!(result.energy, 51);
        assert_eq!(result.xp, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn apply_tick_clamps_at_zero() {
        let mut stats = base_stats();
        stats.hunger = 1;
        stats.happiness = 0;
        stats.energy = 99;
        let (result, _) = apply_tick(stats, None);
        assert_eq!(result.hunger, 0);
        assert_eq!(result.happiness, 0);
        assert_eq!(result.energy, 100);
    }

    #[test]
    fn apply_tick_fires_starvation_warning_at_threshold() {
        let mut stats = base_stats();
        stats.hunger = 22;
        let (_, events) = apply_tick(stats, None);
        assert!(events.contains(&LifecycleEvent::StarvationWarning));
    }

    #[test]
    fn apply_tick_increments_starving_ticks() {
        let mut stats = base_stats();
        stats.hunger = 2;
        let (result, _) = apply_tick(stats, None);
        assert_eq!(result.starving_ticks, 1);
    }

    #[test]
    fn apply_tick_resets_starving_ticks_when_hungry_recovers() {
        let mut stats = base_stats();
        stats.hunger = 0;
        stats.starving_ticks = 1;
        let (result, _) = apply_tick(stats.clone(), None);
        assert_eq!(result.starving_ticks, 2);

        let mut recovered = result;
        recovered.hunger = 50;
        recovered.starving_ticks = 2;
        let (final_stats, _) = apply_tick(recovered, None);
        assert_eq!(final_stats.starving_ticks, 0);
    }

    #[test]
    fn apply_tick_fires_died_after_three_starving_ticks() {
        let mut stats = base_stats();
        stats.hunger = 0;
        stats.starving_ticks = STARVATION_DEATH_TICKS - 1;
        let (_, events) = apply_tick(stats, None);
        assert!(events.contains(&LifecycleEvent::Died));
    }

    #[test]
    fn apply_tick_fires_milestone_at_50_xp() {
        let mut stats = base_stats();
        stats.xp = 49;
        let (result, events) = apply_tick(stats, None);
        assert_eq!(result.xp, 50);
        assert!(events.contains(&LifecycleEvent::MilestoneUnlocked));
        assert_eq!(result.milestone_count, 1);
    }

    #[test]
    fn next_milestone_xp_thresholds_are_correct() {
        assert_eq!(next_milestone_xp(0), 50);
        assert_eq!(next_milestone_xp(1), 200);
        assert_eq!(next_milestone_xp(2), 500);
        assert_eq!(next_milestone_xp(3), 1000);
        assert_eq!(next_milestone_xp(4), 1500);
    }

    #[test]
    fn apply_feed_clamps_at_100() {
        let mut stats = base_stats();
        stats.hunger = 90;
        let result = apply_feed(stats);
        assert_eq!(result.hunger, 100);
        assert_eq!(result.energy, 45);
        assert_eq!(result.xp, FEED_XP);
    }

    #[test]
    fn apply_play_clamps_energy_at_zero() {
        let mut stats = base_stats();
        stats.energy = 5;
        let result = apply_play(stats);
        assert_eq!(result.energy, 0);
        assert_eq!(result.happiness, 65);
    }

    #[test]
    fn roll_rarity_tier_roughly_correct_distribution() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let mut common = 0u32;
        let mut uncommon = 0u32;
        let mut rare = 0u32;
        let mut legendary = 0u32;
        for _ in 0..10_000 {
            match roll_rarity_tier(&mut rng) {
                RarityTier::Common => common += 1,
                RarityTier::Uncommon => uncommon += 1,
                RarityTier::Rare => rare += 1,
                RarityTier::Legendary => legendary += 1,
            }
        }
        assert!(common > 5500 && common < 6500, "common={common}");
        assert!(uncommon > 2000 && uncommon < 3000, "uncommon={uncommon}");
        assert!(rare > 700 && rare < 1700, "rare={rare}");
        assert!(legendary > 0 && legendary < 800, "legendary={legendary}");
    }
}
