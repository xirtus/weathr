use crate::animation_manager::AnimationManager;
use crate::app_state::AppState;
use crate::config::{Config, Provider};
use crate::error::WeatherError;
use crate::gallery::{GALLERY, CityEntry, DEFAULT_CITIES, scene_for_city};
use crate::render::TerminalRenderer;
use crate::scene::beach::BeachScene;
use crate::scene::city::CityScene;
use crate::scene::farm::FarmScene;
use crate::scene::mountain::MountainScene;
use crate::scene::overlay::OverlayRegistry;
use crate::scene::santa_cruz::SantaCruzScene;
use crate::scene::world::WorldScene;
use crate::scene::{SceneContext, SceneRegistry};
use crate::theme::ThemeRegistry;

use crate::weather::provider::WeatherProvider;
use crate::weather::provider::met_office::{MetOfficeProvider, MetOfficeProviderConfig};
use crate::weather::types::CelestialEvents;
use crate::weather::{
    OpenMeteoProvider, WeatherClient, WeatherCondition, WeatherData, WeatherLocation, WeatherUnits,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use serde::Deserialize;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

const REFRESH_INTERVAL: Duration = Duration::from_secs(300);
const INPUT_POLL_FPS: u64 = 30;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / INPUT_POLL_FPS);
const DEFAULT_THEME_ID: &str = "default";

/// Owned version of gallery::CityEntry (allows runtime-constructed entries).
struct OwnedCityEntry {
    name: String,
    lat: f64,
    lon: f64,
    scene_id: &'static str,
}

impl OwnedCityEntry {
    fn from_static(e: &CityEntry) -> Self {
        Self { name: e.name.to_string(), lat: e.lat, lon: e.lon, scene_id: e.scene_id }
    }

    fn location(&self) -> WeatherLocation {
        WeatherLocation { latitude: self.lat, longitude: self.lon, elevation: None }
    }
}

