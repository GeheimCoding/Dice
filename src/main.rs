mod geometry;

use crate::geometry::create_d6;
use avian3d::prelude::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::pbr::PointLightShadowMap;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Spinnable(Vec3);

#[derive(Component)]
struct Die;

#[derive(Resource)]
struct D6((Handle<Mesh>, Collider));

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
                clear_dice.run_if(input_just_pressed(KeyCode::Backspace)),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    commands.insert_resource(D6((meshes.add(d6), collider)));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
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
    d6_mesh: Res<D6>,
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
        d6_mesh.0.1.clone(),
        Transform::from_xyz(0.0, 4.0, 0.0),
        AngularVelocity(angular_velocity * 8.0),
        Mesh3d(d6_mesh.0.0.clone()),
        MeshMaterial3d(materials.add(color)),
    ));
}

fn clear_dice(mut commands: Commands, query: Query<Entity, With<Die>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
