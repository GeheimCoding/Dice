use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use std::collections::HashMap;
use std::fmt::Error;

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

pub fn intersect_mesh_with_plane(
    mesh: Mesh,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Result<Mesh> {
    let mut vertices = Vec::from(
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .ok_or(Error::default())?,
    );
    let indices = Vec::from_iter(mesh.indices().ok_or(Error::default())?.iter());
    let mut index_cache = HashMap::new();
    let mut new_indices = Vec::new();

    for i in (0..indices.len()).step_by(3) {
        let t_indices = vec![indices[i], indices[i + 1], indices[i + 2]];
        let t_vertices = t_indices.iter().map(|i| vertices[*i]).collect::<Vec<_>>();

        let intersections =
            intersect_triangle_with_plane(&t_vertices, &t_indices, plane_point, plane_normal);
        if !needs_triangulation(&intersections) {
            new_indices.extend(t_indices);
            continue;
        }

        let mut triangle = Vec::new();
        for i in 0..3 {
            let i1 = t_indices[i];
            let i2 = t_indices[(i + 1) % 3];
            let key = (std::cmp::min(i1, i2), std::cmp::max(i1, i2));
            if let Some(index) = index_cache.get(&key) {
                triangle.push((i1, Some(*index)));
                continue;
            }
            if let Some(intersection) = intersections[i] {
                let new_index = vertices.len();

                index_cache.insert(key, new_index);
                vertices.push(intersection.to_array());
                triangle.push((i1, Some(new_index)));
            } else {
                triangle.push((i1, None));
            }
        }
        for t in 0..3 {
            let i1 = t_indices[t];
            let i2 = if let Some(index) = triangle[t].1 {
                index
            } else {
                triangle[(t + 1) % 3].0
            };
            let mut x = (t + 2) % 3;
            while triangle[x].1 == None {
                x = (x + 2) % 3;
            }
            let i3 = triangle[x].1.expect("index");
            new_indices.extend([i1, i2, i3]);
        }
    }
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_indices(Indices::U32(
        new_indices.iter().map(|i| *i as u32).collect(),
    ));
    Ok(mesh)
}

fn intersect_triangle_with_plane(
    vertices: &[[f32; 3]],
    indices: &[usize],
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Vec<Option<Vec3>> {
    let mut collisions = Vec::new();
    for index in 0..indices.len() {
        let line = (
            Vec3::from_array(vertices[indices[index]]),
            Vec3::from_array(vertices[indices[(index + 1) % indices.len()]]),
        );
        collisions.push(intersect_line_with_plane(line, plane_point, plane_normal));
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

fn needs_triangulation(collisions: &[Option<Vec3>]) -> bool {
    collisions.iter().filter(|c| c.is_some()).count() > 1
}
