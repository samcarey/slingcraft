#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_simple_subsecond_system::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Slingcraft".to_string(),
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
    mut ui_state: ResMut<'_, UiState>,
    #[cfg(not(target_arch = "wasm32"))]
    mut exit: EventWriter<'_, AppExit>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
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

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Slingcraft - Bevy + egu");

        ui.horizontal(|ui| {
            ui.label("Write something: ");
            ui.text_edit_singleline(&mut ui_state.label);
        });

        ui.add(egui::Slider::new(&mut ui_state.value, 0.0..=10.0).text("value"));
        if ui.button("Increment").clicked() {
            ui_state.value += 1.0;
        }

        ui.separator();

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Powered by ");
                ui.hyperlink_to("Bevy", "https://bevyengine.org");
                ui.label(" and ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(".");
            });
            egui::warn_if_debug_build(ui);
        });
    });
}