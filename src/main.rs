use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, Align2, CentralPanel, Color32, MenuBar, Sense, Stroke, TopBottomPanel, vec2},
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
            primary_window: None,
            ..default()
        }),
        EguiPlugin::default(),
        SimpleSubsecondPlugin::default(),
        PersistentWindowsPlugin,
    ))
    .add_systems(Startup, (setup, spawn_persistent_window).chain())
    .add_systems(
        PostStartup,
        (
            assign_ids,
            assign_masses,
            recalculate_orbital_velocities,
            assign_crafts,
        )
            .chain(),
    )
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
#[require(Mass, Crafts)]
struct Body;

#[derive(Component, Default)]
struct Mass(f32);

#[derive(Component, Default)]
struct Crafts(u32);

#[derive(Component)]
struct EguiId(egui::Id);

#[derive(Resource, Default)]
struct HoveredBody(Option<String>);

fn setup(mut commands: Commands) {
    const G: f32 = 50.0; // Same G as used in gravity function

    commands.spawn(Camera2d);
    commands.insert_resource(PotentialEnergy(0.));
    commands.insert_resource(KineticEnergy(0.));
    commands.insert_resource(TotalEnergy(0.));
    commands.insert_resource(CenterOfMass(Vec3::ZERO));
    commands.insert_resource(HoveredBody::default());

    // Central body (stationary)
    let gliblot_pos = Vec3::new(0., 0., 0.);
    let gliblot_radius = 5.0f32;
    // Calculate expected mass based on volume (will be recalculated in assign_masses)
    let gliblot_mass = (4.0 / 3.0) * PI * gliblot_radius.powi(3); // Density = 1.0
    commands.spawn((
        Body,
        Radius(gliblot_radius),
        Name::new("Gliblot"),
        Fill(Color32::RED),
        Transform::from_translation(gliblot_pos),
        Mass(gliblot_mass), // Override default with calculated mass for initial velocities
        Velocity(Vec3::ZERO),
    ));

    // Orbiting bodies - positions specified, velocities calculated
    let moon_pos = Vec3::new(20., 0., 0.);
    let moon_radius = 2.;
    let moon_distance = (moon_pos - gliblot_pos).length();
    let moon_orbital_speed = (G * gliblot_mass / moon_distance).sqrt();
    let moon_velocity = Vec3::new(0., moon_orbital_speed, 0.); // Tangent to orbit

    commands.spawn((
        Body,
        Radius(moon_radius),
        Name::new("Moon"),
        Fill(Color32::BLUE),
        Transform::from_translation(moon_pos),
        Velocity(moon_velocity),
        // Mass and Crafts will be added with defaults (0.0 and 0)
    ));

    let moon2_pos = Vec3::new(0., 40., 0.);
    let moon2_radius = 1.;
    let moon2_distance = (moon2_pos - gliblot_pos).length();
    let moon2_orbital_speed = (G * gliblot_mass / moon2_distance).sqrt();
    let moon2_velocity = Vec3::new(-moon2_orbital_speed, 0., 0.); // Tangent to orbit

    commands.spawn((
        Body,
        Radius(moon2_radius),
        Name::new("Moon2"),
        Fill(Color32::GREEN),
        Transform::from_translation(moon2_pos),
        Velocity(moon2_velocity),
        // Mass and Crafts will be added with defaults (0.0 and 0)
    ));
}

fn assign_ids(mut commands: Commands, bodies: Query<Entity, (With<Body>, Without<EguiId>)>) {
    for entity in bodies.iter() {
        // Create a unique egui Id from the entity
        commands
            .entity(entity)
            .insert(EguiId(egui::Id::new(entity)));
    }
}

fn assign_masses(mut bodies: Query<(&Radius, &mut Mass)>) {
    // Density constant (arbitrary units, adjust as needed for desired mass distribution)
    const DENSITY: f32 = 2.0e-2;

    // Mass = density * volume
    // For a sphere: volume = (4/3) * π * r³
    for (radius, mut mass) in bodies.iter_mut() {
        let volume = (4.0 / 3.0) * PI * radius.0.powi(3);
        mass.0 = DENSITY * volume;
    }
}

fn recalculate_orbital_velocities(mut bodies: Query<(&Transform, &Mass, &mut Velocity, &Name)>) {
    const G: f32 = 50.0; // Same G as used in gravity function

    // Find the central body (Gliblot - the one with the largest mass)
    let mut central_body: Option<(Vec3, f32)> = None;
    let mut max_mass = 0.0;

    for (transform, mass, _, _name) in bodies.iter() {
        if mass.0 > max_mass {
            max_mass = mass.0;
            central_body = Some((transform.translation, mass.0));
        }
    }

    let Some((central_pos, central_mass)) = central_body else {
        return;
    };

    // Set orbital velocities for all bodies except the central one
    for (transform, mass, mut velocity, _name) in bodies.iter_mut() {
        if mass.0 == max_mass {
            // This is the central body, keep it stationary
            velocity.0 = Vec3::ZERO;
        } else {
            // Calculate orbital velocity for this body
            let direction = transform.translation - central_pos;
            let distance = direction.length();

            if distance > 0.0 {
                let orbital_speed = (G * central_mass / distance).sqrt();
                // Velocity perpendicular to the radius vector
                let tangent = Vec3::new(-direction.y, direction.x, 0.0).normalize();
                velocity.0 = tangent * orbital_speed;
            }
        }
    }
}

