mod geometry;

use crate::geometry::create_icosphere;
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
    let mut rng = rand::rng();
    let normal = Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        0.0,
    )
    .normalize();

    let vertices = vec![[0.0, 0.0, 0.0], (normal * 0.2).to_array()];
    let indices = Indices::U16(vec![0, 1]);
    let mesh = meshes.add(
        Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::Z; vertices.len()])
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_indices(indices),
    );
    query.iter().for_each(|e| commands.entity(e).despawn());
    commands.spawn((
        Marker,
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    let triangle = meshes.get(&triangle.0).ok_or(Error::default())?;
    let vertices = Vec::from(
        triangle
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .ok_or(Error::default())?,
    );
    let indices = Vec::from_iter(triangle.indices().expect("indices").iter());

    let intersections = intersect_triangle_with_plane(vertices, indices, Vec3::default(), normal);
    for intersection in intersections {
        commands.spawn((
            Marker,
            Mesh3d(meshes.add(Sphere::new(0.05))),
            Transform::from_translation(intersection),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.8))),
        ));
    }
    Ok(())
}

fn intersect_triangle_with_plane(
    vertices: Vec<[f32; 3]>,
    indices: Vec<usize>,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Vec<Vec3> {
    let mut lines = Vec::new();
    for index in 0..indices.len() {
        let line = (
            Vec3::from_array(vertices[indices[index]]),
            Vec3::from_array(vertices[indices[(index + 1) % indices.len()]]),
        );
        lines.push(line);
    }
    let mut collisions = Vec::new();
    for line in lines {
        if let Some(intersection) = intersect_line_with_plane(line, plane_point, plane_normal) {
            collisions.push(intersection);
        }
    }
    collisions
}

fn intersect_line_with_plane(
    (l1, l2): (Vec3, Vec3),
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Option<Vec3> {
    let line = l2 - l1;
    let dot = plane_normal.dot(line);
    if dot.abs() <= f32::EPSILON {
        return None;
    }
    let factor = plane_normal.dot(l1 - plane_point) / -dot;
    if factor < 0.0 || factor > 1.0 {
        return None;
    }
    Some(l1 + line * factor)
}
