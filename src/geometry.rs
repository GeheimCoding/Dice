use bevy::asset::RenderAssetUsages;
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use std::collections::HashMap;

pub fn create_icosphere(iterations: u8) -> Mesh {
    assert!(iterations <= 8, "iterations must be between 0 and 8");

    let icosahedron = generate_regular_icosahedron();
    let mut vertices = icosahedron
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(VertexAttributeValues::as_float3)
        .expect("vertices")
        .iter()
        .map(project_to_unit_circle)
        .collect::<Vec<_>>();
    let mut indices = Vec::from_iter(icosahedron.indices().expect("indices").iter());

    for _ in 0..iterations {
        let mut new_indices = Vec::new();
        let mut index_cache = HashMap::new();

        for i in (0..indices.len()).step_by(3) {
            let (p1, p2, p3) = (indices[i], indices[i + 1], indices[i + 2]);

            let m1 = create_midpoint((p1, p2), &mut vertices, &mut index_cache);
            let m2 = create_midpoint((p2, p3), &mut vertices, &mut index_cache);
            let m3 = create_midpoint((p3, p1), &mut vertices, &mut index_cache);

            new_indices.extend([p1, m1, m3]);
            new_indices.extend([p2, m2, m1]);
            new_indices.extend([p3, m3, m2]);
            new_indices.extend([m1, m2, m3]);
        }

        indices = new_indices;
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_indices(Indices::U32(
        indices.iter().map(|i| *i as u32).collect::<Vec<u32>>(),
    ))
}

type Vertex = [f32; 3];

fn project_to_unit_circle(vertex: &Vertex) -> Vertex {
    let length = (vertex[0] * vertex[0] + vertex[1] * vertex[1] + vertex[2] * vertex[2]).sqrt();
    [vertex[0] / length, vertex[1] / length, vertex[2] / length]
}

fn create_midpoint(
    (p1, p2): (usize, usize),
    vertices: &mut Vec<Vertex>,
    index_cache: &mut HashMap<(usize, usize), usize>,
) -> usize {
    let key = (std::cmp::min(p1, p2), std::cmp::max(p1, p2));
    if let Some(index) = index_cache.get(&key) {
        return *index;
    }
    let p1 = vertices[p1];
    let p2 = vertices[p2];
    let midpoint = [
        (p1[0] + p2[0]) / 2.0,
        (p1[1] + p2[1]) / 2.0,
        (p1[2] + p2[2]) / 2.0,
    ];
    let index = vertices.len();

    vertices.push(project_to_unit_circle(&midpoint));
    index_cache.insert(key, index);

    index
}

pub fn generate_regular_icosahedron() -> Mesh {
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
}