fn assign_crafts(mut bodies: Query<(&Mass, &mut Crafts)>) {
    // Find the maximum mass among all bodies
    let max_mass = bodies
        .iter()
        .map(|(mass, _)| mass.0)
        .fold(0.0_f32, |max, mass| max.max(mass));

    if max_mass <= 0.0 {
        return;
    }

    // Assign crafts proportionally to mass (0 to 10 range)
    for (mass, mut crafts) in bodies.iter_mut() {
        crafts.0 = ((mass.0 / max_mass) * 10.0).round().max(0.0).min(10.0) as u32;
    }
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
    bodies: Query<(&Name, &Radius, &Fill, &Transform, &Crafts, Option<&EguiId>)>,
    potential_energy: Res<PotentialEnergy>,
    kinetic_energy: Res<KineticEnergy>,
    total_energy: Res<TotalEnergy>,
    cm: Res<CenterOfMass>,
    mut hovered_body: ResMut<HoveredBody>,
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
            // .legend(Legend::default().hidden_items([].into_iter()))
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
                    crafts,
                    egui_id,
                ) in bodies
                {
                    // Use entity-based ID as the polygon identifier string
                    let polygon_id = egui_id
                        .map(|id| format!("body_{:?}", id.0))
                        .unwrap_or_else(|| name.to_string());

                    // Create the circle points for the body
                    let body_points: Vec<_> = (0..90)
                        .into_iter()
                        .map(|i| i * 4)
                        .map(|i| i as f32 * PI / 180.)
                        .map(|d| [radius.0 * d.cos(), radius.0 * d.sin()])
                        .map(|[x_edge, y_edge]| [x + x_edge, y + y_edge])
                        .map(|[x, y]| [x as f64, y as f64])
                        .collect();

                    // Draw the main body polygon
                    ui.polygon(
                        egui_plot::Polygon::new(polygon_id.clone(), body_points.clone())
                            .name(name)
                            .fill_color(fill.0.gamma_multiply(0.75))
                            .stroke(Stroke::new(2., fill.0.gamma_multiply(1.2))),
                    );

                    let offset = (radius.0 / 2f32.sqrt() + 0.1) as f64;
                    ui.text(
                        egui_plot::Text::new(
                            "",
                            egui_plot::PlotPoint::new(*x as f64 + offset, *y as f64 + offset),
                            egui::RichText::new(crafts.0.to_string()).size(20.0), // .background_color(Color32::from_black_alpha(180)),
                        )
                        .color(Color32::WHITE)
                        .anchor(Align2::LEFT_BOTTOM),
                    );
                }

                ui.points(
                    egui_plot::Points::new("Center Mass", [cm.0.x as f64, cm.0.y as f64])
                        .color(Color32::WHITE)
                        .radius(3.),
                );
            });

        // Check for hover using geometric detection
        let mut new_hovered_body: Option<String> = None;

        if let Some(pointer_pos) = plot_response.response.hover_pos() {
            // Convert screen coordinates to plot coordinates
            let plot_pos = plot_response.transform.value_from_position(pointer_pos);
            // Check which body (if any) the pointer is over
            for (name, radius, _fill, transform, _crafts, _egui_id) in bodies.iter() {
                let body_center = [
                    transform.translation.x as f64,
                    transform.translation.y as f64,
                ];
                let distance = ((plot_pos.x - body_center[0]).powi(2)
                    + (plot_pos.y - body_center[1]).powi(2))
                .sqrt();

                if distance <= radius.0 as f64 {
                    new_hovered_body = Some(name.to_string());
                    break; // Take the first body we find (in case of overlap)
                }
            }
        }

        // Update hover state for next frame
        hovered_body.0 = new_hovered_body;

        // Draw hover outline in overlay if a body is hovered
        if let Some(hovered_name) = &hovered_body.0 {
            // Find the hovered body to get its position and radius
            if let Some((_, radius, _, transform, _, _)) = bodies
                .iter()
                .find(|(name, _, _, _, _, _)| &name.to_string() == hovered_name)
            {
                let body_center = [
                    transform.translation.x as f64,
                    transform.translation.y as f64,
                ];
                let hover_radius = radius.0 as f64 + 0.5; // Just slightly larger

                // Convert body center from plot coordinates to screen coordinates
                let center_screen =
                    plot_response
                        .transform
                        .position_from_point(&egui_plot::PlotPoint::new(
                            body_center[0],
                            body_center[1],
                        ));

                // Calculate the radius in screen space by checking a point on the edge
                let edge_point =
                    egui_plot::PlotPoint::new(body_center[0] + hover_radius, body_center[1]);
                let edge_screen = plot_response.transform.position_from_point(&edge_point);
                let screen_radius = (edge_screen.x - center_screen.x).abs();

                // Draw circle outline on the main UI
                let painter = ui.painter();
                painter.circle_stroke(
                    center_screen,
                    screen_radius,
                    Stroke::new(3.0, Color32::WHITE),
                );
            }
        }

        // Add overlay card in lower left corner with margins
        let margin = 8.0;

        // Create a smaller rect with margins
        let card_rect = plot_response.response.rect.shrink(margin);
        let card_rect = card_rect
            .with_max_x(card_rect.right() + 600.)
            .with_min_y(card_rect.bottom() - 600.);

        let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(card_rect));
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            let hovered_name = hovered_body.0.clone();

            // Show card with hovered body name or default text
            egui::Frame::new()
                .fill(Color32::from_black_alpha(200))
                .corner_radius(8.0)
                .stroke(ui.style().visuals.window_stroke())
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color = Some(Color32::WHITE);
                    if let Some(name) = hovered_name {
                        ui.label(format!("Hovered: {}", name));
                    } else {
                        ui.label("SlingCraft v0.1.0\nOrbital Mechanics Simulator");
                    }
                });
        });
    });
}
