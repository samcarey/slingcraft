use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, CentralPanel, Color32, MenuBar, Sense, Stroke, TopBottomPanel, vec2},
};
use bevy_persistent::prelude::*;
use bevy_persistent_windows::prelude::*;
use bevy_simple_subsecond_system::prelude::*;
use egui_plot::Plot;
use std::f32::consts::PI;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: None, // Disable default primary window
            ..default()
        }),
        EguiPlugin::default(),
        SimpleSubsecondPlugin::default(),
        PersistentWindowsPlugin,
    ))
    .add_systems(Startup, (setup, spawn_persistent_window))
    .add_systems(EguiPrimaryContextPass, ui_system)
    .add_systems(Update, (gravity, motion, regulate_energy));

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        web_sys::console::log_1(&"Starting Slingcraft...".into());
    }

    app.run();
}

fn spawn_persistent_window(mut commands: Commands) {
    let state_directory = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("slingcraft")
        .join("state");

    commands.spawn((
        PrimaryWindow,
        PersistentWindowBundle {
            window: Window {
                title: "SlingCraft".to_string(),
                canvas: Some("#bevy".to_string()),
                prevent_default_event_handling: false,
                fit_canvas_to_parent: true,
                ..default()
            },
            state: Persistent::<WindowState>::builder()
                .name("main_window")
                .format(StorageFormat::Toml)
                .path(state_directory.join("main-window.toml"))
                .default(WindowState::windowed(800, 600))
                .build()
                .expect("failed to initialize persistent window state"),
        },
    ));
}

#[derive(Component)]
struct Fill(Color32);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Radius(f32);

#[derive(Component)]
struct Mass(f32);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.insert_resource(PotentialEnergy(0.));
    commands.insert_resource(KineticEnergy(0.));
    commands.insert_resource(TotalEnergy(0.));
    commands.spawn((
        Radius(5.),
        Name::new("Gliblot"),
        Fill(Color32::RED),
        Transform::from_xyz(0., 0., 0.),
        Mass(10.),
        Velocity(Vec3::new(0., 0., 0.)),
    ));
    commands.spawn((
        Radius(2.),
        Name::new("Moon"),
        Fill(Color32::BLUE),
        Transform::from_xyz(20., 20., 0.),
        Mass(1.),
        Velocity(Vec3::new(-3., 3., 0.)),
    ));
    commands.spawn((
        Radius(1.),
        Name::new("Moon2"),
        Fill(Color32::GREEN),
        Transform::from_xyz(40., 40., 0.),
        Mass(0.5),
        Velocity(Vec3::new(1., -2., 0.)),
    ));
}

fn motion(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in &mut query {
        transform.translation += velocity.0 * time.delta_secs();
    }
}

#[hot]
fn gravity(
    bodies: Query<(Entity, &Radius, &Transform, &Mass)>,
    mut velocities: Query<&mut Velocity>,
    mut potential_energy: ResMut<PotentialEnergy>,
) {
    let mut velocity_updates = Vec::new();
    let mut new_potential_energy = 0.;

    for (entity1, radius1, transform1, mass1) in &bodies {
        let mut total_force = Vec3::ZERO;

        for (entity2, radius2, transform2, mass2) in &bodies {
            if entity1 != entity2 {
                // Calculate gravitational force
                let direction = transform2.translation - transform1.translation;
                let min_dist_sq = (radius1.0 + radius2.0).powi(2);
                let distance_sq = direction.length_squared().max(min_dist_sq); // Avoid division by zero
                let distance = distance_sq.sqrt();
                let mass_product = mass1.0 * mass2.0;
                let force_magnitude = mass_product / distance_sq;
                let potential = mass_product / distance;
                total_force += direction.normalize() * force_magnitude / mass1.0;
                new_potential_energy += potential;
            }
        }
        velocity_updates.push((entity1, total_force));
    }
    potential_energy.0 = new_potential_energy;

    for (entity, force) in velocity_updates {
        if let Ok(mut velocity) = velocities.get_mut(entity) {
            velocity.0 += force;
        }
    }
}

#[derive(Resource, Debug)]
struct PotentialEnergy(f32);

#[derive(Resource, Debug)]
struct KineticEnergy(f32);

#[derive(Resource, Debug)]
struct TotalEnergy(f32);

#[hot]
fn regulate_energy(
    bodies: Query<(&mut Velocity, &Mass)>,
    potential_energy: Res<PotentialEnergy>,
    mut kinetic_energy: ResMut<KineticEnergy>,
    mut total_energy: ResMut<TotalEnergy>,
) {
    let mut new_ke = 0.;
    for (velocity, mass) in bodies {
        let speed_sq = velocity.0.length_squared();
        new_ke += 0.5 * mass.0 * speed_sq;
    }
    kinetic_energy.0 = new_ke;
    total_energy.0 = kinetic_energy.0 - potential_energy.0;
}

#[hot]
fn ui_system(
    mut contexts: EguiContexts,
    bodies: Query<(&Name, &Radius, &Fill, &Transform)>,
    potential_energy: Res<PotentialEnergy>,
    kinetic_energy: Res<KineticEnergy>,
    total_energy: Res<TotalEnergy>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        MenuBar::new().ui(ui, |ui| {
            egui::widgets::global_theme_preference_buttons(ui);
        });
    });

    CentralPanel::default().show(ctx, |ui| {
        ui.label(format!(
            "PE: {:.03}, KE: {:.03}, Total: {:.03}",
            potential_energy.0, kinetic_energy.0, total_energy.0
        ));
        let plot_response = Plot::new("space_plot")
            .data_aspect(1.)
            .allow_axis_zoom_drag(false)
            .allow_boxed_zoom(false)
            .allow_scroll(false)
            .set_margin_fraction(vec2(0.2, 0.2))
            .cursor_color(Color32::TRANSPARENT)
            // .show_grid(false)
            .show_axes(false)
            .show_x(false)
            .show_y(false)
            .sense(Sense::all())
            .show(ui, |ui| {
                for (
                    name,
                    radius,
                    fill,
                    Transform {
                        translation: Vec3 { x, y, .. },
                        ..
                    },
                ) in bodies
                {
                    ui.polygon(
                        egui_plot::Polygon::new(
                            name,
                            (0..90)
                                .into_iter()
                                .map(|i| i * 4)
                                .map(|i| i as f32 * PI / 180.)
                                .map(|d| [radius.0 * d.cos(), radius.0 * d.sin()])
                                .map(|[x_edge, y_edge]| [x + x_edge, y + y_edge])
                                .map(|[x, y]| [x as f64, y as f64])
                                .collect::<Vec<_>>(),
                        )
                        .fill_color(fill.0.gamma_multiply(0.75))
                        .stroke(Stroke::new(2., fill.0.gamma_multiply(1.2))),
                    );
                }
            });
        if plot_response.response.clicked() {
            println!("hi");
        }
        if let Some(id) = plot_response.hovered_plot_item {
            println!("{id:?}");
        }
    });
}
