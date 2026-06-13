use crate::runner::core::TerminalCell;
use super::{Chaos, ExplosionType, Phase};

pub(crate) fn draw_spike(
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    x: i32,
    y: i32,
    ch: char,
    fade: u8,
    green_mult: f32,
    blue_add: u8,
    accent: (u8, u8, u8),
) {
    if x >= 0 && x < cols as i32 && y >= 0 && y < rows as i32 {
        let cell = &mut grid[y as usize * cols + x as usize];
        if cell.ch == ' ' || cell.ch == ch {
            let fg_r = (fade as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
            let fg_g = ((fade as f32 * green_mult) * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
            let fg_b = (fade.saturating_add(blue_add) as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
            cell.ch = ch;
            cell.fg = (fg_r, fg_g, fg_b);
        }
    }
}

impl Chaos {
    pub(crate) fn draw_stars(&self, grid: &mut [TerminalCell], cols: usize, rows: usize, accent: (u8, u8, u8)) {
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
                            draw_spike(grid, cols, rows, sx as i32 + dx as i32, sy as i32, '─', fade, 0.75, 45, accent);
                            draw_spike(grid, cols, rows, sx as i32 - dx as i32, sy as i32, '─', fade, 0.75, 45, accent);
                        }
                    }

                    // Draw vertical flare
                    let v_len = 5;
                    for dy in 1..v_len {
                        let alpha = (90.0 * flare_intensity).max(20.0) as u8;
                        let fade = alpha.saturating_sub((dy * (80 / v_len)) as u8);
                        if fade > 10 {
                            draw_spike(grid, cols, rows, sx as i32, sy as i32 + dy as i32, '│', fade, 0.75, 30, accent);
                            draw_spike(grid, cols, rows, sx as i32, sy as i32 - dy as i32, '│', fade, 0.75, 30, accent);
                        }
                    }

                    // Draw diagonal starburst spikes
                    let d_len = 3;
                    for d in 1..=d_len {
                        let alpha = (70.0 * flare_intensity).max(15.0) as u8;
                        let fade = alpha.saturating_sub((d * (60 / d_len)) as u8);
                        if fade > 10 {
                            draw_spike(grid, cols, rows, sx as i32 + d as i32, sy as i32 - d as i32, '/', fade, 0.65, 20, accent);
                            draw_spike(grid, cols, rows, sx as i32 - d as i32, sy as i32 + d as i32, '/', fade, 0.65, 20, accent);
                            draw_spike(grid, cols, rows, sx as i32 - d as i32, sy as i32 - d as i32, '\\', fade, 0.65, 20, accent);
                            draw_spike(grid, cols, rows, sx as i32 + d as i32, sy as i32 + d as i32, '\\', fade, 0.65, 20, accent);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn draw_special_effects(
        &self,
        grid: &mut [TerminalCell],
        cols: usize,
        rows: usize,
        max_possible_dist: f32,
        center_x: f32,
        center_y: f32,
    ) {
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
    }
}
