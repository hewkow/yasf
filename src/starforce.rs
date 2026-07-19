use rand::prelude::*;
use rand::rngs::SmallRng;
use num_format::{Locale, ToFormattedString};
use std::time::Duration;

/// The bin resolution (in mesos) used for Monte Carlo cost histogram generation.
/// Each bin represents 100,000,000 mesos (100M).
pub const BIN_SIZE: u64 = 100_000_000;

/// Enhancement mode levels for 15-21 star enhancements.
/// Allows custom rate curves (Level 1 through Level 4) or Standard KMS baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnhancementMode {
    #[default]
    Standard,
    Level1,
    Level2,
    Level3,
    Level4,
}

/// Simulation configuration options for starforcing equipment.
#[derive(Default)]
pub struct EnhanceConfig {
    /// Enhancement mode custom overrides for stars 15 through 21.
    pub mode_15_21 : [EnhancementMode; 7],
    /// Enables Star Catch mini-game (+5% multiplicative success rate boost).
    pub star_catch: bool,
    /// Enables SSF (Shining Star Force) 30% cost reduction event.
    pub ssf_cost_reduce_event: bool,
    /// Enables SSF 30% boom reduction event for stars 1..=21.
    pub ssf_boom_reduce_event: bool,
    /// Enables Safeguard protection for stars 15..=17 (prevents destruction at 3x cost).
    pub safeguard: bool,
}

/// Star property configuration calculated for a specific star rank (0–30).
pub struct StarProp {
    pub stars: u8,
    pub cost_multiply : f64,
    pub success_rate : f64,
    pub boom_rate : f64,

    pub enhance_level : EnhancementMode,
}


impl StarProp {
    pub fn new(stars : u8, config: &EnhanceConfig) -> Self{
        let mut mode = match stars {
            15..=21 => config.mode_15_21[(stars - 15) as usize],
            _ => EnhancementMode::Standard
        };

        let (mut cost_mult, mut success, mut boom) = Self::get_base_rates(stars, mode);
        if config.star_catch {
            let base_success = success;
            success = (base_success * 1.05).min(1.0);
            
            let denom = 1.0 - base_success;
            if denom > 0.0 {
                let left = 1.0 - success; 
                boom = (boom * left) / denom;
            } else {
                boom = 0.0;
            }
        }
        
        if config.safeguard && (15..=17).contains(&stars) && boom > 0.0 {
            // change level to 1 when safeguard is true at 15..=17
            match mode {
                EnhancementMode::Standard | EnhancementMode::Level1 => {}
                EnhancementMode::Level2 | EnhancementMode::Level3 | EnhancementMode::Level4 => {
                    mode = EnhancementMode::Level1;
                }
            }
            boom = 0.0;
            cost_mult = 3.0;
        }
                
        if config.ssf_boom_reduce_event && (1..=21).contains(&stars) {
            // now it's confirmed that ssf applied to new enhancement mode 
            boom *= 0.7;
        }

        if config.ssf_cost_reduce_event {
            match  mode {
                EnhancementMode::Standard | EnhancementMode::Level1 => {
                    cost_mult -= 0.30;
                }
                EnhancementMode::Level2 | EnhancementMode::Level3 | EnhancementMode::Level4 => {
                    cost_mult *= 0.7;
                }
            }
        }
        
        
        Self {
            stars,
            cost_multiply: cost_mult,
            success_rate: success,
            boom_rate: boom,
            enhance_level: mode,
        }
    }

