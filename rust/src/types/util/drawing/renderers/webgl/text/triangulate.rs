use std::collections::{HashMap, HashSet};

use i_float::f64_point::F64Point;
use i_overlay::core::{
    fill_rule::FillRule, float_overlay::FloatOverlay, overlay::ShapeType, overlay_rule::OverlayRule,
};
use swash::zeno::{Command, Vector};

use crate::util::{logging::console, rectangle::Rectangle};

pub fn triangulate(
    commands: impl Iterator<Item = Command> + Clone,
    distance_per_sample_point: f32,
) -> Vec<Vector> {
    let paths = sample_paths(commands, distance_per_sample_point);
    let path_groups = get_overlapping_paths(&paths);
    path_groups
        .iter()
        .flat_map(|group| {
            let paths = group
                .iter()
                .map(|polygon| {
                    polygon
                        .iter()
                        .map(|point| vec![point.x, point.y])
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let (vertices, holes, dimensions) = earcutr::flatten(&paths);
            let triangles = earcutr::earcut(&vertices, &holes, dimensions);

            if triangles.is_ok() {
                let triangles = triangles.unwrap();
                let triangle_points = triangles.iter().map(|&index| {
                    vertices
                        .get(2 * index)
                        .and_then(|x| vertices.get(2 * index + 1).map(|y| (x, y)))
                        .map_or_else(|| Vector::new(0., 0.), |(&x, &y)| Vector::new(x, y))
                });
                triangle_points.collect::<Vec<Vector>>()
            } else {
                Vec::new()
            }
        })
        .collect::<Vec<Vector>>()
}

fn sample_paths(
    commands: impl Iterator<Item = Command> + Clone,
    distance_per_point: f32,
) -> Vec<Vec<Vector>> {
    let mut paths = Vec::<Vec<Vector>>::new();

    let mut points = Vec::<Vector>::new();
    let mut cur = Vector::new(0., 0.);
    for cmd in commands {
        match cmd {
            // TODO: properly sample the curves
            Command::MoveTo(p) => {
                cur = p;
            }
            Command::LineTo(p) => {
                points.push(cur);
                cur = p;
            }
            Command::QuadTo(p1, p2) => {
                points.push(cur);
                let point_count = f32::floor(length([cur, p1, p2]) / distance_per_point) as i32;
                for i in 1..=point_count {
                    // starting at 1 such that per does not start at 0
                    let per = i as f32 / (point_count + 1) as f32; // point_count + 1 such that per does not reach 1
                    points.push(quad_lerp(cur, p1, p2, per));
                }
                cur = p2;
            }
            Command::CurveTo(p1, p2, p3) => {
                points.push(cur);
                let point_count = f32::floor(length([cur, p1, p2, p3]) / distance_per_point) as i32;
                for i in 1..=point_count {
                    // starting at 1 such that per does not start at 0
                    let per = i as f32 / (point_count + 1) as f32; // point_count + 1 such that per does not reach 1
                    points.push(cubic_lerp(cur, p1, p2, p3, per));
                }
                cur = p3;
            }
            Command::Close => {
                points.push(cur);
                paths.push(points);
                points = Vec::new();
            }
        }
    }

    paths
}

fn length<const C: usize>(vecs: [Vector; C]) -> f32 {
    let mut sum = 0.;
    for i in 0..C - 1 {
        let a = vecs[i];
        let b = vecs[i + 1];
        sum += a.distance_to(b);
    }
    sum
}

fn cubic_lerp(p1: Vector, p2: Vector, p3: Vector, p4: Vector, per: f32) -> Vector {
    let p5 = lerp(p1, p2, per);
    let p6 = lerp(p2, p3, per);
    let p7 = lerp(p3, p4, per);
    quad_lerp(p5, p6, p7, per)
}
fn quad_lerp(p1: Vector, p2: Vector, p3: Vector, per: f32) -> Vector {
    let p4 = lerp(p1, p2, per);
    let p5 = lerp(p2, p3, per);
    lerp(p4, p5, per)
}
fn lerp(start: Vector, end: Vector, per: f32) -> Vector {
    start * (1. - per) + end * per
}

/// The output represents: A set (vector) of groups (vector) of paths (vector)
fn get_overlapping_paths(paths: &Vec<Vec<Vector>>) -> Vec<Vec<Vec<Vector>>> {
    let bounding_boxes = paths
        .iter()
        .map(|path| {
            let Some(first) = path.get(0) else {
                return Rectangle::new(0., 0., 0., 0.);
            };
            let mut min_x = first.x;
            let mut max_x = first.x;
            let mut min_y = first.y;
            let mut max_y = first.y;
            for p in path {
                if p.x < min_x {
                    min_x = p.x
                }
                if p.x > max_x {
                    max_x = p.x
                }
                if p.y < min_y {
                    min_y = p.y
                }
                if p.y > max_y {
                    max_y = p.y
                }
            }
            Rectangle::new(min_x, min_y, max_x - min_x, max_y - min_y)
        })
        .collect::<Vec<Rectangle>>();

    let mut partition: HashMap<usize, HashSet<usize>> = paths
        .iter()
        .enumerate()
        .map(|(i, _)| (i, HashSet::from([i])))
        .collect();
    let mut partition_lookup: HashMap<usize, usize> =
        paths.iter().enumerate().map(|(i, _)| (i, i)).collect();

    for i in 0..paths.len() {
        let path_a = paths.get(i).unwrap();
        let bb_a = bounding_boxes.get(i).unwrap();
        for j in 0..i {
            if partition_lookup.get(&i) == partition_lookup.get(&j) {
                continue;
            }

            let path_b = paths.get(j).unwrap();
            let bb_b = bounding_boxes.get(j).unwrap();

            if bb_a.overlaps(bb_b) && overlap_polygon(path_a, path_b) {
                // Merge the two groups into the group of b
                let partition_index_a = partition_lookup.get(&i).unwrap();
                let partition_index_b = partition_lookup.get(&j).unwrap();
                let group_a = partition.get(partition_index_a).unwrap().clone();
                let group_b = partition.get_mut(partition_index_b).unwrap();
                group_b.extend(group_a);
                partition.remove(partition_index_a);
                partition_lookup.insert(i, *partition_index_b);
            }
        }
    }

    partition
        .iter()
        .map(|(_, group)| {
            get_polygon_and_holes(
                group
                    .iter()
                    .map(|&i| paths.get(i).unwrap().clone())
                    .collect(),
            )
        })
        .collect()
}

/// Assumes there is no line intersection without point containment, ignores this edge case
fn overlap_polygon(polygon_a: &Vec<Vector>, polygon_b: &Vec<Vector>) -> bool {
    polygon_b.iter().any(|p| inside_polygon(polygon_a, p))
        || polygon_a.iter().any(|p| inside_polygon(polygon_b, p))
}
fn inside_polygon(polygon: &Vec<Vector>, point: &Vector) -> bool {
    let lines = polygon.iter().zip(polygon.iter().cycle().skip(1));
    let mut inside = false;
    for (a, b) in lines {
        let within_y_range = (a.y > point.y) != (b.y > point.y);
        if within_y_range && is_right_of_line(a, b, &point) {
            inside = !inside
        }
    }
    inside
}

fn is_right_of_line(start: &Vector, end: &Vector, point: &Vector) -> bool {
    let res = (end.x - start.x) * (point.y - start.y) - (end.y - start.y) * (point.x - start.x);
    (res < 0.) == (start.y < end.y)
}

/// Combines the given paths such that we have a single polygon with the given holes
fn get_polygon_and_holes(paths: Vec<Vec<Vector>>) -> Vec<Vec<Vector>> {
    if paths.len() <= 1 {
        return paths;
    }
    let paths = paths
        .iter()
        .map(|path| {
            path.iter()
                .map(|point| F64Point::new(point.x as f64, point.y as f64))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let shapes =
        paths
            .iter()
            .skip(1)
            .fold(vec![vec![paths.get(0).unwrap().clone()]], |res, path| {
                let mut overlay = FloatOverlay::new();
                overlay.add_path(path, ShapeType::Subject);
                for shape in res {
                    overlay.add_path(shape.get(0).unwrap(), ShapeType::Clip);
                }
                let graph = overlay.build_graph(FillRule::EvenOdd);
                let shapes = graph.extract_shapes(OverlayRule::Union);
                shapes
            });

    let shape = shapes
        .get(0)
        .unwrap()
        .get(0)
        .unwrap()
        .iter()
        .map(|p| Vector::new(p.x as f32, p.y as f32))
        .collect::<Vec<_>>();
    let shapes_with_holes = paths
        .iter()
        .filter(|path| {
            let is_fully_inside = path
                .iter()
                .all(|p| inside_polygon(&shape, &Vector::new(p.x as f32, p.y as f32)));
            is_fully_inside
        })
        .fold(shapes, |res, path| {
            let mut overlay = FloatOverlay::new();
            for shape in res {
                for (i, polygon) in shape.iter().enumerate() {
                    overlay.add_path(
                        polygon,
                        if i == 0 {
                            ShapeType::Subject
                        } else {
                            ShapeType::Clip
                        },
                    );
                }
            }
            overlay.add_path(path, ShapeType::Clip);
            let graph = overlay.build_graph(FillRule::EvenOdd);
            let shapes = graph.extract_shapes(OverlayRule::Difference);
            shapes
        });

    let shape = shapes_with_holes.get(0).unwrap();
    shape
        .iter()
        .map(|polygon| {
            polygon
                .iter()
                .map(|point| Vector::new(point.x as f32, point.y as f32))
                .collect()
        })
        .collect()
}
