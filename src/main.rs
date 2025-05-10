mod geometry;

use crate::geometry::create_icosphere;
use avian3d::prelude::*;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;

#[derive(Component)]
struct Spinnable;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Dice"),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            WireframePlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, spin)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let depth = 5;
    let icosahedron = create_icosphere(depth)
        //.with_duplicated_vertices()
        .with_computed_normals();
    let sphere = Sphere::new(1.0).mesh().ico(depth as u32).expect("ico");

    info!("{}", icosahedron.count_vertices());
    info!("{}", sphere.count_vertices());

    commands.spawn((
        Spinnable,
        Wireframe,
        Mesh3d(meshes.add(icosahedron)),
        Transform::from_xyz(-1.2, 0.0, 0.0),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.8, 0.8))),
    ));

    commands.spawn((
        Spinnable,
        Wireframe,
        Mesh3d(meshes.add(sphere)),
        Transform::from_xyz(1.2, 0.0, 0.0),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.0, 0.8))),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spin(
    mut query: Query<&mut Transform, With<Spinnable>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut enabled: Local<bool>,
) {
    if keys.just_pressed(KeyCode::Space) {
        *enabled = !*enabled;
    }
    for mut transform in query.iter_mut() {
        if *enabled {
            transform.rotate_x(time.delta_secs() * 0.3);
            transform.rotate_y(time.delta_secs() * 0.7);
            transform.rotate_z(time.delta_secs() * -0.5);
        }
    }
}