    pub fn get_base_rates( stars : u8, mode: EnhancementMode) -> (f64, f64, f64) {
        match (stars, mode) {
    // Your custom enhancement rules for 15-21
                (15..=16, EnhancementMode::Level1) => (1.0, 0.30, 0.0210),
                (15..=16, EnhancementMode::Level2) => (1.5, 0.30, 0.0140),
                (15..=16, EnhancementMode::Level3) => (2.5, 0.30, 0.0070),
                (15..=16, EnhancementMode::Level4) => (3.0, 0.30, 0.0000),
    
                (17, EnhancementMode::Level1)      => (1.0, 0.15, 0.0680),
                (17, EnhancementMode::Level2)      => (1.5, 0.15, 0.0425),
                (17, EnhancementMode::Level3)      => (2.5, 0.15, 0.0170),
                (17, EnhancementMode::Level4)      => (3.0, 0.15, 0.0000),
    
                (18, EnhancementMode::Level1)      => (1.0, 0.15, 0.0680),
                (18, EnhancementMode::Level2)      => (2.0, 0.12, 0.0440),
                (18, EnhancementMode::Level3)      => (3.5, 0.10, 0.0180),
                (18, EnhancementMode::Level4)      => (6.5, 0.08, 0.0000),
    
                (19, EnhancementMode::Level1)      => (1.0, 0.15, 0.0850),
                (19, EnhancementMode::Level2)      => (2.0, 0.12, 0.0616),
                (19, EnhancementMode::Level3)      => (3.5, 0.10, 0.0360),
                (19, EnhancementMode::Level4)      => (6.5, 0.08, 0.0000),
    
                (20, EnhancementMode::Level1)      => (1.0, 0.30, 0.1050),
                (20, EnhancementMode::Level2)      => (2.0, 0.25, 0.0750),
                (20, EnhancementMode::Level3)      => (3.5, 0.20, 0.0400),
                (20, EnhancementMode::Level4)      => (6.5, 0.15, 0.0000),
    
                (21, EnhancementMode::Level1)      => (1.0, 0.15, 0.1275),
                (21, EnhancementMode::Level2)      => (2.0, 0.12, 0.0880),
                (21, EnhancementMode::Level3)      => (3.5, 0.10, 0.0450),
                (21, EnhancementMode::Level4)      => (6.5, 0.08, 0.0000),
    
                // Standard fallback rates mapped from your original array
                (0, _)  => (1.0, 0.95, 0.0),
                (1, _)  => (1.0, 0.90, 0.0),
                (2, _)  => (1.0, 0.85, 0.0),
                (3, _)  => (1.0, 0.85, 0.0),
                (4, _)  => (1.0, 0.80, 0.0),
                (5, _)  => (1.0, 0.75, 0.0),
                (6, _)  => (1.0, 0.70, 0.0),
                (7, _)  => (1.0, 0.65, 0.0),
                (8, _)  => (1.0, 0.60, 0.0),
                (9, _)  => (1.0, 0.55, 0.0),
                (10, _) => (1.0, 0.50, 0.0),
                (11, _) => (1.0, 0.45, 0.0),
                (12, _) => (1.0, 0.40, 0.0),
                (13, _) => (1.0, 0.35, 0.0),
                (14, _) => (1.0, 0.30, 0.0),
                (15, _) => (1.0, 0.30, 0.021),
                (16, _) => (1.0, 0.30, 0.021),
                (17, _) => (1.0, 0.15, 0.068),
                (18, _) => (1.0, 0.15, 0.068),
                (19, _) => (1.0, 0.15, 0.085),
                (20, _) => (1.0, 0.30, 0.105),
                (21, _) => (1.0, 0.15, 0.1275),
                (22, _) => (1.0, 0.15, 0.17),
                (23, _) => (1.0, 0.10, 0.18),
                (24, _) => (1.0, 0.10, 0.18),
                (25, _) => (1.0, 0.10, 0.18),
                (26, _) => (1.0, 0.07, 0.186),
                (27, _) => (1.0, 0.05, 0.19),
                (28, _) => (1.0, 0.03, 0.194),
                (29, _) => (1.0, 0.01, 0.198),
                _ => (1.0, 0.0, 0.0), // Failsafe for out of bounds
        }
    }
}

pub struct MesoConfig {
    divisor: f64,
    current_star_exp: f64,
    extra_mult: f64,
}

