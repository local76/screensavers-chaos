use std::time::Duration;
use crate::runner::core::{TerminalCell, screensaver::Screensaver};
use crate::runner::toolkit::sys_info::get_system_info;
use crate::runner::core::logo_block::render_logo_block;
use super::{Chaos, Phase, Star, Particle, ExplosionType};

impl Screensaver for Chaos {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let dt_secs = dt.as_secs_f32();

        // Auto-detect high refresh rates during the startup phase
        if self.time_elapsed < 2.0 && dt_secs > 0.001 {
            if dt_secs < self.target_frame_time - 0.001 {
                self.target_frame_time = dt_secs;
            }
        }

        // Exponential moving average for frame time (alpha = 0.1)
        self.frame_time_ema = self.frame_time_ema * 0.9 + dt_secs.min(0.2) * 0.1;

        let speed_mult = if self.on_battery { 0.65 } else { 1.0 };
        let delta = dt_secs * speed_mult;
        self.phase_timer += delta;
        self.time_elapsed += delta;

        // Adjust quality_scale based on frame time performance vs target
        if self.time_elapsed > 1.5 {
            if self.frame_time_ema > self.target_frame_time * 1.15 {
                self.quality_scale = (self.quality_scale - 0.15 * delta).max(0.20);
            } else if self.frame_time_ema < self.target_frame_time * 1.05 {
                self.quality_scale = (self.quality_scale + 0.04 * delta).min(1.0);
            }
        }

        // OpenRGB unstable phase-based updates
        // Live: high system load = more chaos/explosions, host_bias unique instability
        // Support sys_refresh_timer = -1000.0 to prevent slow sys_info calls during tests.
        if self.sys_refresh_timer >= 0.0 {
            self.sys_refresh_timer += delta;
            if self.sys_refresh_timer >= 1.0 {
                let sys = get_system_info();
                self.mem_pressure = sys.mem_used_pct / 100.0;
                self.cpu_load = (sys.cpu_usage_pct / 100.0).clamp(0.0, 1.0);
                self.on_battery = sys.power_status.contains("Battery");
                self.sys_refresh_timer = 0.0;
            }
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
                        let mut skip_chance = 0.0;
                        if self.on_battery {
                            skip_chance = 0.45;
                        }
                        let scale_pct = self.quality_scale;
                        if scale_pct < 1.0 {
                            skip_chance = 1.0 - (1.0 - skip_chance) * scale_pct;
                        }
                        if self.particle_limit_opt == 0 {
                            skip_chance = 1.0 - (1.0 - skip_chance) * 0.5;
                        }
                        if skip_chance > 0.0 && self.rng.next_bool(skip_chance) {
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

            self.phase = Phase::Assembled;
            self.phase_timer = 0.0;
            self.last_cols = cols;
            self.last_rows = rows;
        }

        // Dynamically adjust star population to match target capacity
        let target_stars = (((cols * rows / 16).clamp(20, 100)) as f32 * self.quality_scale * (if self.on_battery { 0.55 } else { 1.0 })) as usize;
        if self.stars.len() > target_stars {
            self.stars.truncate(target_stars);
        } else if self.stars.len() < target_stars && target_stars > 0 {
            while self.stars.len() < target_stars {
                self.stars.push(Star {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch: if self.stars.len() % 8 == 0 { '✦' } else if self.stars.len() % 3 == 0 { '•' } else { '.' },
                    excitation: 0.0,
                });
            }
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
