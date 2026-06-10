//! Consolidated chaos screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).


use library::core::{LcgRng, TerminalCell};
use std::time::Duration;
use library::core::screensaver::Screensaver;
use library::core::logo_block::render_logo_block;

use library::platform::native::sys_info::get_system_info;

use library::toolkit::rgb_controller::{RgbController, is_openrgb_enabled};

use library::toolkit::rgb_protocol::RgbColor;
use library::toolkit::sys_info::query_current_palette;

pub struct Particle {
    pub home_x: f32,
    pub home_y: f32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ch: char,
    pub orig_ch: char,
    pub glow: f32,
    pub snapped: bool,
}

pub struct Star {
    pub x: f32,
    pub y: f32,
    pub phase: f32,
    pub ch: char,
    pub excitation: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Assembled,
    Exploding,
    Chaos,
    SnapBack,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExplosionType {
    Supernova,
    BlackHole,
    Vortex,
    GlitchWave,
    Shockwave,
    Entropy,
    Resonance,
}

impl ExplosionType {
    pub fn next(self) -> Self {
        match self {
            ExplosionType::Supernova => ExplosionType::BlackHole,
            ExplosionType::BlackHole => ExplosionType::Vortex,
            ExplosionType::Vortex => ExplosionType::GlitchWave,
            ExplosionType::GlitchWave => ExplosionType::Shockwave,
            ExplosionType::Shockwave => ExplosionType::Entropy,
            ExplosionType::Entropy => ExplosionType::Resonance,
            ExplosionType::Resonance => ExplosionType::Supernova,
        }
    }
}

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
    pub(crate) rgb: Option<RgbController>,
    pub(crate) rgb_timer: f32,
    pub(crate) last_phase: Option<Phase>,
    pub(crate) time_elapsed: f32,
}

impl Default for Chaos {
    fn default() -> Self {
        Self::new()
    }
}

