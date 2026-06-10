//! Consolidated chaos screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

use library::core::{LcgRng, TerminalCell};
use std::time::Duration;
use library::core::screensaver::Screensaver;
use library::core::logo_block::render_logo_block;

use library::platform::native::sys_info::get_system_info;



mod types;
mod physics;

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
    pub(crate) host_bias: f32,

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
            cpu_load: 0.4,
            host_bias,

            time_elapsed: 0.0,
        }
    }

    pub fn update_chaos(&mut self, delta: f32, cols: usize, rows: usize) {
        let center_x = cols as f32 / 2.0;
        let center_y = rows as f32 / 2.0;

        // Handle Black Hole secondary burst
        let burst_time = match self.explosion_freq_opt {
            0 => 2.8,
            2 => 0.7,
            _ => 1.4,
        };
        if self.explosion_type == ExplosionType::BlackHole
            && self.phase_timer >= burst_time
            && !self.black_hole_burst_triggered
        {
            self.black_hole_burst_triggered = true;

            for p in &mut self.particles {
                let mut dx = p.x - center_x;
                let mut dy = (p.y - center_y) * 2.2;
                if dx.abs() < 0.1 && dy.abs() < 0.1 {
                    dx = self.rng.next_range(-1.0, 1.0);
                    dy = self.rng.next_range(-1.0, 1.0);
                }
                let dist = (dx*dx + dy*dy).sqrt().max(1.0);
                let speed = self.rng.next_range(35.0, 65.0);
                p.vx = (dx / dist) * speed + self.rng.next_range(-4.0, 4.0);
                p.vy = ((dy / dist) * speed + self.rng.next_range(-4.0, 4.0)) * 0.48;
                p.glow = 1.5;
            }
        }

        for p in &mut self.particles {
            match self.explosion_type {
                ExplosionType::Supernova => {
                    p.x += p.vx * delta;
                    p.y += p.vy * delta;
                    p.vx *= 1.0 - 0.85 * delta;
                    p.vy *= 1.0 - 0.85 * delta;
                }
                ExplosionType::BlackHole => {
                    if !self.black_hole_burst_triggered {
                        let dx = center_x - p.x;
                        let dy = (center_y - p.y) * 2.2;
                        let dist = (dx*dx + dy*dy).sqrt().max(0.5);

                        let pull = 160.0 / dist.max(2.0);
                        let angle = dy.atan2(dx);
                        let spin = 35.0 / dist.max(2.0);

                        p.vx += (pull * angle.cos() - spin * angle.sin()) * delta;
                        p.vy += (pull * angle.sin() + spin * angle.cos()) * delta * 0.48;

                        p.vx *= 1.0 - 0.9 * delta;
                        p.vy *= 1.0 - 0.9 * delta;
                    } else {
                        p.vx *= 1.0 - 0.7 * delta;
                        p.vy *= 1.0 - 0.7 * delta;
                    }
                    p.x += p.vx * delta;
                    p.y += p.vy * delta;
                }
                ExplosionType::Vortex => {
                    let dx = p.x - center_x;
                    let dy = (p.y - center_y) * 2.2;

                    let angle = dy.atan2(dx);
                    let pull_strength = -6.0;
                    let spin_strength = 28.0;

                    p.vx += (pull_strength * angle.cos() - spin_strength * angle.sin()) * delta;
                    p.vy += (pull_strength * angle.sin() + spin_strength * angle.cos()) * delta * 0.48;

                    p.vx *= 1.0 - 0.4 * delta;
                    p.vy *= 1.0 - 0.4 * delta;

                    p.x += p.vx * delta;
                    p.y += p.vy * delta;
                }
                ExplosionType::GlitchWave => {
                    let wave = (self.phase_timer * 18.0 + p.y * 0.4).sin() * 22.0;
                    p.vx += wave * delta;
                    p.vy += self.rng.next_range(-6.0, 6.0) * delta;

                    p.vx *= 1.0 - 1.4 * delta;
                    p.vy *= 1.0 - 1.4 * delta;

                    p.x += p.vx * delta;
                    p.y += p.vy * delta;

                    if self.rng.next_bool(0.08) {
                        let glitch_chars = ['1', '0', '█', '░', '▒', '▞', '*', '$', '#', '@', '&', '%'];
                        p.ch = glitch_chars[self.rng.next_usize(glitch_chars.len())];
                    } else if self.rng.next_bool(0.1) {
                        p.ch = p.orig_ch;
                    }
                }
                ExplosionType::Shockwave => {
                    if self.rng.next_bool(0.06) {
                        p.ch = if self.rng.next_bool(0.5) { '═' } else { '║' };
                    }

                    let dx = p.x - center_x;
                    let dy = (p.y - center_y) * 2.2;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    let angle = dy.atan2(dx);

                    let wave = (self.phase_timer * 12.0 + dist * 0.1).sin() * 8.0;
                    let push = 18.0 / dist.max(4.0);

                    p.vx += (push * angle.cos() + wave * 0.3) * delta;
                    p.vy += (push * angle.sin() * 0.48) * delta;

                    p.vx *= 1.0 - 0.78 * delta;
                    p.vy *= 1.0 - 0.78 * delta;

                    p.x += p.vx * delta;
                    p.y += p.vy * delta;
                }
                ExplosionType::Entropy => {
                    let jitter = 14.0 + (self.phase_timer * 6.0).min(35.0);
                    p.vx += self.rng.next_range(-jitter, jitter) * delta;
                    p.vy += self.rng.next_range(-jitter * 0.4, jitter * 0.4) * delta;

                    let dx = p.x - center_x;
                    let dy = (p.y - center_y) * 2.2;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    let push = 3.0 / dist.max(3.0);
                    let angle = dy.atan2(dx);

                    p.vx += push * angle.cos() * delta;
                    p.vy += push * angle.sin() * 0.48 * delta;

                    p.vx *= 1.0 - 0.65 * delta;
                    p.vy *= 1.0 - 0.65 * delta;

                    p.x += p.vx * delta;
                    p.y += p.vy * delta;

                    if self.rng.next_bool(0.12) {
                        let corrupt_chars = ['#', '@', '%', '&', '░', '▒', '▓', '█', '0', '1'];
                        p.ch = corrupt_chars[self.rng.next_usize(corrupt_chars.len())];
                    }
                }
                ExplosionType::Resonance => {
                    let dx = p.x - center_x;
                    let dy = (p.y - center_y) * 2.2;
                    let angle = dy.atan2(dx);

                    let oscillation = (self.phase_timer * 22.0).sin() * 18.0;
                    let base_pull = 5.0;

                    p.vx += (base_pull * angle.cos() + oscillation * angle.cos() * 0.6) * delta;
                    p.vy += (base_pull * angle.sin() * 0.48 + oscillation * angle.sin() * 0.48 * 0.6) * delta;

                    p.vx *= 1.0 - 0.55 * delta;
                    p.vy *= 1.0 - 0.55 * delta;

                    p.x += p.vx * delta;
                    p.y += p.vy * delta;
                }
            }

            let bounce_loss = 0.72;
            if p.x < 0.0 {
                p.x = 0.0;
                p.vx = -p.vx * bounce_loss;
            } else if p.x >= cols as f32 {
                p.x = cols.saturating_sub(1) as f32;
                p.vx = -p.vx * bounce_loss;
            }

            if p.y < 0.0 {
                p.y = 0.0;
                p.vy = -p.vy * bounce_loss;
            } else if p.y >= rows as f32 {
                p.y = rows.saturating_sub(1) as f32;
                p.vy = -p.vy * bounce_loss;
            }

            if p.glow > 0.1 {
                p.glow -= delta * 0.15;
            }
        }

        let limit = match self.explosion_freq_opt {
            0 => 10.0,
            2 => 2.5,
            _ => 5.0,
        };
        if self.phase_timer > limit {
            self.phase = Phase::SnapBack;
            self.phase_timer = 0.0;
        }
    }
}

