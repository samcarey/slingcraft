use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, CentralPanel, Color32, MenuBar, Stroke, TopBottomPanel, vec2},
};
use bevy_persistent::prelude::*;
use bevy_persistent_windows::prelude::*;
use bevy_simple_subsecond_system::prelude::*;
use egui_plot::{Legend, Plot};
use std::f32::consts::PI;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            ..default()
        }),
        EguiPlugin::default(),
        SimpleSubsecondPlugin::default(),
        PersistentWindowsPlugin,
    ))
    .add_systems(Startup, (setup, spawn_persistent_window))
    .add_systems(EguiPrimaryContextPass, ui_system)
    .add_systems(
        Update,
        (gravity, motion, regulate_energy, calculate_center_of_mass),
    );

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
    const G: f32 = 50.0; // Same G as used in gravity function

    commands.spawn(Camera2d);
    commands.insert_resource(PotentialEnergy(0.));
    commands.insert_resource(KineticEnergy(0.));
    commands.insert_resource(TotalEnergy(0.));
    commands.insert_resource(CenterOfMass(Vec3::ZERO));

    // Central body (stationary)
    let gliblot_pos = Vec3::new(0., 0., 0.);
    let gliblot_mass = 10.;
    commands.spawn((
        Radius(5.),
        Name::new("Gliblot"),
        Fill(Color32::RED),
        Transform::from_translation(gliblot_pos),
        Mass(gliblot_mass),
        Velocity(Vec3::ZERO),
    ));

    // Orbiting bodies - positions specified, velocities calculated
    let moon_pos = Vec3::new(20., 0., 0.);
    let moon_mass = 1.;
    let moon_distance = (moon_pos - gliblot_pos).length();
    let moon_orbital_speed = (G * gliblot_mass / moon_distance).sqrt();
    let moon_velocity = Vec3::new(0., moon_orbital_speed, 0.); // Tangent to orbit

    commands.spawn((
        Radius(2.),
        Name::new("Moon"),
        Fill(Color32::BLUE),
        Transform::from_translation(moon_pos),
        Mass(moon_mass),
        Velocity(moon_velocity),
    ));

    let moon2_pos = Vec3::new(0., 40., 0.);
    let moon2_mass = 0.5;
    let moon2_distance = (moon2_pos - gliblot_pos).length();
    let moon2_orbital_speed = (G * gliblot_mass / moon2_distance).sqrt();
    let moon2_velocity = Vec3::new(-moon2_orbital_speed, 0., 0.); // Tangent to orbit

    commands.spawn((
        Radius(1.),
        Name::new("Moon2"),
        Fill(Color32::GREEN),
        Transform::from_translation(moon2_pos),
        Mass(moon2_mass),
        Velocity(moon2_velocity),
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
    time: Res<Time>,
) {
    const G: f32 = 50.0; // Gravitational constant (adjusted for better energy balance)

    let mut velocity_updates = Vec::new();
    let mut new_potential_energy = 0.;
    let bodies_vec: Vec<_> = bodies.iter().collect();

    for (entity1, radius1, transform1, _mass1) in &bodies {
        let mut total_acceleration = Vec3::ZERO;

        for (entity2, radius2, transform2, mass2) in &bodies {
            if entity1 != entity2 {
                // Calculate gravitational acceleration: a = G * m2 / r²
                let direction = transform2.translation - transform1.translation;
                let min_dist_sq = (radius1.0 + radius2.0).powi(2);
                let distance_sq = direction.length_squared().max(min_dist_sq); // Avoid division by zero
                let acceleration_magnitude = G * mass2.0 / distance_sq;
                total_acceleration += direction.normalize() * acceleration_magnitude;
            }
        }
        velocity_updates.push((entity1, total_acceleration));
    }

    // Calculate potential energy (avoid double counting by only considering i < j pairs)
    for i in 0..bodies_vec.len() {
        for j in (i + 1)..bodies_vec.len() {
            let (_, radius1, transform1, mass1) = bodies_vec[i];
            let (_, radius2, transform2, mass2) = bodies_vec[j];

            let direction = transform2.translation - transform1.translation;
            let min_dist_sq = (radius1.0 + radius2.0).powi(2);
            let distance_sq = direction.length_squared().max(min_dist_sq);
            let distance = distance_sq.sqrt();
            let mass_product = mass1.0 * mass2.0;

            // Gravitational potential energy: U = -G * m1 * m2 / r
            new_potential_energy += -G * mass_product / distance;
        }
    }
    potential_energy.0 = new_potential_energy;

    for (entity, acceleration) in velocity_updates {
        if let Ok(mut velocity) = velocities.get_mut(entity) {
            velocity.0 += acceleration * time.delta_secs();
        }
    }
}

#[derive(Resource, Debug)]
struct PotentialEnergy(f32);

#[derive(Resource, Debug)]
struct KineticEnergy(f32);

#[derive(Resource, Debug)]
struct TotalEnergy(f32);

#[derive(Resource, Debug)]
struct CenterOfMass(Vec3);

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
    total_energy.0 = kinetic_energy.0 + potential_energy.0;
}

fn calculate_center_of_mass(
    bodies: Query<(&Transform, &Mass)>,
    mut center_of_mass: ResMut<CenterOfMass>,
) {
    let mut total_mass = 0.0;
    let mut weighted_position = Vec3::ZERO;

    for (transform, mass) in bodies.iter() {
        weighted_position += transform.translation * mass.0;
        total_mass += mass.0;
    }

    if total_mass > 0.0 {
        center_of_mass.0 = weighted_position / total_mass;
    } else {
        center_of_mass.0 = Vec3::ZERO;
    }
}

#[hot]
fn ui_system(
    mut contexts: EguiContexts,
    bodies: Query<(&Name, &Radius, &Fill, &Transform)>,
    potential_energy: Res<PotentialEnergy>,
    kinetic_energy: Res<KineticEnergy>,
    total_energy: Res<TotalEnergy>,
    cm: Res<CenterOfMass>,
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
            .legend(Legend::default())
            // .sense(Sense::all())
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

                ui.points(
                    egui_plot::Points::new("Center Mass", [cm.0.x as f64, cm.0.y as f64])
                        .color(Color32::WHITE)
                        .radius(2.),
                );
            });
        if plot_response.response.clicked() {
            println!("hi");
        }
        if let Some(id) = plot_response.hovered_plot_item {
            println!("{id:?}");
        }
    });
}
