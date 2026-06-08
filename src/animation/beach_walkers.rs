use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;
use rand::RngExt;
use std::io;
use std::f32::consts::TAU;

/// Walking person animation frames (4-frame walk cycle, 5 chars wide)
const WALK_A: &[&str] = &[
    r" o  ",
    r"/|\ ",
    r" |  ",
    r"/ \ ",
];

const WALK_B: &[&str] = &[
    r" o  ",
    r"/|\ ",
    r" |  ",
    r"_/ \_",
];

const WALK_C: &[&str] = &[
    r" o  ",
    r"/|\ ",
    r"/|  ",
    r" |  ",
];

const WALK_D: &[&str] = &[
    r" o  ",
    r"/|\ ",
    r" |  ",
    r" |\ ",
];

/// Dog walking alongside (shorter, 3 chars wide)
const DOG_A: &[&str] = &[
    r"  _",
    r" / \",
    r"|_o|",
];

const DOG_B: &[&str] = &[
    r" _ ",
    r"/ \",
    r"|o_|",
];

/// Seagull stand/walk (2 chars wide)
const GULL_A: &[&str] = &[
    r">v",
];

const GULL_B: &[&str] = &[
    r"v<",
];

#[derive(Clone, Copy, PartialEq)]
enum WalkerKind {
    Person,
    DogOwner,
    Seagull,
}

struct Walker {
    x: f32,
    walk_phase: f32,
    speed: f32,
    kind: WalkerKind,
    /// 0 = walk right, 1 = walk left
    direction: u8,
    /// Y offset from ground_y
    y_offset: u16,
}

pub struct BeachWalkersSystem {
    walkers: Vec<Walker>,
    width: u16,
    height: u16,
    frame_tick: u32,
}

impl BeachWalkersSystem {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            walkers: Vec::new(),
            width,
            height,
            frame_tick: 0,
        }
    }

    fn is_beach(ctx: &FrameContext<'_>) -> bool {
        let id = ctx.state.active_scene_id;
        id == "beach" || id == "santa_cruz"
    }

    fn spawn_walkers(&mut self, rng: &mut (impl Rng + ?Sized)) {
        // Cap total walkers
        if self.walkers.len() >= 8 {
            return;
        }

        let w = self.width as f32;
        if w < 30.0 {
            return;
        }

        // Spawn chance per frame
        if rng.random::<f32>() > 0.008 {
            return;
        }

        // Decide walk direction and starting side
        let going_right = rng.random::<bool>();
        let start_x = if going_right { -6.0 } else { w + 2.0 };
        let speed = 0.1 + rng.random::<f32>() * 0.25;
        let dir = if going_right { 0 } else { 1 };

        // Pick kind
        let kind_roll = rng.random::<f32>();
        let kind = if kind_roll < 0.55 {
            WalkerKind::Person
        } else if kind_roll < 0.85 {
            WalkerKind::DogOwner
        } else {
            WalkerKind::Seagull
        };

        // Y offset: walkers appear on sand rows 2-6 (middle of the beach)
        let y_off = match kind {
            WalkerKind::Person => rng.random_range(2u16..6),
            WalkerKind::DogOwner => rng.random_range(2u16..6),
            WalkerKind::Seagull => rng.random_range(1u16..5),
        };

        self.walkers.push(Walker {
            x: start_x,
            walk_phase: rng.random::<f32>() * TAU,
            speed,
            kind,
            direction: dir,
            y_offset: y_off,
        });
    }

    fn render_walker(
        renderer: &mut TerminalRenderer,
        frames: &[&[&str]],
        frame_idx: usize,
        x: u16,
        ground_y: u16,
        y_offset: u16,
        mirrored: bool,
        primary_color: Color,
        accent_color: Color,
    ) -> io::Result<()> {
        let frame = frames[frame_idx % frames.len()];
        let h = frame.len() as u16;
        // Place feet on the sand: feet at ground_y + y_offset, walker extends upward
        let start_y = ground_y.saturating_add(y_offset).saturating_sub(h);

        for (row, line) in frame.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let draw_col = if mirrored {
                    x + (line.len() - 1 - col) as u16
                } else {
                    x + col as u16
                };

                let color = match ch {
                    'o' | 'O' => accent_color, // head
                    '/' | '\\' | '|' | '_' => primary_color, // body/limbs
                    _ => primary_color,
                };
                renderer.render_char(draw_col, start_y + row as u16, ch, color)?;
            }
        }
        Ok(())
    }
}

