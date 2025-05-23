mod geometry;

use crate::geometry::{create_icosphere, intersect_mesh_with_plane, remove_if};
use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::input::common_conditions::input_just_pressed;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use std::fmt::Error;

#[derive(Component)]
struct Spinnable;

#[derive(Component)]
struct Marker;

#[derive(Resource)]
struct SphereMesh(Handle<Mesh>);

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
        .add_systems(
            Update,
            collide_and_mark.run_if(input_just_pressed(KeyCode::Enter)),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let depth = 5;
    let icosphere = create_icosphere(depth)
        //.with_duplicated_vertices()
        .with_computed_normals();
    let sphere = Sphere::new(1.0).mesh().ico(depth as u32).expect("ico");

    info!("{}", icosphere.count_vertices());
    info!("{}", sphere.count_vertices());

    let icosphere = meshes.add(icosphere);
    commands.insert_resource(SphereMesh(icosphere.clone()));

    commands.spawn((
        Spinnable,
        Wireframe,
        Visibility::Hidden,
        Mesh3d(icosphere),
        Transform::from_xyz(-1.2, 0.0, 0.0),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.8, 0.8))),
    ));

    commands.spawn((
        Spinnable,
        Wireframe,
        Visibility::Hidden,
        Mesh3d(meshes.add(sphere)),
        Transform::from_xyz(1.2, 0.0, 0.0),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.0, 0.8))),
    ));

    let vertices = vec![
        [-1.0, -0.5, 0.0],
        [1.0, -0.5, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
    ];
    let indices = Indices::U16(vec![0, 1, 2, 3, 1, 0]);
    let mesh = meshes.add(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_indices(indices)
        .with_computed_normals(),
    );
    commands.spawn((Visibility::Hidden, Wireframe, Mesh3d(mesh)));

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

fn collide_and_mark(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<Entity, With<Marker>>,
    triangle: Res<SphereMesh>,
) -> Result {
    query.iter().for_each(|e| commands.entity(e).despawn());

    const THRESHOLD: f32 = 0.75;
    let mut die = meshes.get(&triangle.0).ok_or(Error::default())?.clone();
    let normals = vec![-Vec3::X, Vec3::X, -Vec3::Y, Vec3::Y, -Vec3::Z, Vec3::Z];
    for normal in normals {
        die = intersect_mesh_with_plane(die, normal * THRESHOLD, normal)?;
    }
    let mesh = remove_if(die, |vertex| vertex.iter().any(|c| c.abs() > THRESHOLD));
    commands.spawn((
        Marker,
        Spinnable,
        Mesh3d(meshes.add(mesh.with_computed_normals())),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.8, 0.8))),
    ));
    Ok(())
}