/// Calculates the baseline KMS (Korean MapleStory) meso cost for enhancing an item.
///
/// # Arguments
/// * `current_star` - The current star level of the item before enhancement attempt.
/// * `item_level` - Equipment level (e.g. 150, 160, 200).
///
/// # Formula Rationale
/// Uses specific piecewise formulas depending on the current star rank range
/// (0-9, 10-14, 15-21) to match standard official KMS formulas.
pub fn kms_cost(current_star: u32, item_level: u32) -> u64 {
    let config = match current_star {
        11 => MesoConfig {
            divisor: 22000.0,
            current_star_exp: 2.7,
            extra_mult: 1.0,
        },
        12 => MesoConfig {
            divisor: 15000.0,
            current_star_exp: 2.7,
            extra_mult: 1.0,
        },
        13 => MesoConfig {
            divisor: 11000.0,
            current_star_exp: 2.7,
            extra_mult: 1.0,
        },
        14 => MesoConfig {
            divisor: 7500.0,
            current_star_exp: 2.7,
            extra_mult: 1.0,
        },
        17 => MesoConfig {
            divisor: 20000.0,
            current_star_exp: 2.7,
            extra_mult: 4.0 / 3.0,
        },
        18 => MesoConfig {
            divisor: 20000.0,
            current_star_exp: 2.7,
            extra_mult: 20.0 / 7.0,
        },
        19 => MesoConfig {
            divisor: 20000.0,
            current_star_exp: 2.7,
            extra_mult: 40.0 / 9.0,
        },
        21 => MesoConfig {
            divisor: 20000.0,
            current_star_exp: 2.7,
            extra_mult: 8.0 / 5.0,
        },
        15.. => MesoConfig {
            divisor: 20000.0,
            current_star_exp: 2.7,
            extra_mult: 1.0,
        },
        10.. => MesoConfig {
            divisor: 40000.0,
            current_star_exp: 2.7,
            extra_mult: 1.0,
        },
        _ => MesoConfig {
            divisor: 2500.0,
            current_star_exp: 1.0,
            extra_mult: 1.0,
        },
    };

    let level_factor = ((item_level / 10) * 10) as f64;
    let star_factor = (current_star + 1) as f64;

    let base_calc =
        (config.extra_mult * level_factor.powi(3) * star_factor.powf(config.current_star_exp))
            / config.divisor;

    100 * ((base_calc + 10.0).round() as u64)
}

/// Accumulator for fast Monte Carlo simulation metrics across trial runs.
///
/// Efficiently aggregates cost distributions into fixed-width bins (`BIN_SIZE`) to allow fast
/// percentile queries without requiring dynamic allocation or sorting millions of floats.
pub struct FastMetrics {
    pub cost_histogram: Vec<u32>,
    pub joint_histogram: Vec<[u32; 100]>,
    pub session_booms_histogram: Box<[u32; 100]>,
    pub per_star_friction: Box<[[u64; 3]; 30]>,
    pub total_runs: u32,
    pub total_cost: u128,
    pub total_boom: u64,
}

impl FastMetrics {
    pub fn new() -> Self {
        Self {
            // Preallocate 10,000 bins (Handles up to 1 Trillion meso costs without reallocating)
            cost_histogram: vec![0; 10_000],
            joint_histogram: vec![[0; 100]; 10_000],
            session_booms_histogram: Box::new([0; 100]),
            per_star_friction: Box::new([[0; 3]; 30]),
            total_runs: 0,
            total_cost: 0,
            total_boom: 0,
        }
    }

    /// Records the outcome of a single enhancement simulation run.
    #[inline(always)]
    pub fn record_run(&mut self, cost: u64, booms: u32) {
        self.total_runs += 1;
        self.total_cost += cost as u128;
        self.total_boom += booms as u64;

        let bin = (cost / BIN_SIZE) as usize;
        
        // Dynamically scale up only if the run hits an extreme outlier
        if bin >= self.cost_histogram.len() {
            self.cost_histogram.resize(bin + 1000, 0);
            self.joint_histogram.resize(bin + 1000, [0; 100]);
        }
        
        self.cost_histogram[bin] += 1;

        let boom_idx = (booms as usize).min(99);
        self.session_booms_histogram[boom_idx] += 1;
        self.joint_histogram[bin][boom_idx] += 1;
    }
}