impl Chaos {
    pub fn new() -> Self {
        // Pre-4.1 HKEY_CURRENT_USER registry reads (ParticleLimit, ExplosionFreq)
        // collapsed to defaults for the inline migration. Re-added in 4.2.
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
            rgb: if is_openrgb_enabled() { Some(RgbController::new()) } else { None },
            rgb_timer: 0.0,
            last_phase: None,
            time_elapsed: 0.0,
        }
    }

    pub fn update_assembled(&mut self, delta: f32) {
        for p in &mut self.particles {
            p.x = p.home_x;
            p.y = p.home_y;
            p.vx = 0.0;
            p.vy = 0.0;
            p.snapped = true;
            p.ch = p.orig_ch;
            if p.glow > 0.0 {
                p.glow -= delta * 1.5;
            }
        }

        // Live load = shorter time to next explosion/chaos
        let load_mult = 1.0 + self.cpu_load * 0.8 + self.mem_pressure * 0.4;
        let limit = (match self.explosion_freq_opt {
            0 => 10.0,
            2 => 2.5,
            _ => 5.0,
        } / load_mult).max(1.0);
        if self.phase_timer > limit {
            self.phase = Phase::Exploding;
            self.phase_timer = 0.0;
        }
    }

    pub fn update_exploding(&mut self, cols: usize, rows: usize) {
        self.explosion_type = self.explosion_type.next();
        self.black_hole_burst_triggered = false;

        let center_x = cols as f32 / 2.0;
        let center_y = rows as f32 / 2.0;

        for p in &mut self.particles {
            let mut dx = p.x - center_x;
            let mut dy = (p.y - center_y) * 2.2; // aspect ratio scaling

            if dx.abs() < 0.1 && dy.abs() < 0.1 {
                dx = self.rng.next_range(-1.0, 1.0);
                dy = self.rng.next_range(-1.0, 1.0);
            }

            match self.explosion_type {
                ExplosionType::Supernova => {
                    let angle = dy.atan2(dx);
                    let disp = self.rng.next_range(-0.4, 0.4);
                    let speed = self.rng.next_range(20.0, 42.0);

                    p.vx = speed * (angle + disp).cos();
                    p.vy = speed * (angle + disp).sin() * 0.48;
                    p.glow = 1.0;
                }
                ExplosionType::BlackHole => {
                    // Implode: pull inward towards center
                    let angle = dy.atan2(dx);
                    let disp = self.rng.next_range(-0.1, 0.1);
                    let speed = self.rng.next_range(12.0, 24.0);

                    p.vx = -speed * (angle + disp).cos();
                    p.vy = -speed * (angle + disp).sin() * 0.48;
                    p.glow = 0.8;
                }
                ExplosionType::Vortex => {
                    let angle = dy.atan2(dx);
                    let speed = self.rng.next_range(22.0, 38.0);
                    let spin_speed = speed;
                    let radial_speed = 6.0;

                    p.vx = radial_speed * angle.cos() - spin_speed * angle.sin();
                    p.vy = (radial_speed * angle.sin() + spin_speed * angle.cos()) * 0.48;
                    p.glow = 1.0;
                }
                ExplosionType::GlitchWave => {
                    p.vx = self.rng.next_range(-40.0, 40.0);
                    p.vy = self.rng.next_range(-2.5, 2.5);
                    p.glow = 1.0;
                }
                ExplosionType::Shockwave => {
                    // Strong expanding ring / pressure wave
                    let angle = dy.atan2(dx);
                    let disp = self.rng.next_range(-0.2, 0.2);
                    let speed = self.rng.next_range(28.0, 55.0);

                    p.vx = speed * (angle + disp).cos();
                    p.vy = speed * (angle + disp).sin() * 0.48;
                    p.glow = 1.2;
                }
                ExplosionType::Entropy => {
                    // Slow, messy outward drift + jitter
                    let angle = dy.atan2(dx);
                    let speed = self.rng.next_range(8.0, 18.0);
                    let jitter = self.rng.next_range(-12.0, 12.0);

                    p.vx = speed * angle.cos() + jitter;
                    p.vy = speed * angle.sin() * 0.48 + jitter * 0.4;
                    p.glow = 0.7;
                }
                ExplosionType::Resonance => {
                    // Oscillating / vibrating along radial lines
                    let angle = dy.atan2(dx);
                    let speed = self.rng.next_range(15.0, 28.0);

                    p.vx = speed * angle.cos();
                    p.vy = speed * angle.sin() * 0.48;
                    p.glow = 0.9;
                }
            }
            p.snapped = false;
        }

        self.phase = Phase::Chaos;
        self.phase_timer = 0.0;
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

    pub fn update_snapback(&mut self, delta: f32) {
        let mut all_snapped = true;

        for p in &mut self.particles {
            p.ch = p.orig_ch;

            if p.snapped {
                p.x = p.home_x;
                p.y = p.home_y;
                p.vx = 0.0;
                p.vy = 0.0;
                if p.glow > 0.0 {
                    p.glow -= delta * 1.5;
                }
                continue;
            }

            all_snapped = false;

            let dx = p.home_x - p.x;
            let dy = p.home_y - p.y;
            let dist = (dx*dx + dy*dy).sqrt();

            if dist < 0.5 {
                p.x = p.home_x;
                p.y = p.home_y;
                p.vx = 0.0;
                p.vy = 0.0;
                p.glow = 1.5;
                p.snapped = true;
            } else {
                let spring_strength = 20.0;
                p.vx += dx * spring_strength * delta;
                p.vy += dy * spring_strength * delta;

                let damping = 4.2;
                p.vx *= 1.0 - damping * delta;
                p.vy *= 1.0 - damping * delta;

                p.x += p.vx * delta;
                p.y += p.vy * delta;
            }
        }

        if all_snapped {
            self.phase = Phase::Assembled;
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
        self.rgb_timer += delta;
        if self.rgb_timer >= 0.08 {
            self.rgb_timer = 0.0;
            if let Some(ref r) = self.rgb {
                // library 4.0: pull from the canonical ScreenPalette.
                let accent = library::toolkit::sys_info::query_current_palette().accent;
                
                if Some(self.phase) != self.last_phase {
                    self.last_phase = Some(self.phase);
                    if self.phase == Phase::Exploding {
                        let (flash_c, dur_ms) = match self.explosion_type {
                            ExplosionType::Supernova => (RgbColor::new(255, 255, 220), 500),
                            ExplosionType::BlackHole => (RgbColor::new(120, 0, 255), 600),
                            ExplosionType::Vortex => (RgbColor::new(0, 180, 255), 450),
                            ExplosionType::GlitchWave => (RgbColor::new(255, 0, 0), 300),
                            ExplosionType::Shockwave => (RgbColor::new(0, 255, 180), 400),
                            ExplosionType::Entropy => (RgbColor::new(255, 128, 0), 500),
                            ExplosionType::Resonance => (RgbColor::new(255, 0, 128), 450),
                        };
                        r.flash(flash_c, Duration::from_millis(dur_ms));
                    }
                }
                
                // Set baseline color in non-exploding phases
                if self.phase != Phase::Exploding {
                    match self.phase {
                        Phase::Assembled => {
                            r.set_color(RgbColor::new(accent.0 / 2, accent.1 / 2, accent.2 / 2));
                        }
                        Phase::Chaos => {
                            // Glitchy orange/red flicker
                            let r_val = self.rng.next_range(100.0, 220.0) as u8;
                            let g_val = self.rng.next_range(20.0, 100.0) as u8;
                            r.set_color(RgbColor::new(r_val, g_val, 0));
                        }
                        Phase::SnapBack => {
                            // Smoothly blend from red/orange back to theme accent color
                            let progress = (self.phase_timer / 1.5).clamp(0.0, 1.0);
                            let red = (accent.0 as f32 * progress + 150.0 * (1.0 - progress)) as u8;
                            let green = (accent.1 as f32 * progress + 50.0 * (1.0 - progress)) as u8;
                            let blue = (accent.2 as f32 * progress) as u8;
                            r.set_color(RgbColor::new(red / 2, green / 2, blue / 2));
                        }
                        Phase::Exploding => {}
                    }
                }
            }
        }

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
            // library 4.1: render the centered system logo from the live OS info
            // (replaces pre-4.1 `trance_core::logo_lines()`).
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


impl Chaos {
    pub fn draw_impl(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        // library 4.0: pull the accent per-frame from the canonical
        // ScreenPalette.
        let accent = query_current_palette().accent;

        // Find top candidates for lens flares (only highly excited stars, max 4)
        let mut flare_candidates: Vec<(usize, f32)> = self.stars.iter()
            .enumerate()
            .filter(|(_, star)| star.excitation > 0.8)
            .map(|(idx, star)| (idx, star.excitation))
            .collect();
        flare_candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
        let allowed_flares: Vec<usize> = flare_candidates.iter()
            .take(4)
            .map(|&(idx, _)| idx)
            .collect();

        // 2. Draw background stars and their cinematic lens flares/starbursts
        for (i, star) in self.stars.iter().enumerate() {
            let sx = (star.x * cols as f32) as usize;
            let sy = (star.y * rows as f32) as usize;

            if sx < cols && sy < rows {
                // Sparkle value is augmented by excitation!
                let sparkle_base = ((self.phase_timer * 2.0 + star.phase).sin() + 1.0) * 0.5;
                let sparkle = (sparkle_base + star.excitation).min(2.0);
                
                let mut r = (50.0 + sparkle * 80.0) as u8;
                let mut g = (50.0 + sparkle * 80.0) as u8;
                let mut b = (65.0 + sparkle * 75.0) as u8;

                // Blend with accent color when excited
                if star.excitation > 0.1 {
                    let blend = (star.excitation * 0.5).min(1.0);
                    r = (r as f32 * (1.0 - blend) + accent.0 as f32 * blend).min(255.0) as u8;
                    g = (g as f32 * (1.0 - blend) + accent.1 as f32 * blend).min(255.0) as u8;
                    b = (b as f32 * (1.0 - blend) + accent.2 as f32 * blend).min(255.0) as u8;
                }

                let ch = if sparkle > 1.2 {
                    '✹'
                } else if sparkle > 0.6 {
                    '✦'
                } else {
                    star.ch
                };

                grid[sy * cols + sx] = TerminalCell {
                    ch,
                    fg: (r, g, b),
                    bg: grid[sy * cols + sx].bg,
                    bold: sparkle > 0.6 || star.excitation > 0.3,
                };

                // 2b. Draw lens flares and starbursts on highly excited stars
                let is_excited = allowed_flares.contains(&i);
                if is_excited {
                    let flare_intensity = ((star.excitation - 0.8) / 0.7 + 0.5).min(1.5);
                    
                    // Draw horizontal flare (cinematic anamorphic streak, longer if excited)
                    let h_len = 12;
                    for dx in 1..h_len {
                        let alpha = (120.0 * flare_intensity).max(30.0) as u8;
                        let fade = alpha.saturating_sub((dx * (110 / h_len)) as u8);

                        if fade > 10 {
                            if sx + dx < cols {
                                let cell = &mut grid[sy * cols + (sx + dx)];
                                if cell.ch == ' ' || cell.ch == '─' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.75) as u8;
                                    let mut fg_b = fade.saturating_add(45);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '─';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sx >= dx {
                                let cell = &mut grid[sy * cols + (sx - dx)];
                                if cell.ch == ' ' || cell.ch == '─' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.75) as u8;
                                    let mut fg_b = fade.saturating_add(45);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '─';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                        }
                    }

                    // Draw vertical flare
                    let v_len = 5;
                    for dy in 1..v_len {
                        let alpha = (90.0 * flare_intensity).max(20.0) as u8;
                        let fade = alpha.saturating_sub((dy * (80 / v_len)) as u8);

                        if fade > 10 {
                            if sy + dy < rows {
                                let cell = &mut grid[(sy + dy) * cols + sx];
                                if cell.ch == ' ' || cell.ch == '│' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.75) as u8;
                                    let mut fg_b = fade.saturating_add(30);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '│';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sy >= dy {
                                let cell = &mut grid[(sy - dy) * cols + sx];
                                if cell.ch == ' ' || cell.ch == '│' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.75) as u8;
                                    let mut fg_b = fade.saturating_add(30);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '│';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                        }
                    }

                    // Draw diagonal starburst spikes
                    let d_len = 3;
                    for d in 1..=d_len {
                        let alpha = (70.0 * flare_intensity).max(15.0) as u8;
                        let fade = alpha.saturating_sub((d * (60 / d_len)) as u8);
                        if fade > 10 {
                            if sx + d < cols && sy >= d {
                                let cell = &mut grid[(sy - d) * cols + (sx + d)];
                                if cell.ch == ' ' || cell.ch == '/' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.65) as u8;
                                    let mut fg_b = fade.saturating_add(20);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '/';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sx >= d && sy + d < rows {
                                let cell = &mut grid[(sy + d) * cols + (sx - d)];
                                if cell.ch == ' ' || cell.ch == '/' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.65) as u8;
                                    let mut fg_b = fade.saturating_add(20);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '/';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sx >= d && sy >= d {
                                let cell = &mut grid[(sy - d) * cols + (sx - d)];
                                if cell.ch == ' ' || cell.ch == '\\' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.65) as u8;
                                    let mut fg_b = fade.saturating_add(20);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '\\';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sx + d < cols && sy + d < rows {
                                let cell = &mut grid[(sy + d) * cols + (sx + d)];
                                if cell.ch == ' ' || cell.ch == '\\' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.65) as u8;
                                    let mut fg_b = fade.saturating_add(20);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '\\';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Draw particles + special side effects per explosion type
        let center_x = cols as f32 / 2.0;
        let center_y = rows as f32 / 2.0;
        let max_possible_dist = (center_x*center_x + center_y*center_y).sqrt().max(1.0);

        // Special side effects (visual flair only in Chaos phase)
        if self.phase == Phase::Chaos {
            match self.explosion_type {
                ExplosionType::Shockwave => {
                    // Expanding shock ring (deterministic pattern using phase_timer)
                    let ring_radius = ((self.phase_timer * 28.0) % (max_possible_dist * 1.2)) as i32;
                    let ring_thickness = 2;
                    for r in (ring_radius - ring_thickness)..=(ring_radius + ring_thickness) {
                        if r < 2 { continue; }
                        for angle_step in 0..36 {
                            let angle = (angle_step as f32) * 10.0 * std::f32::consts::PI / 180.0;
                            let rx = (center_x + r as f32 * angle.cos()).round() as i32;
                            let ry = (center_y + r as f32 * angle.sin() * 0.48).round() as i32; // aspect
                            if rx >= 0 && rx < cols as i32 && ry >= 0 && ry < rows as i32 {
                                let idx = (ry as usize) * cols + (rx as usize);
                                let cell = &mut grid[idx];
                                if cell.ch == ' ' || cell.ch == '.' || cell.ch == '•' {
                                    // Deterministic choice
                                    let use_block = ((r + angle_step) % 3) == 0;
                                    cell.ch = if use_block { '▓' } else { '░' };
                                    let intensity = (180.0 + (r as f32 - ring_radius as f32).abs() * 20.0).min(255.0) as u8;
                                    cell.fg = (intensity, (intensity as f32 * 0.7) as u8, intensity.saturating_sub(30));
                                    cell.bold = true;
                                }
                            }
                        }
                    }
                }
                ExplosionType::Entropy => {
                    // Data rot: deterministically corrupt background cells near unsnapped particles (no &mut rng in &self draw)
                    for p in &self.particles {
                        if !p.snapped {
                            let px = p.x.round() as i32;
                            let py = p.y.round() as i32;
                            // Use phase_timer + position for deterministic "randomness"
                            let seed = ((self.phase_timer * 17.0 + px as f32 * 0.7 + py as f32) as i32) % 17;
                            if seed % 7 < 2 {
                                for d in 0..3 {
                                    let ox = (seed + d) % 7 - 3;
                                    let oy = (seed * 3 + d) % 5 - 2;
                                    let rx = px + ox;
                                    let ry = py + oy;
                                    if rx >= 0 && rx < cols as i32 && ry >= 0 && ry < rows as i32 {
                                        let idx = (ry as usize) * cols + (rx as usize);
                                        let cell = &mut grid[idx];
                                        if cell.ch == ' ' || cell.ch == '.' {
                                            cell.ch = ['░', '▒', '▓', '?', '#'][((seed + d) as usize) % 5];
                                            cell.fg = (80, 60, 40);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ExplosionType::Resonance => {
                    // Resonance hum: modulate brightness of nearby static grid cells
                    let hum = ((self.phase_timer * 25.0).sin() * 0.5 + 0.5).min(1.0);
                    for p in &self.particles {
                        if p.snapped {
                            let px = p.x.round() as i32;
                            let py = p.y.round() as i32;
                            if px >= 0 && px < cols as i32 && py >= 0 && py < rows as i32 {
                                let idx = (py as usize) * cols + (px as usize);
                                let cell = &mut grid[idx];
                                if cell.ch != ' ' {
                                    let boost = (hum * 40.0) as u8;
                                    cell.fg = (
                                        cell.fg.0.saturating_add(boost),
                                        cell.fg.1.saturating_add(boost / 2),
                                        cell.fg.2.saturating_add(boost / 3),
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let is_aberration = match self.phase {
            Phase::Exploding => self.phase_timer < 0.6,
            Phase::SnapBack => true,
            Phase::Assembled => (self.phase_timer % 4.0) < 0.18,
            _ => false,
        };

        let shift = if is_aberration {
            match self.phase {
                Phase::Exploding => (((0.6 - self.phase_timer) * 5.0) as i32).max(1),
                Phase::Assembled if (self.phase_timer * 20.0) as i32 % 2 == 0 => 2,
                _ => 1,
            }
        } else {
            0
        };

        for p in &self.particles {
            let px = p.x.round() as i32;
            let py = p.y.round() as i32;

            if px >= 0 && px < cols as i32 && py >= 0 && py < rows as i32 {
                let dx = p.x - center_x;
                let dy = p.y - center_y;
                let dist = (dx*dx + dy*dy).sqrt();

                let color = if self.phase == Phase::Assembled {
                    let glow_factor = p.glow.min(1.5);
                    if glow_factor > 1.0 {
                        let extra = ((glow_factor - 1.0) * 2.0 * 255.0).min(255.0) as u8;
                        let r = (extra.max(accent.0)).max(160);
                        let g = (extra.max(accent.1)).max(160);
                        let b = (extra.max(accent.2)).max(160);
                        (r, g, b)
                    } else {
                        let r = (accent.0 as f32 * (0.6 + 0.4 * glow_factor)).min(255.0) as u8;
                        let g = (accent.1 as f32 * (0.6 + 0.4 * glow_factor)).min(255.0) as u8;
                        let b = (accent.2 as f32 * (0.6 + 0.4 * glow_factor)).min(255.0) as u8;
                        (r, g, b)
                    }
                } else {
                    let ratio = (dist / max_possible_dist).min(1.0);
                    let r = (255.0 * ratio + (accent.0 as f32) * (1.0 - ratio)) as u8;
                    let g = (110.0 * ratio + (accent.1 as f32) * (1.0 - ratio)) as u8;
                    let b = ((accent.2 as f32) * (1.0 - ratio)) as u8;
                    (r, g, b)
                };

                let idx = py as usize * cols + px as usize;

                // Draw chromatic splits (red left, blue right)
                if shift > 0 {
                    let rx = px - shift;
                    if rx >= 0 && rx < cols as i32 {
                        let r_idx = py as usize * cols + rx as usize;
                        grid[r_idx] = TerminalCell {
                            ch: p.ch,
                            fg: (230, 10, 50),
                            bg: grid[r_idx].bg,
                            bold: false,
                        };
                    }
                    let bx = px + shift;
                    if bx >= 0 && bx < cols as i32 {
                        let b_idx = py as usize * cols + bx as usize;
                        grid[b_idx] = TerminalCell {
                            ch: p.ch,
                            fg: (0, 120, 255),
                            bg: grid[b_idx].bg,
                            bold: false,
                        };
                    }
                }

                // Main particle (shifts to neon green during RGB glitch splits)
                grid[idx] = TerminalCell {
                    ch: p.ch,
                    fg: if shift > 0 { (10, 230, 80) } else { color },
                    bg: grid[idx].bg,
                    bold: self.phase == Phase::Assembled || p.glow > 0.8,
                };
            }
        }
    }
}
