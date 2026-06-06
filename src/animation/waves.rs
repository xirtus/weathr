use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;
use std::io;

pub struct WaveSystem {
    offset: f32,
    width: u16,
    height: u16,
}

impl WaveSystem {
    pub fn new(width: u16, height: u16) -> Self {
        Self { offset: 0.0, width, height }
    }

    fn is_beach_scene(ctx: &FrameContext<'_>) -> bool {
        let id = ctx.state.active_scene_id;
        id == "beach" || id == "santa_cruz"
    }
}

impl AnimationSystem for WaveSystem {
    fn id(&self) -> &'static str {
        "waves"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Foreground
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        Self::is_beach_scene(ctx)
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
    }

    fn update(&mut self, _ctx: &FrameContext<'_>, _rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        self.offset += 0.15;
        if self.offset > self.width as f32 {
            self.offset -= self.width as f32;
        }
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>) -> io::Result<()> {
        if !Self::is_beach_scene(ctx) {
            return Ok(());
        }

        let horizon_y = ctx.horizon_y;
        if horizon_y < 2 {
            return Ok(());
        }

        let w = self.width;
        let is_day = ctx.conditions.sun.is_day;
        let wave_color = if is_day { Color::Cyan } else { Color::DarkCyan };
        let foam_color = if is_day { Color::White } else { Color::Grey };

        // Three scrolling wave rows above the ground line
        for layer in 0..3u16 {
            let row_y = horizon_y.saturating_sub(layer + 1);
            let phase = self.offset + (layer as f32 * 8.0);
            let speed = 1.0 - (layer as f32 * 0.25);

            for col in 0..w {
                let wave_x = ((col as f32 + phase * speed) % 16.0) as usize;
                let ch = match wave_x {
                    0..=3  => '~',
                    4..=5  => '^',
                    6..=9  => '~',
                    10..=11 => ' ',
                    _       => '~',
                };
                if ch != ' ' {
                    let color = if wave_x == 4 || wave_x == 5 { foam_color } else { wave_color };
                    renderer.render_char(col, row_y, ch, color)?;
                }
            }
        }

        Ok(())
    }
}
