mod geometry;

use crate::geometry::{create_icosphere, intersect_mesh_with_plane};
use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::input::common_conditions::input_just_pressed;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use rand::Rng;
use std::fmt::Error;

#[derive(Component)]
struct Spinnable;

#[derive(Component)]
struct Marker;

#[derive(Resource)]
struct TriangleMesh(Handle<Mesh>);

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

    commands.spawn((
        Spinnable,
        Wireframe,
        Visibility::Hidden,
        Mesh3d(meshes.add(icosphere)),
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

    let vertices = vec![[-1.0, -0.5, 0.0], [1.0, -0.5, 0.0], [0.0, 1.0, 0.0]];
    let indices = Indices::U16(vec![0, 1, 2]);
    let mesh = meshes.add(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_indices(indices)
        .with_computed_normals(),
    );
    commands.insert_resource(TriangleMesh(mesh.clone()));
    commands.spawn((Wireframe, Mesh3d(mesh)));

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
    triangle: Res<TriangleMesh>,
) -> Result {
    query.iter().for_each(|e| commands.entity(e).despawn());

    let mut rng = rand::rng();
    let plane_point = Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        0.0,
    );
    let plane_normal = Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        0.0,
    )
    .normalize();

    let triangle = meshes.get(&triangle.0).ok_or(Error::default())?;
    let triangulation = intersect_mesh_with_plane(triangle.clone(), plane_point, plane_normal)?;
    let vertices = Vec::from(
        triangulation
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .ok_or(Error::default())?,
    );
    let indices = Vec::from_iter(triangulation.indices().expect("indices").iter());

    let colors = vec![
        Color::srgb(0.2, 0.2, 0.4),
        Color::srgb(0.4, 0.2, 0.4),
        Color::srgb(0.4, 0.4, 0.2),
    ];
    for i in (0..indices.len()).step_by(3) {
        let indices = vec![
            indices[i] as u16,
            indices[i + 1] as u16,
            indices[i + 2] as u16,
        ];
        let color = colors[i / 3];
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone())
        .with_inserted_indices(Indices::U16(indices))
        .with_duplicated_vertices()
        .with_computed_normals();

        commands.spawn((
            Marker,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(color)),
        ));
    }
    commands.spawn((
        Marker,
        Mesh3d(meshes.add(Sphere::new(0.05))),
        Transform::from_translation(plane_point),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
    let line = vec![[0.0, 0.0, 0.0], (plane_normal * 0.2).to_array()];
    let indices = Indices::U16(vec![0, 1]);
    let mesh = meshes.add(
        Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::Z; line.len()])
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, line)
            .with_inserted_indices(indices),
    );
    commands.spawn((
        Marker,
        Mesh3d(mesh),
        Transform::from_translation(plane_point.with_z(0.01)),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    for vertex in vertices {
        commands.spawn((
            Marker,
            Mesh3d(meshes.add(Sphere::new(0.03))),
            Transform::from_translation(Vec3::from_array(vertex)),
            MeshMaterial3d(materials.add(Color::BLACK)),
        ));
    }
    Ok(())
}
