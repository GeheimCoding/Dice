mod geometry;

use crate::geometry::create_d6;
use avian3d::prelude::*;
use bevy::image::ImageLoaderSettings;
use bevy::input::common_conditions::{input_just_pressed, input_toggle_active};
use bevy::pbr::PointLightShadowMap;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::Rng;

#[derive(Component)]
struct Spinnable(Vec3);

#[derive(Component)]
struct Die;

#[derive(Component)]
struct Cup;

#[derive(Component)]
struct Ground;

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
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        ))
        //.insert_resource(DeactivationTime(0.2))
        .insert_resource(PointLightShadowMap { size: 2048 })
        .add_systems(Startup, (setup, spawn_cube).chain())
        .add_systems(PreUpdate, handle_asset_events)
        .add_systems(
            Update,
            (
                spin,
                move_cup,
                follow_mouse,
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
        Ground,
        RigidBody::Static,
        Collider::cylinder(5.0, 0.2),
        Mesh3d(meshes.add(Cylinder::new(5.0, 0.2))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
    let d6 = create_d6(4, 0.72, 0.6);
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
            intensity: 2000000.0,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 8.0),
    ));
    commands.spawn((
        Msaa::Sample8,
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));

    let cup = asset_server.load(GltfAssetLabel::Scene(0).from_asset("Cup.glb#Scene0"));
    commands.spawn((
        Cup,
        SceneRoot(cup),
        RigidBody::Kinematic,
        Name::new("Cup"),
        Transform::from_translation(Vec3::new(2.0, 1.2, 0.0)),
    ));
}

fn spin(
    mut query: Query<(&mut AngularVelocity, &Spinnable)>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut enabled: Local<bool>,
) {
    if keys.just_pressed(KeyCode::Space) {
        *enabled = !*enabled;
    }
    for (mut rotation, spinnable) in query.iter_mut() {
        if *enabled {
            rotation.x = time.delta_secs() * spinnable.0.x;
            rotation.y = time.delta_secs() * spinnable.0.y;
            rotation.z = time.delta_secs() * spinnable.0.z;
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
    let _color = Color::srgb(
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
        Spinnable(spin * 600.0),
        RigidBody::Dynamic,
        GravityScale(10.0),
        LinearDamping(2.0),
        AngularDamping(0.5),
        TransformInterpolation,
        Restitution::new(0.4),
        AngularVelocity(angular_velocity * 8.0),
        Mesh3d(d6.mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            normal_map_texture: Some(d6.normal_texture.clone()),
            base_color_texture: Some(d6.color_texture.clone()),
            depth_map: Some(d6.depth_texture.clone()),
            parallax_depth_scale: 0.008,
            perceptual_roughness: 0.8,
            //base_color: color,
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

fn move_cup(
    mut query: Query<(&mut LinearVelocity, &mut AngularVelocity), With<Cup>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (mut linear, mut angular) = query.single_mut().expect("cup");
    let movement_speed = 400.0;
    let rotation_speed = 300.0;

    **angular = Vec3::ZERO;

    if input.pressed(KeyCode::KeyA) {
        linear.x -= movement_speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyD) {
        linear.x += movement_speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyW) {
        linear.z -= movement_speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyS) {
        linear.z += movement_speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyQ) {
        linear.y -= movement_speed * time.delta_secs();
    }
    if input.pressed(KeyCode::KeyE) {
        linear.y += movement_speed * time.delta_secs();
    }

    if input.pressed(KeyCode::ArrowLeft) {
        angular.x -= rotation_speed * time.delta_secs();
    }
    if input.pressed(KeyCode::ArrowRight) {
        angular.x += rotation_speed * time.delta_secs();
    }
    if input.pressed(KeyCode::ArrowUp) {
        angular.z -= rotation_speed * time.delta_secs();
    }
    if input.pressed(KeyCode::ArrowDown) {
        angular.z += rotation_speed * time.delta_secs();
    }
}

fn follow_mouse(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    mut linear_velocity: Single<(&mut LinearVelocity, &Transform), With<Cup>>,
) {
    linear_velocity.0.0 = Vec3::ZERO;
    let (camera, camera_transform) = *camera;
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    let Some(distance) =
        ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);
    let translation = linear_velocity.1.translation;
    let target_point = Vec3::new(point.x, translation.y, point.z);

    let max = Vec3::ONE * 1000.0;
    let move_towards = target_point - translation;
    let distance = translation.distance(target_point);
    linear_velocity.0.0 = (move_towards * distance * 8.0).clamp(-max, max);
}

// TODO compare with: https://docs.rs/avian3d/latest/avian3d/collision/collider/struct.ColliderConstructorHierarchy.html
fn handle_asset_events(
    mut commands: Commands,
    query: Query<(Entity, &SceneRoot), With<RigidBody>>,
    mut events: EventReader<AssetEvent<Scene>>,
    mut scenes: ResMut<Assets<Scene>>,
    meshes: Res<Assets<Mesh>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for (entity, scene_root) in query.iter() {
                if scene_root.0.id() == *id {
                    let scene = scenes.get_mut(*id).expect("scene");
                    let mut query = scene.world.query::<(&Mesh3d, Option<&Name>)>();
                    for (mesh, name) in query.iter(&mut scene.world) {
                        info!("{:?}", name);
                        if let Some(name) = name {
                            if !name.starts_with("Cylinder") {
                                continue;
                            }
                        } else {
                            continue;
                        }
                        let mesh = meshes.get(mesh.0.id()).expect("mesh");
                        let collider = Collider::convex_decomposition_from_mesh_with_config(
                            mesh,
                            &VhacdParameters {
                                concavity: 0.0,
                                convex_hull_approximation: false,
                                ..default()
                            },
                        )
                        .expect("convex decomposition");
                        commands.entity(entity).with_child(collider);
                    }
                }
            }
        }
    }
}
