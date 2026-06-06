mod style;

use crate::render::TerminalRenderer;
use crate::scene::{Scene, SceneContext, SceneLayout};
use std::io;
use style::SantaCruzStyle;

pub struct SantaCruzScene {
    width: u16,
    height: u16,
}

impl SantaCruzScene {
    pub const GROUND_HEIGHT: u16 = 7;

    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Render the Giant Dipper track structure.
    /// Returns the list of (x, y) waypoints for the coaster car animation.
    pub fn render_track(
        renderer: &mut TerminalRenderer,
        center_x: u16,
        ground_y: u16,
        style: &SantaCruzStyle,
    ) -> io::Result<Vec<(u16, u16)>> {
        // Track is drawn relative to an anchor point (right side, near ground)
        // The Giant Dipper profile: lift hill on left, big drop, smaller hills to right
        //
        //        _____
        //       /     \
        //  ____/       \              ___
        // /              \           /   \
        //                 \___/\___/       \___
        //
        // We draw this as a series of characters and collect waypoints.

        let ax = center_x.saturating_sub(30); // anchor left x
        let base_y = ground_y.saturating_sub(2); // just above ground

        // Track profile: (relative_x, relative_y_up) — y_up = chars above base_y
        // Positive y_up means higher on screen (smaller terminal row number)
        #[rustfmt::skip]
        let profile: &[(i16, i16)] = &[
            // Lift hill
            (0,  0), (2,  0), (4,  1), (6,  2), (8,  3), (10, 4),
            (12, 6), (14, 8), (16, 10), (18, 11), (20, 11),
            // First big drop
            (22, 10), (24, 8), (26, 6), (28, 4), (30, 2),
            // Valley and second rise
            (32, 1), (34, 2), (36, 4), (38, 5),
            // Second drop
            (40, 4), (42, 3), (44, 2),
            // Small rise
            (46, 3), (48, 3),
            // Final descent
            (50, 2), (52, 1), (54, 0), (56, 0),
        ];

        // Draw track line segments between profile points
        for window in profile.windows(2) {
            let (x1, yu1) = window[0];
            let (x2, yu2) = window[1];
            let tx1 = ax as i16 + x1;
            let tx2 = ax as i16 + x2;
            let ty1 = base_y as i16 - yu1;
            let ty2 = base_y as i16 - yu2;

            if tx1 < 0 || tx2 < 0 { continue; }

            // Draw a character at each x between tx1 and tx2
            let dx = tx2 - tx1;
            let dy = ty2 - ty1;
            let steps = dx.abs().max(1);
            for step in 0..=steps {
                let px = tx1 + (dx * step) / steps;
                let py = ty1 + (dy * step) / steps;
                if px >= 0 && py >= 0 {
                    let ch = if dy < 0 { '/' } else if dy > 0 { '\\' } else { '_' };
                    renderer.render_char(px as u16, py as u16, ch, style.track)?;
                }
            }
        }

        // Draw vertical supports
        for &(rx, yu) in profile.iter() {
            let tx = ax as i16 + rx;
            if tx < 0 { continue; }
            // Support pillars every 4 units
            if rx % 8 == 0 && yu > 1 {
                for sy in 0..yu {
                    let py = base_y as i16 - sy;
                    if py >= 0 {
                        renderer.render_char(tx as u16, py as u16, '|', style.track_support)?;
                    }
                }
            }
        }

        // Draw boardwalk base
        for col in ax..ax + 60 {
            if col < renderer_width_sentinel() {
                renderer.render_char(col, base_y + 1, '=', style.boardwalk)?;
            }
        }

        // Collect waypoints (absolute terminal coordinates)
        let waypoints: Vec<(u16, u16)> = profile
            .iter()
            .map(|&(rx, yu)| {
                let tx = (ax as i16 + rx).max(0) as u16;
                let ty = (base_y as i16 - yu).max(0) as u16;
                (tx, ty)
            })
            .collect();

        Ok(waypoints)
    }

    fn render_ground(
        renderer: &mut TerminalRenderer,
        width: u16,
        ground_y: u16,
        style: &SantaCruzStyle,
    ) -> io::Result<()> {
        let w = width as usize;
        for row in 0..Self::GROUND_HEIGHT {
            for col in 0..w {
                let (ch, color) = if row == 0 {
                    // Boardwalk planks
                    if col % 4 == 0 { ('|', style.boardwalk) } else { ('_', style.boardwalk) }
                } else if row == 1 {
                    // Top of sand / wet sand
                    if col % 6 < 3 { ('~', style.sand_wet) } else { ('_', style.sand_wet) }
                } else {
                    let r = ((col as u32 ^ 0xABCD).wrapping_mul(row as u32 ^ 0x1234)) % 100;
                    let ch = if r < 5 { '.' } else if r < 10 { ',' } else { ' ' };
                    (ch, style.sand)
                };
                renderer.render_char(col as u16, ground_y + row, ch, color)?;
            }
        }
        Ok(())
    }

    fn render_sign(
        renderer: &mut TerminalRenderer,
        x: u16,
        y: u16,
        style: &SantaCruzStyle,
    ) -> io::Result<()> {
        let lines = [
            ".-----------.",
            "| SANTA CRUZ|",
            "| BOARDWALK |",
            "'-----------'",
        ];
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    renderer.render_char(x + col as u16, y + row as u16, ch, style.sign)?;
                }
            }
        }
        Ok(())
    }
}

// Sentinel used only to avoid rendering past screen — actual width used by caller
fn renderer_width_sentinel() -> u16 {
    u16::MAX
}

impl Scene for SantaCruzScene {
    fn id(&self) -> &'static str {
        "santa_cruz"
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    fn layout(&self) -> SceneLayout {
        let ground_y = self.height.saturating_sub(Self::GROUND_HEIGHT + 1);
        SceneLayout {
            ground_y,
            chimney_pos: None,
            width: self.width,
            height: self.height,
        }
    }

    fn render(&self, renderer: &mut TerminalRenderer, ctx: &SceneContext<'_>) -> io::Result<()> {
        let style = SantaCruzStyle::resolve(ctx);
        let layout = self.layout();
        let ground_y = layout.ground_y;
        let w = self.width;

        Self::render_ground(renderer, w, ground_y, &style)?;

        // Sign in upper-left area
        let sign_y = ground_y.saturating_sub(18);
        if sign_y > 2 && w > 20 {
            Self::render_sign(renderer, 2, sign_y, &style)?;
        }

        // Rollercoaster track centered
        let center = w / 2;
        Self::render_track(renderer, center, ground_y, &style)?;

        Ok(())
    }
}