/// Executes a single Monte Carlo simulation run for enhancing an item from `start_stars` to `target_star`.
///
/// Pre-calculated threshold lookup tables (`boom_thresholds`, `success_thresholds`, `cost_lookup`) are passed in
/// to avoid recalculating rates inside the tight simulation loop.
#[inline(always)]
pub fn run_single_sim(
    start_stars: usize,
    target_star: usize,
    rng: &mut SmallRng,
    boom_thresholds: &[u32; 30],

    success_thresholds: &[u32; 30],
    cost_lookup: &[u64; 30],
    metrics: &mut FastMetrics,
) {
    let mut current_star: usize = start_stars;
    let mut total_cost: u64 = 0;
    let mut total_booms: u32 = 0;

    while current_star < target_star && current_star < 30 {
        let attempt_cost = cost_lookup[current_star];

        total_cost += attempt_cost;
        metrics.per_star_friction[current_star][0] += attempt_cost;
        metrics.per_star_friction[current_star][2] += 1;

        let val: u32 = rng.random();

        if val < boom_thresholds[current_star] {
            total_booms += 1;
            metrics.per_star_friction[current_star][1] += 1;
            current_star = match current_star {
                26.. => 20,
                23..=25 => 19,
                21..=22 => 17,
                20 => 15,
                _ => 12,
            };
        } else if val < success_thresholds[current_star] {
            current_star += 1;
        }
        // Strict adherence to logic: failures fall through with no change
    }
    
    // Push results into metrics struct directly
    metrics.record_run(total_cost, total_booms);
}

/// Consolidated simulation results for a starforce enhancement run.
#[derive(Clone, Debug, PartialEq)]
pub struct SimResult {
    pub average_cost: u128,
    pub average_booms: f64,
    pub average_attempts: f64,
    pub median_cost: u128,
    pub median_booms: u32,
    pub percentile_10: u128,
    pub percentile_25: u128,
    pub percentile_75: u128,
    pub percentile_90: u128,
    pub percentile_99: u128,
    pub boom_distribution: Vec<(u32, f64)>,
    pub star_friction: Vec<StarFrictionInfo>,
    pub cdf: Vec<(u128, f64)>,
    pub joint_histogram: Vec<[u32; 100]>,
    pub total_trials: u32,
}

/// Friction and cost analysis per individual star level.
#[derive(Clone, Debug, PartialEq)]
pub struct StarFrictionInfo {
    pub star: usize,
    pub average_cost: u64,
    pub average_booms: f64,
    pub average_attempts: f64,
}

/// Formats raw meso count into human-readable string (e.g., "1.5B", "350M", "10K").
pub fn format_mesos(mesos: u128) -> String {
    if mesos >= 1_000_000_000 {
        let billions = mesos as f64 / 1_000_000_000.0;
        let s = format!("{:.2}", billions);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        format!("{}B", s)
    } else if mesos >= 1_000_000 {
        let millions = mesos as f64 / 1_000_000.0;
        let s = format!("{:.2}", millions);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        format!("{}M", s)
    } else if mesos >= 1_000 {
        let thousands = mesos as f64 / 1_000.0;
        let s = format!("{:.1}", thousands);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        format!("{}K", s)
    } else {
        mesos.to_formatted_string(&Locale::en)
    }
}

/// Formats execution duration into clean format.
pub fn format_duration(dur: Duration) -> String {
    if dur.as_secs() > 0 {
        format!("{:.2}s", dur.as_secs_f64())
    } else {
        format!("{}ms", dur.as_millis())
    }
}

