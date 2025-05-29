use bevy::asset::RenderAssetUsages;
use bevy::prelude::ops::{cos, sin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use std::collections::{HashMap, HashSet};
use std::fmt::Error;

pub fn create_icosphere(iterations: u8) -> Mesh {
    assert!(iterations <= 8, "iterations must be between 0 and 8");

    let icosahedron = generate_regular_icosahedron();
    let (vertices, mut indices) = extract_mesh_attributes(&icosahedron).expect("valid mesh");
    let mut vertices = vertices
        .iter()
        .map(project_to_unit_circle)
        .collect::<Vec<_>>();

    for _ in 0..iterations {
        let mut new_indices = vec![];
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
    construct_mesh(vertices, indices)
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

pub fn create_d6(depth: u8, threshold: f32, size: f32) -> Mesh {
    let mut d6 = create_icosphere(depth);
    let orientations = vec![
        (2, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Y), // left
        (5, Vec3::X, Vec3::Z, Vec3::Y),         // right
        (6, Vec3::Y, Vec3::NEG_Z, Vec3::X),     // up
        (1, Vec3::NEG_Y, Vec3::Z, Vec3::X),     // down
        (4, Vec3::Z, Vec3::Y, Vec3::X),         // front
        (3, Vec3::NEG_Z, Vec3::Y, Vec3::NEG_X), // back
    ];
    let mut uvs = vec![[0.0, 0.0]; d6.count_vertices()];
    for (die_face, plane_normal, reference, clockwise_normal) in orientations {
        let center = plane_normal * threshold;
        let circle_start_index = d6.count_vertices();
        d6 = intersect_mesh_with_plane(d6, center, plane_normal).expect("valid mesh");
        let circle_count = d6.count_vertices() - circle_start_index;
        uvs.extend(vec![[0.0, 0.0]; circle_count]);
        d6 = fill_circle(
            d6,
            (center, reference * threshold, clockwise_normal * threshold),
            circle_start_index,
            &mut uvs,
        );
        for i in uvs.len() - circle_count - 1..uvs.len() {
            uvs[i][0] = (die_face - 1) as f32 * 1.0 / 6.0 + uvs[i][0] / 6.0;
        }
    }
    d6 = remove_if(
        d6,
        |vertex| vertex.iter().any(|c| c.abs() > threshold),
        &mut uvs,
    );
    let (vertices, indices) = extract_mesh_attributes(&d6).expect("valid mesh");

    let scale_factor = size / (2.0 * threshold);
    let scaled_vertices = vertices
        .iter()
        .map(|[x, y, z]| [x * scale_factor, y * scale_factor, z * scale_factor])
        .collect::<Vec<_>>();

    construct_mesh(scaled_vertices, indices)
        .with_computed_normals()
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn intersect_mesh_with_plane(mesh: Mesh, plane_point: Vec3, plane_normal: Vec3) -> Result<Mesh> {
    let (mut vertices, indices) = extract_mesh_attributes(&mesh).ok_or(Error::default())?;
    let mut index_cache = HashMap::new();
    let mut new_indices = vec![];

    for i in (0..indices.len()).step_by(3) {
        let triangle_indices = vec![indices[i], indices[i + 1], indices[i + 2]];
        let intersections =
            intersect_triangle_with_plane(&vertices, &triangle_indices, plane_point, plane_normal);
        if !needs_triangulation(&intersections) {
            new_indices.extend(triangle_indices);
            continue;
        }

        let mut triangle = vec![];
        for i in 0..3 {
            let i1 = triangle_indices[i];
            let i2 = triangle_indices[(i + 1) % 3];
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
            let i1 = triangle_indices[t];
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
    Ok(construct_mesh(vertices, new_indices))
}

fn intersect_triangle_with_plane(
    vertices: &[[f32; 3]],
    indices: &[usize],
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Vec<Option<Vec3>> {
    let mut collisions = vec![];
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

fn remove_if<Predicate: Fn(Vertex) -> bool>(
    mesh: Mesh,
    predicate: Predicate,
    uvs: &mut Vec<[f32; 2]>,
) -> Mesh {
    let (vertices, indices) = extract_mesh_attributes(&mesh).expect("valid mesh");
    let mut index_offsets = vec![];
    let mut new_vertices = vec![];
    let mut new_indices = vec![];
    let mut removed = HashSet::new();

    let mut new_uvs = vec![];
    for v in 0..vertices.len() {
        let vertex = vertices[v];
        if predicate(vertex) {
            removed.insert(v);
        } else {
            new_vertices.push(vertex);
            new_uvs.push(uvs[v]);
        }
        index_offsets.push(removed.len());
    }
    *uvs = new_uvs;
    for i in (0..indices.len()).step_by(3) {
        let indices = vec![indices[i], indices[i + 1], indices[i + 2]];
        if indices.iter().all(|i| !removed.contains(i)) {
            new_indices.extend(indices.iter().map(|i| i - index_offsets[*i]));
        }
    }
    construct_mesh(new_vertices, new_indices)
}

fn fill_circle(
    mesh: Mesh,
    (center, reference, clockwise_normal): (Vec3, Vec3, Vec3),
    mut start_index: usize,
    uvs: &mut Vec<[f32; 2]>,
) -> Mesh {
    let reference = center + reference;
    let (mut clockwise, mut counter) = (vec![], vec![]);
    let (mut vertices, mut indices) = extract_mesh_attributes(&mesh).expect("valid mesh");

    let len = vertices.len();
    for i in start_index..len {
        vertices.push(vertices[i]);
        uvs.push([0.0, 0.0]);
        start_index = i + 1;
    }
    for index in start_index..vertices.len() {
        let vertex = Vec3::from_array(vertices[index]);
        if vertex.distance(center + clockwise_normal) < vertex.distance(center - clockwise_normal) {
            clockwise.push(index);
        } else {
            counter.push(index);
        }
    }
    let sort_by_angle = |a, b| {
        reference
            .angle_between(Vec3::from_array(vertices[a]))
            .partial_cmp(&reference.angle_between(Vec3::from_array(vertices[b])))
            .expect("comparable")
    };
    clockwise.sort_by(|a, b| sort_by_angle(*b, *a));
    counter.sort_by(|a, b| sort_by_angle(*a, *b));

    let counter_len = counter.len();
    let mut sorted_indices = counter;
    sorted_indices.extend(clockwise);

    let center_index = vertices.len();
    for i in 0..sorted_indices.len() {
        let v1 = sorted_indices[i];
        let v2 = sorted_indices[(i + 1) % sorted_indices.len()];
        indices.extend([v1, v2, center_index]);

        let mut angle = (reference - center).angle_between(Vec3::from_array(vertices[v1]) - center);
        if i >= counter_len {
            angle = 2.0 * std::f32::consts::PI - angle;
        }
        uvs[v1] = [(-sin(angle) + 1.0) / 2.0, 1.0 - (cos(angle) + 1.0) / 2.0];
    }
    vertices.push(center.to_array());
    uvs.push([0.5, 0.5]);

    construct_mesh(vertices, indices)
}

fn extract_mesh_attributes(mesh: &Mesh) -> Option<(Vec<Vertex>, Vec<usize>)> {
    Some((
        Vec::from(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(VertexAttributeValues::as_float3)?,
        ),
        Vec::from_iter(mesh.indices()?.iter()),
    ))
}

fn construct_mesh(vertices: Vec<Vertex>, indices: Vec<usize>) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_indices(Indices::U32(indices.iter().map(|i| *i as u32).collect()))
}
