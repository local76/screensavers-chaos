use super::{Chaos, Phase, ExplosionType};

impl Chaos {
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
            let dist = (dx * dx + dy * dy).sqrt();

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
