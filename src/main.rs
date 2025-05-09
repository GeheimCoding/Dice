use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

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
    let icosahedron = generate_regular_icosahedron();
    commands.spawn((
        Spinnable,
        Mesh3d(meshes.add(icosahedron)),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.8, 0.8))),
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
        Transform::from_xyz(0.0, 0.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
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

fn generate_regular_icosahedron() -> Mesh {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let lu = (0, [-phi, 1.0, 0.0]);
    let ld = (1, [-phi, -1.0, 0.0]);
    let ru = (2, [phi, 1.0, 0.0]);
    let rd = (3, [phi, -1.0, 0.0]);
    let uf = (4, [0.0, phi, 1.0]);
    let ub = (5, [0.0, phi, -1.0]);
    let df = (6, [0.0, -phi, 1.0]);
    let db = (7, [0.0, -phi, -1.0]);
    let fl = (8, [-1.0, 0.0, phi]);
    let fr = (9, [1.0, 0.0, phi]);
    let bl = (10, [-1.0, 0.0, -phi]);
    let br = (11, [1.0, 0.0, -phi]);

    let vertices = vec![lu, ld, ru, rd, uf, ub, df, db, fl, fr, bl, br];
    let triangles = vec![
        // top pyramid
        [ub, uf, ru],
        [ub, ru, br],
        [ub, br, bl],
        [ub, bl, lu],
        [ub, lu, uf],
        // pentagonal biprism
        [fl, uf, lu],
        [fl, fr, uf],
        [fr, ru, uf],
        [fr, rd, ru],
        [rd, br, ru],
        [rd, db, br],
        [db, bl, br],
        [db, ld, bl],
        [ld, lu, bl],
        [ld, fl, lu],
        // bottom pyramid
        [df, db, rd],
        [df, rd, fr],
        [df, fr, fl],
        [df, fl, ld],
        [df, ld, db],
    ];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertices.iter().map(|p| p.1).collect::<Vec<_>>(),
    )
    .with_inserted_indices(Indices::U16(
        triangles.iter().flat_map(|i| i.map(|i| i.0)).collect(),
    ))
    .with_duplicated_vertices()
    .with_computed_normals()
}
