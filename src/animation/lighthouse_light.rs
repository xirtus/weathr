use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::{Rng, RngExt};
use std::io;

/// A single twinkling sparkle particle at the lighthouse lantern.
struct Spark {
    /// Screen x offset from the lantern centre.
    dx: i16,
    /// Screen y offset from the lantern row.
    dy: i16,
    /// Twinkle phase (0 – 2π).
    phase: f32,
    /// Intrinsic brightness 0–1.
    brightness: f32,
    /// Twinkle speed multiplier.
    speed: f32,
}

/// Subtle twinkling star-field around the existing `g"y` lighthouse beam
/// in the Maldives ASCII art.  Does NOT sweep — it sparkles in place
/// like the night-sky stars, enhancing the hand-drawn beam.
pub struct LighthouseLightSystem {
    /// Lantern screen-x (left edge of the `g"y` beam).
    lx: u16,
    /// Lantern screen-y (top row of the Maldives art).
    ly: u16,
    /// Twinkling particles clustered around the lantern & beam.
    sparks: Vec<Spark>,
    /// Terminal dimensions.
    width: u16,
    height: u16,
    /// Monotonic frame counter for slow cycle effects.
    frame: u64,
}

impl LighthouseLightSystem {
    // ── Art constants — must match maldives.txt ──
    const ART_WIDTH: u16 = 74;
    const ART_HEIGHT: u16 = 10;
    /// Column of the `g` character (leftmost of `g"y`) in the art.
    const ART_LIGHT_COL: u16 = 6;

    pub fn new(width: u16, height: u16) -> Self {
        let (lx, ly) = Self::compute_light_pos(width, height);
        let sparks = Self::seed_sparks();
        Self { lx, ly, sparks, width, height, frame: 0 }
    }

    /// Recompute the screen position of the lantern from terminal size.
    fn compute_light_pos(width: u16, height: u16) -> (u16, u16) {
        // ground_y = height - BeachScene::GROUND_HEIGHT - 1  (= height - 9)
        let ground_y = height.saturating_sub(9);
        let offset_x = if width > Self::ART_WIDTH {
            (width - Self::ART_WIDTH) / 2
        } else {
            0
        };
        let start_y = ground_y.saturating_sub(Self::ART_HEIGHT + 2);
        let lx = offset_x + Self::ART_LIGHT_COL; // leftmost of g"y
        let ly = start_y;                         // top row of art
        (lx, ly)
    }

    /// Create the initial set of sparkle particles.
    fn seed_sparks() -> Vec<Spark> {
        let mut rng = rand::rng();
        let mut sparks = Vec::with_capacity(12);

        for _ in 0..12 {
            // Cluster tightly around the `g"y` beam (cols 0-3, rows -1..1)
            let dx: i16 = {
                let r: f32 = rng.random();
                if r < 0.5 {
                    // mostly along the beam (cols 0..3)
                    (rng.random::<u16>() % 4) as i16
                } else {
                    // occasionally just left of the lantern (-1..1)
                    (rng.random::<u16>() % 2) as i16 - 1
                }
            };
            let dy: i16 = (rng.random::<u16>() % 3) as i16 - 1;
            sparks.push(Spark {
                dx,
                dy,
                phase: rng.random::<f32>() * std::f32::consts::TAU,
                brightness: rng.random::<f32>() * 0.4 + 0.6,
                speed: rng.random::<f32>() * 0.03 + 0.02,
            });
        }
        sparks
    }

    /// Re-seed sparkles (called on resize to avoid stale offsets).
    fn reseed(&mut self) {
        self.sparks = Self::seed_sparks();
    }
}

impl AnimationSystem for LighthouseLightSystem {
    fn id(&self) -> &'static str {
        "lighthouse_light"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::PostScene
    }

    fn is_active(&self, _ctx: &FrameContext<'_>) -> bool {
        // Only active on the Maldives beach scene.
        _ctx.state.active_scene_id == "beach"
            && _ctx
                .state
                .city_name
                .as_deref()
                .map(|n| n.to_lowercase())
                .unwrap_or_default()
                .contains("maldives")
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
        let (lx, ly) = Self::compute_light_pos(size.width, size.height);
        self.lx = lx;
        self.ly = ly;
        self.reseed();
    }

    fn update(
        &mut self,
        _ctx: &FrameContext<'_>,
        _rng: &mut dyn Rng,
        _commands: &mut FrameCommands,
    ) {
        self.frame = self.frame.wrapping_add(1);
        // Advance each spark's phase independently
        for s in &mut self.sparks {
            s.phase = (s.phase + s.speed) % std::f32::consts::TAU;
        }
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        _ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        // ── Twinkling sparkles ──
        for s in &self.sparks {
            let sx = (self.lx as i32 + s.dx as i32).max(0) as u16;
            if sx >= self.width {
                continue;
            }
            let sy = (self.ly as i32 + s.dy as i32).max(0) as u16;
            if sy >= self.height {
                continue;
            }

            // Twinkling brightness: sine wave mapped to 0..1
            let twinkle = ((s.phase.sin() + 1.0) / 2.0) * s.brightness;

            // Slow overall pulse (all sparks share this)
            let pulse = ((self.frame as f32 * 0.02).sin() + 1.0) / 2.0; // 0..1
            let effective = twinkle * (0.4 + 0.6 * pulse);

            let (ch, color) = if effective > 0.7 {
                (
                    '*',
                    Color::Rgb {
                        r: 255,
                        g: (220.0 * effective) as u8,
                        b: (30.0 * effective) as u8,
                    },
                )
            } else if effective > 0.4 {
                (
                    '✧',
                    Color::Rgb {
                        r: (240.0 * effective) as u8,
                        g: (190.0 * effective) as u8,
                        b: (40.0 * effective) as u8,
                    },
                )
            } else if effective > 0.2 {
                (
                    '·',
                    Color::Rgb {
                        r: (200.0 * effective) as u8,
                        g: (150.0 * effective) as u8,
                        b: (50.0 * effective) as u8,
                    },
                )
            } else if effective > 0.06 {
                (
                    '°',
                    Color::Rgb {
                        r: (150.0 * effective) as u8,
                        g: (110.0 * effective) as u8,
                        b: (60.0 * effective) as u8,
                    },
                )
            } else {
                continue; // invisible — don't render at all
            };

            renderer.render_char(sx, sy, ch, color)?;
        }

        Ok(())
    }
}
