use crate::types::util::drawing::diagram_layout::Point;

pub fn remove_redundant_bendpoints(points: &Vec<Point>) -> Vec<Point> {
    if points.len() <= 2 {
        return points.clone();
    }

    let mut out = Vec::new();
    out.push(points[0]);
    for i in 1..points.len() - 1 {
        let p0 = points[i - 1];
        let p1 = points[i];
        let p2 = points[i + 1];
        if colinear(p0, p1, p2) {
            continue;
        }
        out.push(p1);
    }
    out.push(points[points.len() - 1]);

    out
}

fn colinear(a: Point, b: Point, c: Point) -> bool {
    let double_area = a.x * b.y + b.x * c.y + c.x * a.y - b.x * a.y - a.x * c.y - c.x * b.y;
    double_area < 1.0e-5
}