fn spawn_weather_fetch(
    provider: Arc<dyn WeatherProvider>,
    wanted_provider: Provider,
    location: WeatherLocation,
    units: WeatherUnits,
    tx: mpsc::Sender<Result<WeatherData, WeatherError>>,
) {
    use crate::weather::normalizer::WeatherNormalizer;
    tokio::spawn(async move {
        let result = provider.get_current_weather(&location, &units).await;
        let mapped = result.map(|r| WeatherNormalizer::normalize(r));
        let _ = tx.send(mapped).await;
    });
    let _ = wanted_provider; // suppress unused
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeBindings {
    theme_id: &'static str,
    scene_id: &'static str,
    overlay_id: Option<&'static str>,
}

fn resolve_theme_bindings(
    themes: &ThemeRegistry,
    scenes: &SceneRegistry,
    overlays: &OverlayRegistry,
) -> ThemeBindings {
    let active_theme = themes.active();
    let mut theme_id = active_theme.id;
    let mut scene_id = active_theme.scene_id;
    let mut overlay_id = active_theme.overlay_id;

    let scene_missing = scenes.get(scene_id).is_none();
    if scene_missing {
        if theme_id != DEFAULT_THEME_ID {
            eprintln!(
                "Warning: theme '{}' references missing scene '{}'. Falling back to '{}'.",
                theme_id, scene_id, DEFAULT_THEME_ID
            );
            let fallback_theme = themes
                .get(DEFAULT_THEME_ID)
                .expect("default theme must be registered");
            theme_id = fallback_theme.id;
            scene_id = fallback_theme.scene_id;
            overlay_id = fallback_theme.overlay_id;
        } else {
            panic!("default theme references missing scene '{}'.", scene_id);
        }
    }

    if scenes.get(scene_id).is_none() {
        panic!(
            "theme '{}' references missing scene '{}', and no fallback scene is available",
            theme_id, scene_id
        );
    }

    let validated_overlay = overlay_id.and_then(|id| {
        if overlays.get(id).is_some() {
            Some(id)
        } else {
            eprintln!(
                "Warning: theme '{}' references missing overlay '{}'. Overlay disabled.",
                theme_id, id
            );
            None
        }
    });

    ThemeBindings {
        theme_id,
        scene_id,
        overlay_id: validated_overlay,
    }
}

fn generate_offline_weather(rng: &mut impl rand::Rng) -> WeatherData {
    use chrono::{Local, Timelike};
    use rand::RngExt;

    let now = Local::now();
    let hour = now.hour();
    let is_day = (6..18).contains(&hour);

    let conditions = [
        WeatherCondition::Clear,
        WeatherCondition::PartlyCloudy,
        WeatherCondition::Cloudy,
        WeatherCondition::Rain,
    ];

    let condition = conditions[rng.random_range(0..conditions.len())];

    WeatherData {
        condition,
        temperature: rng.random_range(10.0..25.0),
        precipitation: if condition.is_raining() {
            rng.random_range(1.0..5.0)
        } else {
            0.0
        },
        wind_speed: rng.random_range(5.0..15.0),
        wind_direction: rng.random_range(0.0..360.0),
        sun: CelestialEvents::from_bool(is_day),
        moon_phase: Some(0.5),
        timestamp: now.format("%Y-%m-%dT%H:%M:%S").to_string(),
        attribution: "".to_string(),
    }
}

pub struct App {
    state: AppState,
    animations: AnimationManager,
    scenes: SceneRegistry,
    overlays: OverlayRegistry,
    themes: ThemeRegistry,
    active_scene_id: &'static str,
    active_overlay_id: Option<&'static str>,
    weather_receiver: mpsc::Receiver<Result<WeatherData, WeatherError>>,
    /// Sender so city-switch can push a fresh weather result onto the same channel
    weather_sender: mpsc::Sender<Result<WeatherData, WeatherError>>,
    hide_hud: bool,
    /// Current index into GALLERY for cycling visual scenes with [ / ]
    gallery_index: Option<usize>,
    /// Ordered city list for ← → navigation
    cities: Vec<OwnedCityEntry>,
    /// Index into `cities` of the currently displayed city
    city_index: usize,
    /// Weather provider (needed for on-demand city-switch fetches)
    provider: Arc<dyn WeatherProvider>,
    wanted_provider: Provider,
    units: WeatherUnits,
}

impl App {
    pub fn new(
        config: &Config,
        simulate_condition: Option<String>,
        simulate_night: bool,
        show_leaves: bool,
        term_width: u16,
        term_height: u16,
        themes: ThemeRegistry,
    ) -> Self {
        let location = WeatherLocation {
            latitude: config.location.latitude,
            longitude: config.location.longitude,
            elevation: None,
        };

        let mut state = AppState::new(
            location,
            config.location.city.clone(),
            config.location.display,
            config.location.hide,
            config.units,
        );
        let mut animations = AnimationManager::new(term_width, term_height, show_leaves);

        // Auto-select scene based on city name before resolving theme
        let city_name = config.location.city.clone();
        let auto_scene_id = city_name
            .as_deref()
            .map(scene_for_city)
            .unwrap_or("world");

        let mut scenes = SceneRegistry::new();
        scenes.register(Box::new(WorldScene::new(term_width, term_height)));
        scenes.register(Box::new(CityScene::new(term_width, term_height, city_name.clone())));
        scenes.register(Box::new(FarmScene::new(term_width, term_height)));
        scenes.register(Box::new(BeachScene::new(term_width, term_height)));
        scenes.register(Box::new(MountainScene::new(term_width, term_height)));
        scenes.register(Box::new(SantaCruzScene::new(term_width, term_height)));

        let overlays = OverlayRegistry::new();

        // If auto-scene differs from theme's scene, override the theme binding
        let mut bindings = resolve_theme_bindings(&themes, &scenes, &overlays);
        if bindings.scene_id == "world" && auto_scene_id != "world" {
            // Use auto-detected scene (city name matched)
            bindings = ThemeBindings {
                theme_id: auto_scene_id,
                scene_id: auto_scene_id,
                overlay_id: None,
            };
        }

        // Build city list: user's current location first, then the built-in gallery
        let mut cities: Vec<OwnedCityEntry> = Vec::new();
        cities.push(OwnedCityEntry {
            name: city_name.clone().unwrap_or_else(|| {
                format!("{:.2}°, {:.2}°", config.location.latitude, config.location.longitude)
            }),
            lat: config.location.latitude,
            lon: config.location.longitude,
            scene_id: auto_scene_id,
        });
        for e in DEFAULT_CITIES {
            cities.push(OwnedCityEntry::from_static(e));
        }

        let (tx, rx) = mpsc::channel(4);

        let wanted_provider = config
            .provider
            .keys()
            .next()
            .cloned()
            .unwrap_or(Provider::default());

        let provider: Arc<dyn WeatherProvider> = match wanted_provider {
            Provider::OpenMeteo => Arc::new(OpenMeteoProvider::new()),
            Provider::MetOffice => {
                let provider_config = {
                    if let Some(provider_config) = config.provider.get(&wanted_provider) {
                        MetOfficeProviderConfig::deserialize(provider_config.clone()).unwrap()
                    } else {
                        MetOfficeProviderConfig::default()
                    }
                };
                Arc::new(MetOfficeProvider::new(provider_config).unwrap())
            }
        };

        if let Some(ref condition_str) = simulate_condition {
            let simulated_condition =
                condition_str
                    .parse::<WeatherCondition>()
                    .unwrap_or_else(|e| {
                        eprintln!("{}", e);
                        WeatherCondition::Clear
                    });

            let weather = WeatherData {
                condition: simulated_condition,
                temperature: 20.0,
                precipitation: if simulated_condition.is_raining() { 2.5 } else { 0.0 },
                wind_speed: if simulated_condition.is_thunderstorm() { 45.0 } else { 10.0 },
                wind_direction: 225.0,
                sun: CelestialEvents::from_bool(!simulate_night),
                moon_phase: Some(0.5),
                timestamp: "simulated".to_string(),
                attribution: "".to_string(),
            };

            let rain_intensity = weather.condition.rain_intensity();
            let snow_intensity = weather.condition.snow_intensity();
            let wind_speed = weather.wind_speed;
            let wind_direction = weather.wind_direction;

            state.update_weather(weather);
            animations.update_rain_intensity(rain_intensity);
            animations.update_snow_intensity(snow_intensity);
            animations.update_wind(wind_speed as f32, wind_direction as f32);
        } else {
            let weather_client = WeatherClient::new(Arc::clone(&provider), REFRESH_INTERVAL);
            let tx_bg = tx.clone();
            let units = config.units;

            tokio::spawn(async move {
                loop {
                    let result = weather_client
                        .get_current_weather(&location, &units, wanted_provider)
                        .await;
                    if tx_bg.send(result).await.is_err() {
                        break;
                    }
                    tokio::time::sleep(REFRESH_INTERVAL).await;
                }
            });
        }

        let initial_scene_id = bindings.scene_id;
        state.active_scene_id = initial_scene_id;

        Self {
            state,
            animations,
            scenes,
            overlays,
            themes,
            active_scene_id: initial_scene_id,
            active_overlay_id: bindings.overlay_id,
            weather_receiver: rx,
            weather_sender: tx,
            hide_hud: config.hide_hud,
            gallery_index: None,
            cities,
            city_index: 0,
            provider,
            wanted_provider,
            units: config.units,
        }
    }

    pub async fn run(&mut self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        let mut rng = rand::rng();
        let mut attribution = "Awaiting weather data".to_string();

        loop {
            match self.weather_receiver.try_recv() {
                Ok(result) => match result {
                    Ok(weather) => {
                        let rain_intensity = weather.condition.rain_intensity();
                        let snow_intensity = weather.condition.snow_intensity();
                        let fog_intensity = weather.condition.fog_intensity();
                        let wind_speed = weather.wind_speed;
                        let wind_direction = weather.wind_direction;
                        attribution = weather.attribution.clone();

                        if let Some(moon_phase) = weather.moon_phase {
                            self.animations.update_moon_phase(moon_phase);
                        }

                        self.state.update_weather(weather);
                        self.animations.update_rain_intensity(rain_intensity);
                        self.animations.update_snow_intensity(snow_intensity);
                        self.animations.update_fog_intensity(fog_intensity);
                        self.animations
                            .update_wind(wind_speed as f32, wind_direction as f32);
                    }
                    Err(error) => {
                        let error_msg = match &error {
                            WeatherError::Network(net_err) => net_err.user_friendly_message(),
                            _ => format!("Failed to fetch weather: {}", error),
                        };

                        if self.state.current_weather.is_none() {
                            attribution = format!("Provider failed with {error_msg} - Simulating");
                            let offline_weather = generate_offline_weather(&mut rng);
                            let rain_intensity = offline_weather.condition.rain_intensity();
                            let snow_intensity = offline_weather.condition.snow_intensity();
                            let fog_intensity = offline_weather.condition.fog_intensity();
                            let wind_speed = offline_weather.wind_speed;
                            let wind_direction = offline_weather.wind_direction;

                            self.state.update_weather(offline_weather);
                            self.state.set_offline_mode(true);
                            self.animations.update_rain_intensity(rain_intensity);
                            self.animations.update_snow_intensity(snow_intensity);
                            self.animations.update_fog_intensity(fog_intensity);
                            self.animations
                                .update_wind(wind_speed as f32, wind_direction as f32);
                        } else {
                            self.state.set_offline_mode(true);
                            attribution = format!("Provider failed with {error_msg}");
                        }
                    }
                },
                Err(e) => {
                    if e == mpsc::error::TryRecvError::Disconnected {
                        attribution = "".to_string();
                    }
                }
            }

            renderer.clear()?;

            let theme = self.themes.active();
            let palette = &theme.palette;

            let (term_width, term_height) = renderer.get_size();
            let scene = self
                .scenes
                .get_mut(self.active_scene_id)
                .expect("active scene must be registered");
            scene.update_size(term_width, term_height);

            let layout = scene.layout();
            let ctx = SceneContext {
                conditions: &self.state.weather_conditions,
                palette,
                city_name: self.state.city_name.as_deref(),
            };

            self.animations.render_background(
                renderer,
                &self.state.weather_conditions,
                &self.state,
                &layout,
                &mut rng,
            )?;

            scene.render(renderer, &ctx)?;

            if let Some(ov_id) = self.active_overlay_id {
                if let Some(overlay) = self.overlays.get_mut(ov_id) {
                    overlay.update_size(term_width, term_height);
                    overlay.render(renderer, &ctx, &layout)?;
                }
            }

            self.animations.render_chimney_smoke(
                renderer,
                &self.state.weather_conditions,
                &self.state,
                &layout,
                &mut rng,
            )?;

            self.animations.render_foreground(
                renderer,
                &self.state.weather_conditions,
                &self.state,
                &layout,
                &mut rng,
            )?;

            self.state.update_loading_animation();
            self.state.update_cached_info();

            if !self.hide_hud {
                renderer.render_line_colored(
                    2,
                    1,
                    &self.state.cached_weather_info,
                    crossterm::style::Color::Cyan,
                )?;

                // City navigation bar (row 2)
                let n = self.cities.len();
                let city = &self.cities[self.city_index];
                let nav_str = format!(
                    " ← → city ({}/{})  [/] scene  r refresh  q quit ",
                    self.city_index + 1,
                    n
                );
                renderer.render_line_colored(2, 2, &nav_str, crossterm::style::Color::DarkGrey)?;

                // City name highlight (row 3, centered)
                let city_label = format!("  {}  ", city.name);
                let cx = term_width.saturating_sub(city_label.len() as u16) / 2;
                renderer.render_line_colored(cx, 3, &city_label, crossterm::style::Color::Yellow)?;
            }

            let attribution_x = if term_width > attribution.len() as u16 {
                term_width - attribution.len() as u16 - 2
            } else {
                0
            };
            let attribution_y = if term_height > 0 { term_height - 1 } else { 0 };
            renderer.render_line_colored(
                attribution_x,
                attribution_y,
                &attribution,
                crossterm::style::Color::DarkGrey,
            )?;

            // Gallery mode label (row 2 from bottom, centered)
            if let Some(idx) = self.gallery_index {
                let entry = &GALLERY[idx];
                let label = format!(
                    " Scene {}/{}: {} ",
                    idx + 1,
                    GALLERY.len(),
                    entry.label
                );
                let label_x = term_width.saturating_sub(label.len() as u16) / 2;
                let label_y = attribution_y.saturating_sub(1);
                renderer.render_line_colored(
                    label_x,
                    label_y,
                    &label,
                    crossterm::style::Color::Magenta,
                )?;
            }

            renderer.flush()?;

            if event::poll(FRAME_DURATION)? {
                match event::read()? {
                    Event::Resize(width, height) => {
                        renderer.manual_resize(width, height)?;
                        let (new_width, new_height) = renderer.get_size();
                        self.animations.on_resize(new_width, new_height);
                    }
                    Event::Key(key_event) => match key_event.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Char('c')
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            break;
                        }

                        // ← → navigate cities (fetches live weather)
                        KeyCode::Left | KeyCode::Right => {
                            let n = self.cities.len();
                            self.city_index = if key_event.code == KeyCode::Right {
                                (self.city_index + 1) % n
                            } else {
                                (self.city_index + n - 1) % n
                            };
                            self.gallery_index = None; // exit scene-gallery mode
                            let city = &self.cities[self.city_index];
                            let scene_id = city.scene_id;
                            let city_name = city.name.clone();
                            let loc = city.location();

                            // Switch scene + theme immediately
                            self.active_scene_id = scene_id;
                            self.state.active_scene_id = scene_id;
                            self.state.city_name = Some(city_name);
                            self.state.location = loc;
                            self.state.current_weather = None; // show loading
                            self.state.weather_info_needs_update = true;
                            let _ = self.themes.set_active(scene_id);

                            // Kick off a fresh weather fetch for new coordinates
                            spawn_weather_fetch(
                                Arc::clone(&self.provider),
                                self.wanted_provider,
                                loc,
                                self.units,
                                self.weather_sender.clone(),
                            );
                        }

                        // r = refresh weather for current city
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            let city = &self.cities[self.city_index];
                            let loc = city.location();
                            spawn_weather_fetch(
                                Arc::clone(&self.provider),
                                self.wanted_provider,
                                loc,
                                self.units,
                                self.weather_sender.clone(),
                            );
                        }

                        // [ / ] cycle visual scene gallery (no weather fetch)
                        KeyCode::Char(']') | KeyCode::Char('[') => {
                            let n = GALLERY.len();
                            let current = self.gallery_index.unwrap_or_else(|| {
                                GALLERY.iter().position(|e| e.scene_id == self.active_scene_id).unwrap_or(0)
                            });
                            let next = if key_event.code == KeyCode::Char(']') {
                                (current + 1) % n
                            } else {
                                (current + n - 1) % n
                            };
                            self.gallery_index = Some(next);
                            let entry = &GALLERY[next];
                            self.active_scene_id = entry.scene_id;
                            self.state.active_scene_id = entry.scene_id;
                            let _ = self.themes.set_active(entry.scene_id);
                        }

                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::TerminalRenderer;
    use crate::scene::overlay::SceneOverlay;
    use crate::scene::{Scene, SceneContext, SceneLayout};
    use crate::theme::catalogue::DEFAULT_PALETTE;
    use crate::theme::{Theme, ThemeRegistry};
    use std::io;

    struct TestScene {
        id: &'static str,
    }

    impl TestScene {
        fn new(id: &'static str) -> Self {
            Self { id }
        }
    }

    impl Scene for TestScene {
        fn id(&self) -> &'static str {
            self.id
        }

        fn update_size(&mut self, _width: u16, _height: u16) {}

        fn render(
            &self,
            _renderer: &mut TerminalRenderer,
            _ctx: &SceneContext<'_>,
        ) -> io::Result<()> {
            Ok(())
        }

        fn layout(&self) -> SceneLayout {
            SceneLayout {
                ground_y: 0,
                chimney_pos: None,
                width: 0,
                height: 0,
            }
        }
    }

    struct TestOverlay {
        id: &'static str,
    }

    impl TestOverlay {
        fn new(id: &'static str) -> Self {
            Self { id }
        }
    }

    impl SceneOverlay for TestOverlay {
        fn id(&self) -> &'static str {
            self.id
        }

        fn update_size(&mut self, _width: u16, _height: u16) {}

        fn render(
            &self,
            _renderer: &mut TerminalRenderer,
            _ctx: &SceneContext<'_>,
            _layout: &SceneLayout,
        ) -> io::Result<()> {
            Ok(())
        }
    }

    fn scene_registry_with_world() -> SceneRegistry {
        let mut scenes = SceneRegistry::new();
        scenes.register(Box::new(TestScene::new("world")));
        scenes
    }

    #[test]
    fn bindings_fall_back_to_default_when_scene_missing() {
        let scenes = scene_registry_with_world();
        let overlays = OverlayRegistry::new();
        let mut themes = ThemeRegistry::new();
        themes.register(Theme {
            id: "custom",
            display_name: "Custom",
            scene_id: "unknown",
            overlay_id: None,
            palette: DEFAULT_PALETTE,
        });
        themes.set_active("custom").unwrap();

        let bindings = resolve_theme_bindings(&themes, &scenes, &overlays);

        assert_eq!(bindings.theme_id, DEFAULT_THEME_ID);
        assert_eq!(bindings.scene_id, "world");
        assert_eq!(bindings.overlay_id, None);
    }

    #[test]
    fn bindings_disable_unregistered_overlay() {
        let scenes = scene_registry_with_world();
        let overlays = OverlayRegistry::new();
        let mut themes = ThemeRegistry::new();
        themes.register(Theme {
            id: "overlay-theme",
            display_name: "Overlay Theme",
            scene_id: "world",
            overlay_id: Some("hud"),
            palette: DEFAULT_PALETTE,
        });
        themes.set_active("overlay-theme").unwrap();

        let bindings = resolve_theme_bindings(&themes, &scenes, &overlays);

        assert_eq!(bindings.theme_id, "overlay-theme");
        assert_eq!(bindings.scene_id, "world");
        assert_eq!(bindings.overlay_id, None);
    }

    #[test]
    fn bindings_keep_registered_overlay() {
        let scenes = scene_registry_with_world();
        let mut overlays = OverlayRegistry::new();
        overlays.register(Box::new(TestOverlay::new("hud")));
        let mut themes = ThemeRegistry::new();
        themes.register(Theme {
            id: "overlay",
            display_name: "Overlay",
            scene_id: "world",
            overlay_id: Some("hud"),
            palette: DEFAULT_PALETTE,
        });
        themes.set_active("overlay").unwrap();

        let bindings = resolve_theme_bindings(&themes, &scenes, &overlays);

        assert_eq!(bindings.theme_id, "overlay");
        assert_eq!(bindings.overlay_id, Some("hud"));
    }
}