impl AnimationSystem for BeachWalkersSystem {
    fn id(&self) -> &'static str {
        "beach_walkers"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::PostScene
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        Self::is_beach(ctx)
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
        // Clear walkers on resize to avoid stale positions
        self.walkers.clear();
    }

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        if !Self::is_beach(ctx) {
            self.walkers.clear();
            return;
        }

        self.frame_tick = self.frame_tick.wrapping_add(1);

        // Spawn new walkers
        self.spawn_walkers(rng);

        let w = self.width as f32;

        // Update positions
        for walker in &mut self.walkers {
            walker.walk_phase = (walker.walk_phase + 0.15) % TAU;

            let dx = walker.speed;
            if walker.direction == 0 {
                walker.x += dx;
            } else {
                walker.x -= dx;
            }
        }

        // Remove walkers that have moved off screen
        self.walkers.retain(|wk| {
            if wk.direction == 0 {
                wk.x < w + 8.0
            } else {
                wk.x > -8.0
            }
        });
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>) -> io::Result<()> {
        if !Self::is_beach(ctx) {
            return Ok(());
        }

        let ground_y = ctx.horizon_y;
        let is_day = ctx.conditions.sun.is_day;

        let (body_color, head_color) = if is_day {
            (
                Color::Rgb { r: 40, g: 80, b: 160 },    // body (clothing)
                Color::Rgb { r: 255, g: 210, b: 150 },   // skin
            )
        } else {
            (
                Color::Rgb { r: 30, g: 50, b: 90 },
                Color::Rgb { r: 180, g: 140, b: 100 },
            )
        };

        for walker in &self.walkers {
            let x = walker.x.round() as i32;
            if x < -8 || x + 5 > self.width as i32 + 8 {
                continue;
            }
            let x = x.clamp(0, self.width as i32 - 1) as u16;
            let mirrored = walker.direction == 1;

            match walker.kind {
                WalkerKind::Person => {
                    let frames = [WALK_A, WALK_B, WALK_C, WALK_D];
                    let fidx = (walker.walk_phase.sin() * 4.0 + 4.0) as usize % 4;
                    Self::render_walker(
                        renderer, &frames, fidx, x, ground_y,
                        walker.y_offset, mirrored, body_color, head_color,
                    )?;
                }
                WalkerKind::DogOwner => {
                    // Person frame
                    let frames = [WALK_A, WALK_B, WALK_C, WALK_D];
                    let fidx = (walker.walk_phase.sin() * 4.0 + 4.0) as usize % 4;
                    Self::render_walker(
                        renderer, &frames, fidx, x, ground_y,
                        walker.y_offset, mirrored, body_color, head_color,
                    )?;

                    // Dog alongside (offset based on direction)
                    let dog_frames = [DOG_A, DOG_B];
                    let dog_fidx = (walker.walk_phase.sin() * 2.0 + 2.0) as usize % 2;
                    let dog_x = if mirrored { x + 4 } else { x.saturating_sub(4) };
                    Self::render_walker(
                        renderer, &dog_frames, dog_fidx, dog_x, ground_y,
                        walker.y_offset + 1, mirrored,
                        Color::DarkYellow,
                        Color::Rgb { r: 180, g: 120, b: 50 },
                    )?;
                }
                WalkerKind::Seagull => {
                    let frames = [GULL_A, GULL_B];
                    let fidx = (walker.walk_phase.sin() * 2.0 + 2.0) as usize % 2;
                    let gull_color = if is_day { Color::White } else { Color::Grey };
                    Self::render_walker(
                        renderer, &frames, fidx, x, ground_y,
                        walker.y_offset, mirrored, gull_color, gull_color,
                    )?;
                }
            }
        }

        Ok(())
    }
}
