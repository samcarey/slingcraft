#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::f64::consts::PI;

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, CentralPanel, MenuBar, TopBottomPanel},
};
use bevy_simple_subsecond_system::prelude::*;
use egui_plot::Plot;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "SlingCraft".to_string(),
            canvas: Some("#bevy".to_string()),
            prevent_default_event_handling: false,
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EguiPlugin::default())
    .add_plugins(SimpleSubsecondPlugin::default())
    .init_resource::<UiState>()
    .add_systems(Startup, setup)
    .add_systems(EguiPrimaryContextPass, ui_system);

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        web_sys::console::log_1(&"Starting Slingcraft...".into());
    }

    app.run();
}

#[derive(Resource, Default)]
struct UiState {
    label: String,
    value: f32,
}

impl UiState {
    fn new() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

fn setup(mut commands: Commands<'_, '_>) {
    commands.spawn(Camera2d);
    commands.insert_resource(UiState::new());
}

#[hot]
fn ui_system(
    mut contexts: EguiContexts<'_, '_>,
    _ui_state: ResMut<'_, UiState>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: EventWriter<'_, AppExit>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        MenuBar::new().ui(ui, |ui| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        exit.write(AppExit::Success);
                    }
                });
                ui.add_space(16.0);
            }

            egui::widgets::global_theme_preference_buttons(ui);
        });
    });

    CentralPanel::default().show(ctx, |ui| {
        ui.heading("SlingCraft");

        Plot::new("space_plot").show(ui, |ui| {
            ui.polygon(egui_plot::Polygon::new(
                "Gliblot",
                (0..45)
                    .into_iter()
                    .map(|i| i * 8)
                    .map(|i| i as f64 * PI / 180.)
                    .map(|d| [d.cos(), d.sin()])
                    .collect::<Vec<_>>(),
            ));
        });
    });
}