/// Searches the binned cost histogram to find the meso cost corresponding to a target percentile.
pub fn find_percentile(histogram: &[u32], trials: u32, pct: f64) -> u128 {
    let target_count = (trials as f64 * pct).round() as u32;
    let mut current_count = 0;
    for (bin_idx, &count) in histogram.iter().enumerate() {
        current_count += count;
        if current_count >= target_count {
            return bin_idx as u128 * BIN_SIZE as u128;
        }
    }
    histogram.len() as u128 * BIN_SIZE as u128
}

/// Runs full Monte Carlo simulation engine and computes statistical distributions.
pub fn stars_engine(
    start_stars: usize,
    target_stars: usize,
    equipment_level: u32,
    trials: u32,
    config: EnhanceConfig,
) -> SimResult {
    let trials = trials.max(1);
    let stars: [StarProp; 30] = core::array::from_fn(|i| StarProp::new(i as u8, &config));


    let mut boom_thresholds = [0u32; 30];
    let mut success_thresholds = [0u32; 30];
    let mut cost_lookup = [0u64; 30];

    for i in 0..30 {
        boom_thresholds[i] = (stars[i].boom_rate * 4294967296.0).round() as u32;
        success_thresholds[i] =
            ((stars[i].boom_rate + stars[i].success_rate) * 4294967296.0).round() as u32;
        cost_lookup[i] =
            (kms_cost(i as u32, equipment_level) as f64 * stars[i].cost_multiply).round() as u64;
    }

    let mut rng = SmallRng::from_os_rng();
    let mut metrics = FastMetrics::new();

    for _ in 0..trials {
        run_single_sim(
            start_stars,
            target_stars,
            &mut rng,
            &boom_thresholds,
            &success_thresholds,
            &cost_lookup,
            &mut metrics,
        );
    }

    let average_cost = metrics.total_cost / trials as u128;
    let average_booms = metrics.total_boom as f64 / trials as f64;
    
    let mut total_attempts = 0u64;
    for i in start_stars..target_stars {
        if i < 30 {
            total_attempts += metrics.per_star_friction[i][2];
        }
    }
    let average_attempts = total_attempts as f64 / trials as f64;

    let percentile_10 = find_percentile(&metrics.cost_histogram, trials, 0.10);
    let percentile_25 = find_percentile(&metrics.cost_histogram, trials, 0.25);
    let median_cost = find_percentile(&metrics.cost_histogram, trials, 0.50);
    let percentile_75 = find_percentile(&metrics.cost_histogram, trials, 0.75);
    let percentile_90 = find_percentile(&metrics.cost_histogram, trials, 0.90);
    let percentile_99 = find_percentile(&metrics.cost_histogram, trials, 0.99);

    let limit = ((average_booms * 2.0).ceil() as usize).max(3).min(99);
    let mut boom_distribution = Vec::new();
    let mut accumulated_prob = 0.0;
    for booms in 0..limit {
        let count = metrics.session_booms_histogram[booms];
        let prob = count as f64 / trials as f64;
        boom_distribution.push((booms as u32, prob));
        accumulated_prob += prob;
    }
    let remaining_prob = (1.0 - accumulated_prob).max(0.0);
    boom_distribution.push((limit as u32, remaining_prob));

    let mut star_friction = Vec::new();
    for star in start_stars..target_stars {
        if star >= 30 { break; }
        let cost = metrics.per_star_friction[star][0];
        let booms = metrics.per_star_friction[star][1];
        let attempts = metrics.per_star_friction[star][2];
        
        star_friction.push(StarFrictionInfo {
            star,
            average_cost: cost / trials as u64,
            average_booms: booms as f64 / trials as f64,
            average_attempts: attempts as f64 / trials as f64,
        });
    }

    let max_cost = percentile_99;
    let mut cdf = Vec::new();
    cdf.push((0u128, 0.0f64));
    for i in 1..=20 {
        let budget = (max_cost as f64 * (i as f64 / 20.0)) as u128;
        let target_bin = (budget / BIN_SIZE as u128) as usize;
        let mut accumulated_count = 0;
        for bin_idx in 0..=target_bin {
            if bin_idx < metrics.cost_histogram.len() {
                accumulated_count += metrics.cost_histogram[bin_idx];
            }
        }
        let prob = accumulated_count as f64 / trials as f64;
        cdf.push((budget, prob));
    }

    let mut accumulated_booms_count = 0;
    let target_booms_count = (trials as f64 * 0.50).round() as u32;
    let mut median_booms = 0u32;
    for (booms, &count) in metrics.session_booms_histogram.iter().enumerate() {
        accumulated_booms_count += count;
        if accumulated_booms_count >= target_booms_count {
            median_booms = booms as u32;
            break;
        }
    }

    SimResult {
        average_cost,
        average_booms,
        average_attempts,
        median_cost,
        median_booms,
        percentile_10,
        percentile_25,
        percentile_75,
        percentile_90,
        percentile_99,
        boom_distribution,
        star_friction,
        cdf,
        joint_histogram: metrics.joint_histogram,
        total_trials: trials,
    }
}

