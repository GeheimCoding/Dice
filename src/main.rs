mod geometry;

use crate::geometry::create_d6;
use avian3d::prelude::*;
use bevy::image::ImageLoaderSettings;
use bevy::input::common_conditions::input_just_pressed;
use bevy::pbr::PointLightShadowMap;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Spinnable(Vec3);

#[derive(Component)]
struct Die;

#[derive(Resource)]
struct D6 {
    mesh: Handle<Mesh>,
    collider: Collider,
    color_texture: Handle<Image>,
    depth_texture: Handle<Image>,
    normal_texture: Handle<Image>,
}

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
        ))
        .insert_resource(PointLightShadowMap { size: 2048 })
        .add_systems(Startup, (setup, spawn_cube).chain())
        .add_systems(
            Update,
            (
                spin,
                spawn_cube.run_if(input_just_pressed(KeyCode::Enter)),
                count_faces.run_if(input_just_pressed(KeyCode::KeyC)),
                clear_dice.run_if(input_just_pressed(KeyCode::Backspace)),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(5.0, 0.1),
        Mesh3d(meshes.add(Cylinder::new(5.0, 0.1))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
    let d6 = create_d6(4, 0.72, 0.8);
    let collider = Collider::convex_decomposition_from_mesh_with_config(
        &d6,
        &VhacdParameters {
            fill_mode: FillMode::SurfaceOnly,
            ..default()
        },
    )
    .expect("collider");
    commands.insert_resource(D6 {
        mesh: meshes.add(d6),
        collider,
        color_texture: asset_server.load("d6.png"),
        depth_texture: asset_server.load("d6_depth.png"),
        normal_texture: asset_server
            .load_with_settings("d6_normal.png", |settings: &mut ImageLoaderSettings| {
                settings.is_srgb = false
            }),
    });
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        Msaa::Sample8,
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));
}

fn spin(
    mut query: Query<(&mut Transform, &Spinnable)>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut enabled: Local<bool>,
) {
    if keys.just_pressed(KeyCode::Space) {
        *enabled = !*enabled;
    }
    for (mut transform, spinnable) in query.iter_mut() {
        if *enabled {
            transform.rotate_x(time.delta_secs() * spinnable.0.x);
            transform.rotate_y(time.delta_secs() * spinnable.0.y);
            transform.rotate_z(time.delta_secs() * spinnable.0.z);
        }
    }
}

fn spawn_cube(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    d6: Res<D6>,
) {
    let mut rng = rand::rng();
    let angular_velocity = Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
    );
    let color = Color::srgb(
        rng.random_range(0.0..1.0),
        rng.random_range(0.0..1.0),
        rng.random_range(0.0..1.0),
    );
    let spin = Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
    );
    commands.spawn((
        Die,
        Spinnable(spin * 10.0),
        RigidBody::Dynamic,
        GravityScale(10.0),
        LinearDamping(2.0),
        AngularDamping(0.5),
        Restitution::new(0.4),
        AngularVelocity(angular_velocity * 8.0),
        Mesh3d(d6.mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            normal_map_texture: Some(d6.normal_texture.clone()),
            base_color_texture: Some(d6.color_texture.clone()),
            depth_map: Some(d6.depth_texture.clone()),
            parallax_depth_scale: 0.008,
            perceptual_roughness: 0.8,
            base_color: color,
            ..default()
        })),
        d6.collider.clone(),
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));
}

fn clear_dice(mut commands: Commands, query: Query<Entity, With<Die>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn count_faces(query: Query<&Transform, With<Die>>) {
    let mut total_count = 0;
    for transform in query.iter() {
        let sides = vec![
            (transform.left(), 2),
            (transform.right(), 5),
            (transform.up(), 6),
            (transform.down(), 1),
            (transform.forward(), 3),
            (transform.back(), 4),
        ];
        if let Some((_, face)) = sides.iter().max_by(|lhs, rhs| {
            lhs.0
                .dot(Vec3::Y)
                .partial_cmp(&rhs.0.dot(Vec3::Y))
                .expect("comparable")
        }) {
            total_count += face;
        }
    }
    info!("{total_count}");
}