impl Screensaver for Chaos {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let delta = dt.as_secs_f32();
        self.phase_timer += delta;
        self.time_elapsed += delta;

        // OpenRGB unstable phase-based updates
// Live: high system load = more chaos/explosions, host_bias unique instability
        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (self.mem_pressure * 0.6 + 0.3).min(0.9);
            if self.host_bias > 0.6 { self.cpu_load = (self.cpu_load + 0.1).min(0.98); }
            self.sys_refresh_timer = 0.0;
        }

        // Reinitialize if screen size changed
        if cols != self.last_cols || rows != self.last_rows {
            self.particles.clear();
            self.stars.clear();
            let logo_text = get_system_info().logo_text;
            let lines = render_logo_block(&logo_text, None);
            let logo_h = lines.len();
            let logo_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
            let logo_x = cols.saturating_sub(logo_w) / 2;
            let logo_y = rows.saturating_sub(logo_h) / 2;

            for (r_offset, line) in lines.iter().enumerate().take(logo_h) {
                for (c_offset, ch) in line.chars().enumerate() {
                    if ch != ' ' {
                        if self.particle_limit_opt == 0 && self.rng.next_bool(0.5) {
                            continue;
                        }
                        let hx = (logo_x + c_offset) as f32;
                        let hy = (logo_y + r_offset) as f32;
                        self.particles.push(Particle {
                            home_x: hx,
                            home_y: hy,
                            x: hx,
                            y: hy,
                            vx: 0.0,
                            vy: 0.0,
                            ch,
                            orig_ch: ch,
                            glow: 0.0,
                            snapped: true,
                        });
                    }
                }
            }

            // Create background stars
            let target_stars = (cols * rows / 16).clamp(20, 100);
            for i in 0..target_stars {
                self.stars.push(Star {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch: if i % 8 == 0 { '✦' } else if i % 3 == 0 { '•' } else { '.' },
                    excitation: 0.0,
                });
            }

            self.phase = Phase::Assembled;
            self.phase_timer = 0.0;
            self.last_cols = cols;
            self.last_rows = rows;
        }

        // Decay star excitations
        for star in &mut self.stars {
            if star.excitation > 0.0 {
                star.excitation -= delta * 2.5;
                if star.excitation < 0.0 {
                    star.excitation = 0.0;
                }
            }
        }

        // Particle-star proximity interaction: unsnapped particles excite nearby stars
        let cols_f = cols as f32;
        let rows_f = rows as f32;
        let star_excite_mult = match self.explosion_type {
            ExplosionType::Shockwave => 2.2,
            ExplosionType::Entropy => 1.8,
            ExplosionType::Resonance => 1.4,
            ExplosionType::BlackHole => 0.7,
            _ => 1.5,
        };
        for p in &self.particles {
            if !p.snapped {
                for star in &mut self.stars {
                    let sx = star.x * cols_f;
                    let sy = star.y * rows_f;
                    let dx = p.x - sx;
                    let dy = (p.y - sy) * 2.0;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq < 9.0 {
                        let dist = dist_sq.sqrt();
                        let force = (1.0 - dist / 3.0) * 1.5 * star_excite_mult;
                        star.excitation = star.excitation.max(force);
                    }
                }
            }
        }

        // Particle dynamics update based on phase
        match self.phase {
            Phase::Assembled => {
                self.update_assembled(delta);
            }
            Phase::Exploding => {
                self.update_exploding(cols, rows);
            }
            Phase::Chaos => {
                self.update_chaos(delta, cols, rows);
            }
            Phase::SnapBack => {
                self.update_snapback(delta);
            }
        }
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        self.draw_impl(grid, cols, rows);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_new() {
        let chaos = Chaos::new();
        assert_eq!(chaos.phase, Phase::Assembled);
        assert_eq!(chaos.particles.len(), 0);
    }

    #[test]
    fn test_chaos_update_and_draw() {
        let mut chaos = Chaos::new();
        chaos.update(Duration::from_millis(16), 80, 24);
        let mut grid = vec![TerminalCell::default(); 80 * 24];
        chaos.draw(&mut grid, 80, 24);
        // Should initialize and have stars/particles set up
        assert!(!chaos.stars.is_empty());
        assert!(!chaos.particles.is_empty());
    }
}