/// Helper function to simulate a single equipment enhancement configuration.
pub fn simulate_equip(
    start: usize,
    target: usize,
    level: u32,
    safeguard: bool,
    star_catch: bool,
    ssf_cost: bool,
    ssf_boom: bool,
    mode_15_21: [EnhancementMode; 7],
    trials: u32,
) -> (u128, f64, u128, f64) {
    if target <= start {
        return (0, 0.0, 0, 0.0);
    }
    let config = EnhanceConfig {
        mode_15_21,
        star_catch,
        ssf_boom_reduce_event: ssf_boom,
        ssf_cost_reduce_event: ssf_cost,
        safeguard,
    };
    let res = stars_engine(start, target, level, trials, config);
    (res.average_cost, res.average_booms, res.median_cost, res.median_booms as f64)
}

/// Canvas item representing equipment node on the visual flow graph.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasItem {
    pub id: usize,
    pub name: String,
    pub level: u32,
    pub png_url: String,
    pub is_custom: bool,
    pub start_stars: usize,
    pub target_stars: usize,
    pub safeguard: bool,
    pub star_catch: bool,
    pub ssf_cost: bool,
    pub ssf_boom: bool,
    pub mode_15_21: [EnhancementMode; 7],
    pub x: i32,
    pub y: i32,
    pub avg_cost: u128,
    pub avg_booms: f64,
    pub median_cost: u128,
    pub median_booms: f64,
}

/// Propagates target star results along connected canvas equipment transfer chains.
pub fn propagate_canvas_items(items: &mut [CanvasItem], connections: &[(usize, usize)], trials: u32) {
    let mut changed = true;
    let mut iterations = 0;
    while changed && iterations < 100 {
        changed = false;
        for &(src_id, tgt_id) in connections {
            let src_opt = items.iter().find(|x| x.id == src_id).cloned();
            if let Some(src) = src_opt {
                if let Some(tgt) = items.iter_mut().find(|x| x.id == tgt_id) {
                    let level_diff = tgt.level.saturating_sub(src.level);
                    let star_loss = (level_diff / 10) as usize;
                    let expected_start = src.target_stars.saturating_sub(star_loss);
                    
                    if tgt.start_stars != expected_start {
                        tgt.start_stars = expected_start;
                        if tgt.target_stars < expected_start {
                            tgt.target_stars = expected_start;
                        }
                        // Recalculate simulation for target item
                        let (cost, booms, med_cost, med_booms) = simulate_equip(
                            tgt.start_stars,
                            tgt.target_stars,
                            tgt.level,
                            tgt.safeguard,
                            tgt.star_catch,
                            tgt.ssf_cost,
                            tgt.ssf_boom,
                            tgt.mode_15_21,
                            trials,
                        );
                        tgt.avg_cost = cost;
                        tgt.avg_booms = booms;
                        tgt.median_cost = med_cost;
                        tgt.median_booms = med_booms;
                        changed = true;
                    }
                }
            }
        }
        iterations += 1;
    }
}
