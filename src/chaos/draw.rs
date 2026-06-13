use crate::runner::core::TerminalCell;
use crate::runner::toolkit::sys_info::query_current_palette;
use super::{Chaos, Phase};

impl Chaos {
    pub fn draw_impl(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        // library 4.0: pull the accent per-frame from the canonical ScreenPalette.
        let accent = query_current_palette().accent;

        // Draw background stars
        self.draw_stars(grid, cols, rows, accent);

        let center_x = cols as f32 / 2.0;
        let center_y = rows as f32 / 2.0;
        let max_possible_dist = (center_x * center_x + center_y * center_y).sqrt().max(1.0);

        // Draw special side effects per explosion type
        self.draw_special_effects(grid, cols, rows, max_possible_dist, center_x, center_y);

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

        let inv_max_possible_dist = 1.0 / max_possible_dist;

        for p in &self.particles {
            let px = p.x.round() as i32;
            let py = p.y.round() as i32;

            if px >= 0 && px < cols as i32 && py >= 0 && py < rows as i32 {
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
                    let dx = p.x - center_x;
                    let dy = p.y - center_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let ratio = (dist * inv_max_possible_dist).min(1.0);
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
