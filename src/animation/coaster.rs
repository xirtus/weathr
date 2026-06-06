use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;
use std::io;

/// Animated rollercoaster car that follows a predefined track profile.
/// Active only when scene is "santa_cruz".
pub struct CoasterSystem {
    /// Progress along the track, 0.0..1.0, cycling
    progress: f32,
    width: u16,
    height: u16,
    /// Cached (x,y) waypoints from the last render
    waypoints: Vec<(u16, u16)>,
    /// Whether to move forward or backward (simulate lap)
    going_forward: bool,
}

impl CoasterSystem {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            progress: 0.0,
            width,
            height,
            waypoints: Vec::new(),
            going_forward: true,
        }
    }

    fn is_active_scene(ctx: &FrameContext<'_>) -> bool {
        ctx.state.active_scene_id == "santa_cruz"
    }

    /// Compute the track profile waypoints for the given layout, matching
    /// the logic in SantaCruzScene::render_track.
    fn compute_waypoints(&self, ground_y: u16) -> Vec<(u16, u16)> {
        let center_x = self.width / 2;
        let ax = center_x.saturating_sub(30);
        let base_y = ground_y.saturating_sub(2);

        #[rustfmt::skip]
        let profile: &[(i16, i16)] = &[
            (0,  0), (2,  0), (4,  1), (6,  2), (8,  3), (10, 4),
            (12, 6), (14, 8), (16, 10), (18, 11), (20, 11),
            (22, 10), (24, 8), (26, 6), (28, 4), (30, 2),
            (32, 1), (34, 2), (36, 4), (38, 5),
            (40, 4), (42, 3), (44, 2),
            (46, 3), (48, 3),
            (50, 2), (52, 1), (54, 0), (56, 0),
        ];

        profile
            .iter()
            .map(|&(rx, yu)| {
                let tx = (ax as i16 + rx).max(0) as u16;
                let ty = (base_y as i16 - yu).max(0) as u16;
                (tx, ty)
            })
            .collect()
    }

    /// Interpolate between waypoints at a given progress (0.0..1.0).
    fn position_at(waypoints: &[(u16, u16)], t: f32) -> (u16, u16) {
        if waypoints.is_empty() {
            return (0, 0);
        }
        let n = waypoints.len();
        let scaled = t.clamp(0.0, 1.0) * (n - 1) as f32;
        let idx = scaled as usize;
        let frac = scaled - idx as f32;

        if idx + 1 >= n {
            return waypoints[n - 1];
        }

        let (x0, y0) = waypoints[idx];
        let (x1, y1) = waypoints[idx + 1];
        let x = (x0 as f32 + (x1 as f32 - x0 as f32) * frac) as u16;
        let y = (y0 as f32 + (y1 as f32 - y0 as f32) * frac) as u16;
        (x, y)
    }

    fn car_speed(t: f32) -> f32 {
        // Slow on the lift hill (first 40%), fast on the drop (40-55%), moderate rest
        if t < 0.40 {
            0.0008 // slow climbing
        } else if t < 0.55 {
            0.006  // fast on the drop
        } else {
            0.002  // moderate through the rest
        }
    }
}

impl AnimationSystem for CoasterSystem {
    fn id(&self) -> &'static str {
        "coaster"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::PostScene
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        Self::is_active_scene(ctx)
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
        self.waypoints.clear(); // recalculate next frame
    }

    fn update(&mut self, ctx: &FrameContext<'_>, _rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        if !Self::is_active_scene(ctx) {
            return;
        }

        // Recompute waypoints if needed
        if self.waypoints.is_empty() {
            self.waypoints = self.compute_waypoints(ctx.horizon_y);
        }

        let speed = Self::car_speed(self.progress);
        self.progress += speed;

        if self.progress >= 1.0 {
            // Lap complete — reset to start
            self.progress = 0.0;
        }
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>) -> io::Result<()> {
        if !Self::is_active_scene(ctx) || self.waypoints.is_empty() {
            return Ok(());
        }

        let (car_x, car_y) = Self::position_at(&self.waypoints, self.progress);

        // Coaster car: [oO] — person silhouette inside
        let is_day = ctx.conditions.sun.is_day;
        let car_color = if is_day { Color::Red } else { Color::DarkRed };
        let person_color = if is_day { Color::White } else { Color::Grey };

        // Draw the car (5 chars wide)
        let car_str = "[oO]";
        let cx = car_x.saturating_sub(2);
        for (i, ch) in car_str.chars().enumerate() {
            let color = if ch == 'o' || ch == 'O' { person_color } else { car_color };
            renderer.render_char(cx + i as u16, car_y, ch, color)?;
        }

        // Trailing car (second car, slightly behind)
        let trail_t = (self.progress - 0.06).max(0.0);
        let (trail_x, trail_y) = Self::position_at(&self.waypoints, trail_t);
        let tcx = trail_x.saturating_sub(2);
        for (i, ch) in car_str.chars().enumerate() {
            let color = if ch == 'o' || ch == 'O' { person_color } else { car_color };
            renderer.render_char(tcx + i as u16, trail_y, ch, color)?;
        }

        Ok(())
    }
}
