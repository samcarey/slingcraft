use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, CentralPanel, Color32, MenuBar, Stroke, TopBottomPanel, vec2},
};
use bevy_simple_subsecond_system::prelude::*;
use egui_plot::Plot;
use std::f64::consts::PI;

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
    .add_systems(Startup, setup)
    .add_systems(EguiPrimaryContextPass, ui_system);

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        web_sys::console::log_1(&"Starting Slingcraft...".into());
    }

    app.run();
}

#[derive(Component)]
struct Body {
    label: String,
    radius: f64,
    position: [f64; 2],
    color: Color32,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(Body {
        label: "Gliblot".into(),
        radius: 5.,
        position: [0., 0.],
        color: Color32::RED,
    });
    commands.spawn(Body {
        label: "Moon".into(),
        radius: 2.,
        position: [20., 20.],
        color: Color32::BLUE,
    });
}

#[hot]
fn ui_system(
    mut contexts: EguiContexts,
    bodies: Query<&Body>,
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

        Plot::new("space_plot")
            .data_aspect(1.)
            .allow_axis_zoom_drag(false)
            .allow_boxed_zoom(false)
            .allow_scroll(false)
            .set_margin_fraction(vec2(0.2, 0.2))
            .cursor_color(Color32::TRANSPARENT)
            .show_grid(false)
            .show_axes(false)
            .show_x(false)
            .show_y(false)
            .show(ui, |ui| {
                for Body {
                    label,
                    radius,
                    position: [x, y],
                    color,
                } in bodies
                {
                    ui.polygon(
                        egui_plot::Polygon::new(
                            label,
                            (0..90)
                                .into_iter()
                                .map(|i| i * 4)
                                .map(|i| i as f64 * PI / 180.)
                                .map(|d| [*radius * d.cos(), *radius * d.sin()])
                                .map(|[x_edge, y_edge]| [*x + x_edge, *y + y_edge])
                                .collect::<Vec<_>>(),
                        )
                        .fill_color(color.gamma_multiply(0.75))
                        .stroke(Stroke::new(2., color.gamma_multiply(1.2))),
                    );
                }
            });
    });
}
