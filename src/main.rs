mod geometry;

use crate::geometry::create_d6;
use avian3d::math::Vector;
use avian3d::prelude::*;
use bevy::color::palettes::css::{ORANGE, RED};
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

#[derive(Component)]
struct Counted;

#[derive(Component)]
struct Cursor;

#[derive(Resource)]
struct DebugRenderEnabled(bool);

#[derive(Component, Default)]
struct AutoSleep {
    translation: Vec3,
    rotation: Vec3,
    time: f32,
}

#[derive(Component)]
struct Roll {
    faces: Vec<u8>,
}

#[derive(Resource)]
struct CountDie(bool);

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
            PhysicsDebugPlugin::default(),
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        ))
        .insert_gizmo_config(
            PhysicsGizmos::default(),
            GizmoConfig {
                enabled: false,
                ..default()
            },
        )
        .insert_resource(DebugRenderEnabled(false))
        .insert_resource(CountDie(false))
        //.insert_resource(DeactivationTime(0.2))
        .insert_resource(PointLightShadowMap { size: 2048 })
        .add_systems(Startup, (setup, spawn_cube).chain())
        .add_systems(
            Update,
            (
                spin,
                count_faces,
                detect_sleep,
                position_cursor,
                move_cup_with_mouse,
                highlight_selected_die,
                roll_cup_towards_center,
                spawn_cube.run_if(input_just_pressed(KeyCode::Enter)),
                toggle_debug_render.run_if(input_just_pressed(KeyCode::Escape)),
            ),
        )
        .add_systems(
            PostUpdate,
            (
                // scene spawning happens between Update and PostUpdate
                handle_asset_events,
                despawn_fallen_dice,
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
        Collider::cylinder(6.0, 0.2),
        Mesh3d(meshes.add(Cylinder::new(6.0, 0.2))),
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
    let camera_transform = Transform::from_xyz(-2.5, 7.0, 13.0).looking_at(Vec3::ZERO, Dir3::Y);
    commands.spawn((Msaa::Sample8, Camera3d::default(), camera_transform.clone()));

    let cup = asset_server.load(GltfAssetLabel::Scene(0).from_asset("Cup mit col.glb"));
    commands.spawn((
        Cup,
        SceneRoot(cup),
        RigidBody::Kinematic,
        Name::new("Cup"),
        Transform::from_translation(Vec3::new(2.0, 1.2, 0.0)),
    ));

    commands.spawn((
        Roll { faces: vec![] },
        Text::new("Roll:"),
        TextFont {
            font_size: 60.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));
    commands.spawn((
        Cursor,
        RayCaster::new(camera_transform.translation, camera_transform.forward()),
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
        AutoSleep::default(),
        Spinnable(spin * 800.0),
        RigidBody::Dynamic,
        GravityScale(20.0),
        // this causes the dice to clip outside the cup, which looks awful
        //TransformInterpolation,
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

fn clear_dice(
    mut commands: Commands,
    mut roll: Single<(&mut Roll, &mut Text)>,
    query: Query<Entity, With<Die>>,
    mut count_die: ResMut<CountDie>,
) {
    count_die.0 = false;
    roll.0.faces.clear();
    roll.1.0 = String::from("Roll:");
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn count_faces(
    mut commands: Commands,
    count_die: Res<CountDie>,
    mut roll: Single<(&mut Roll, &mut Text)>,
    query: Query<(Entity, &Transform), (With<Die>, Added<Sleeping>, Without<Counted>)>,
) {
    if !count_die.0 {
        return;
    }
    for (entity, transform) in query.iter() {
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
            roll.0.faces.push(*face);
            //roll.0.faces.sort();
            roll.1.0 = format!(
                "Roll: {}",
                roll.0
                    .faces
                    .iter()
                    .map(|face| face.to_string())
                    .collect::<Vec<_>>()
                    .join(" + ")
            );
        }
        commands.entity(entity).insert(Counted);
    }
}

fn move_cup_with_mouse(
    time: Res<Time>,
    window: Single<&Window>,
    input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    mut linear_velocity: Single<(&mut LinearVelocity, &Transform), With<Cup>>,
) {
    let movement_speed = 400.0 * time.delta_secs();
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

    if input.pressed(MouseButton::Right) {
        linear_velocity.0.0.y -= movement_speed;
    }
    if input.pressed(MouseButton::Left) {
        linear_velocity.0.0.y += movement_speed;
    }
}

fn roll_cup_towards_center(
    mut count_die: ResMut<CountDie>,
    input: Res<ButtonInput<KeyCode>>,
    ground: Single<&GlobalTransform, With<Ground>>,
    mut angular_velocity: Single<(&mut AngularVelocity, &Transform), With<Cup>>,
) {
    let center = ground.translation();
    let direction = (center - angular_velocity.1.translation).normalize();
    let target_up = if input.pressed(KeyCode::KeyR) {
        count_die.0 = true;
        direction
    } else {
        Vec3::Y
    };

    **angular_velocity.0 =
        Quat::from_rotation_arc(*angular_velocity.1.up(), target_up).to_scaled_axis() * 4.0;
}

fn detect_sleep(
    mut commands: Commands,
    deactivation_time: Res<DeactivationTime>,
    mut query: Query<(Entity, &Transform, &mut AutoSleep, &GravityScale), Without<Sleeping>>,
    time: Res<Time>,
) {
    for (entity, transform, mut auto_sleep, gravity_scale) in query.iter_mut() {
        let translation = transform.translation;
        let rotation = transform.rotation.to_scaled_axis();
        let changed = (auto_sleep.translation - translation).length()
            + (auto_sleep.rotation - rotation).length();
        if changed > 0.1 {
            if gravity_scale.0 != 20.0 {
                commands.entity(entity).insert(GravityScale(20.0));
                commands.entity(entity).insert(LinearDamping::default());
                commands.entity(entity).insert(AngularDamping::default());
            }
            auto_sleep.translation = translation;
            auto_sleep.rotation = rotation;
            auto_sleep.time = 0.0;
        } else {
            auto_sleep.time += time.delta_secs();
        }
        if auto_sleep.time > deactivation_time.0 {
            //info!("auto sleeping for {entity}");
            commands.entity(entity).insert(Sleeping);
            if gravity_scale.0 != 1.0 {
                commands.entity(entity).insert(GravityScale(1.0));
                commands.entity(entity).insert(LinearDamping(100000.0));
                commands.entity(entity).insert(AngularDamping(100000.0));
            }
        }
    }
}

fn despawn_fallen_dice(mut commands: Commands, query: Query<(Entity, &Transform), With<Die>>) {
    for (die, transform) in query.iter() {
        if transform.translation.y < -100.0 {
            commands.entity(die).despawn();
        }
    }
}

fn position_cursor(
    mut query: Query<&mut RayCaster, With<Cursor>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = *camera;
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    let mut cursor = query.single_mut().expect("cursor");
    cursor.origin = ray.origin;
    cursor.direction = ray.direction;
}

fn highlight_selected_die(
    dice: Query<&MeshMaterial3d<StandardMaterial>, With<Die>>,
    cursor: Query<(&RayCaster, &RayHits), With<Cursor>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for mesh_material in dice.iter() {
        let mesh_material = materials.get_mut(mesh_material).expect("mesh_material");
        mesh_material.base_color = Color::default();
    }
    for (_ray, hits) in cursor.iter() {
        for hit in hits.iter_sorted() {
            if let Ok(mesh_material) = dice.get(hit.entity) {
                let mesh_material = materials.get_mut(mesh_material).expect("mesh_material");
                mesh_material.base_color = Color::srgb(0.0, 0.8, 0.8);
            }
            break;
        }
    }
}

fn toggle_debug_render(
    mut gizmo_config: ResMut<GizmoConfigStore>,
    mut flag: ResMut<DebugRenderEnabled>,
) {
    flag.0 = !flag.0;
    gizmo_config.insert(
        GizmoConfig {
            enabled: flag.0,
            ..default()
        },
        PhysicsGizmos {
            contact_point_color: Some(RED.into()),
            axis_lengths: Some(Vector::new(0.5, 0.5, 0.5)),
            aabb_color: Some(Color::srgb(0.8, 0.8, 0.8)),
            collider_color: Some(ORANGE.into()),
            sleeping_color_multiplier: Some([1.0, 1.0, 0.4, 1.0]),
            hide_meshes: true,
            ..default()
        },
    );
}

// TODO compare with: https://docs.rs/avian3d/latest/avian3d/collision/collider/struct.ColliderConstructorHierarchy.html
fn handle_asset_events(
    mut commands: Commands,
    query: Query<(Entity, &SceneRoot), With<RigidBody>>,
    children: Query<&Children>,
    mesh_query: Query<(&Mesh3d, &Name)>,
    mut events: EventReader<AssetEvent<Scene>>,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for (entity, scene_root) in query.iter() {
                if scene_root.0.id() == *id {
                    for entity in children.iter_descendants(entity) {
                        let Ok((mesh, name)) = mesh_query.get(entity) else {
                            continue;
                        };
                        //info!("{:?}", name);
                        if name.starts_with("Cube") {
                            commands.entity(entity).insert(Visibility::Hidden);
                        } else {
                            commands.entity(entity).insert(MeshMaterial3d(materials.add(
                                StandardMaterial {
                                    base_color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                                    alpha_mode: AlphaMode::Add,
                                    ..default()
                                },
                            )));
                        }
                        if !name.starts_with("Cylinder.001") {
                            continue;
                        }
                        commands.entity(entity).insert(Visibility::Hidden);
                        let mesh = meshes.get(mesh.0.id()).expect("mesh");
                        let collider = Collider::convex_decomposition_from_mesh_with_config(
                            mesh,
                            &VhacdParameters {
                                fill_mode: FillMode::SurfaceOnly,
                                ..default()
                            },
                        )
                        .expect("collider");
                        commands.entity(entity).with_child(collider);
                    }
                }
            }
        }
    }
}
