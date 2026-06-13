//! Consolidated chaos screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

use crate::runner::core::LcgRng;
use crate::runner::toolkit::sys_info::get_system_info;

mod types;
mod physics;
mod update_chaos;
mod screensaver_impl;
mod draw;
mod draw_helpers;

#[cfg(test)]
#[path = "chaos_tests.rs"]
mod tests;

pub use types::{Particle, Star, Phase, ExplosionType};

pub struct Chaos {
    pub(crate) rng: LcgRng,
    pub(crate) particles: Vec<Particle>,
    pub(crate) stars: Vec<Star>,
    pub(crate) phase: Phase,
    pub(crate) phase_timer: f32,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,
    pub(crate) explosion_type: ExplosionType,
    pub(crate) black_hole_burst_triggered: bool,
    pub(crate) particle_limit_opt: u32,
    pub(crate) explosion_freq_opt: u32,

    // Live system dynamics
    pub(crate) sys_refresh_timer: f32,
    pub(crate) mem_pressure: f32,
    pub(crate) cpu_load: f32,
    pub(crate) _host_bias: f32,
    pub(super) on_battery: bool,
    pub(super) frame_time_ema: f32,
    pub(super) quality_scale: f32,
    pub(super) target_frame_time: f32,

    pub(crate) time_elapsed: f32,
}

impl Default for Chaos {
    fn default() -> Self {
        Self::new()
    }
}

impl Chaos {
    pub fn new() -> Self {
        let particle_limit_opt: u32 = 1;
        let explosion_freq_opt: u32 = 1;

        let sys = get_system_info();
        let host_bias = sys.hostname.chars().map(|c| c as u32).sum::<u32>() as f32 / 1000.0 % 1.0;
        let on_battery = sys.power_status.contains("Battery");

        Self {
            rng: LcgRng::new(8888),
            particles: Vec::new(),
            stars: Vec::new(),
            phase: Phase::Assembled,
            phase_timer: 0.0,
            last_cols: 0,
            last_rows: 0,
            explosion_type: ExplosionType::Supernova,
            black_hole_burst_triggered: false,
            particle_limit_opt,
            explosion_freq_opt,
            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            cpu_load: (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0),
            _host_bias: host_bias,
            on_battery,
            frame_time_ema: 0.01666667,
            quality_scale: 1.0,
            target_frame_time: 0.01666667,

            time_elapsed: 0.0,
        }
    }
}
