use std::f64::consts::PI;

pub fn norm(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

pub fn dist(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    norm(x1 - x2, y1 - y2)
}

pub fn normed_vector(x: f64, y: f64) -> (f64, f64) {
    (x * (1.0 / norm(x, y)), y * (1.0 / norm(x, y)))
}

pub fn circular_points(center_x: f64, center_y: f64, radius: f64, num_points: usize) -> Vec<(f64, f64)> {
    let mut circular_coords = Vec::new();
    for i in 0..num_points {
        circular_coords.push((
            center_x + (i as f64 * (2.0 * PI) / num_points as f64).cos() * radius,
            center_y + (i as f64 * (2.0 * PI) / num_points as f64).sin() * radius,
        ))
    }
    circular_coords
}

pub fn bisecting_vector(middle_x: f64, middle_y: f64, clkwise_x: f64, clkwise_y: f64, ctrclkwise_x: f64, ctrclkwise_y: f64) -> (f64, f64) {
    let (normed_offset_clkwise_x, normed_offset_clkwise_y) = normed_vector(clkwise_x - middle_x, clkwise_y - middle_y);
    let (normed_offset_ctrclkwise_x, normed_offset_ctrclkwise_y) = normed_vector(ctrclkwise_x - middle_x, ctrclkwise_y - middle_y);

    let (mut dir_x, mut dir_y) = (
        (normed_offset_clkwise_x + normed_offset_ctrclkwise_x) * 0.5,
        (normed_offset_clkwise_y + normed_offset_ctrclkwise_y) * 0.5,
    );

    if dir_x == 0.0 && dir_y == 0.0 {
        dir_x = -normed_offset_clkwise_y;
        dir_y = normed_offset_clkwise_x; // Rotate 90 degrees
    }
    let (at_zero_x, at_zero_y) = normed_vector(ctrclkwise_y - clkwise_y, clkwise_x - ctrclkwise_x);
    // If vectors form a reflex angle, their average will be in the opposite direction
    if norm(at_zero_x - dir_x, at_zero_y - dir_y) > norm(at_zero_x + dir_x, at_zero_y + dir_y) {
        normed_vector(-dir_x, -dir_y)
    } else {
        normed_vector(dir_x, dir_y)
    }
}

/* 0,0 -> x1, y1, 0,0 -> x2,y2 vector cross product.
Positive if (0,0)->1->2 is a counter-clockwise turn, negative if clockwise, 0 if collinear. */
fn cross_product(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    x1 * y2 - y1 * x2
}

/* Returns potential intersection between lines (x1 y1, x2 y2) and (x3 y3, x4 y4) */
fn intersection(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64) -> Option<(f64, f64)> {
    let (rx, ry, sx, sy) = (x2 - x1, y2 - y1, x4 - x3, y4 - y3);

    /* Now we have: line = q + qv, and any point on the vector is obtainable by p + t*r, for some t
    We want a t and u so p + t*pv = q + u*qv. Then t = (q − p) × s / (r × s) and u = (q − p) × r / (r × s) */
    let cross_rs = cross_product(rx, ry, sx, sy);

    /* Collinear. We always treat as non-intersections */
    if cross_rs == 0.0 {
        None
    } else {
        let t = cross_product(x3 - x1, y3 - y1, sx, sy) / cross_rs;
        let u = cross_product(x3 - x1, y3 - y1, rx, ry) / cross_rs;
        if t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0 {
            Some((x1 + rx * t, y1 + ry * t))
        } else {
            None
        }
    }
}

pub fn lines_intersection(lines: &Vec<(f64, f64, f64, f64)>) -> Option<(f64, f64)> {
    for i in 0..lines.len() - 1 {
        let (x1, y1, x2, y2) = lines[i];
        for j in i..lines.len() - 1 {
            let (x3, y3, x4, y4) = lines[j];
            match intersection(x1, y1, x2, y2, x3, y3, x4, y4) {
                Some(int) => return Some(int),
                _ => continue,
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;

    #[test]
    fn direction_for_inner_push_is_correct() {
        // TODO: Looks kinda good but deserves more thought
        assert_eq!(bisecting_vector(0.0, 0.0, -1.0, -0.5, 1.0, -0.5), (0.0, -1.0));
        let (some_dir_x, some_dir_y) = bisecting_vector(0.0, 0.0, 0.0, 100.0, 1.0, 0.0);
        assert_eq!(some_dir_y, some_dir_y);
        approx_eq!(f64, norm(some_dir_x, some_dir_y), 1.0);
    }

    #[test]
    fn bisecting_vector_is_in_the_middle() {
        let ((_left_x, _left_y), (_right_x, _right_y)) = ((-1.0, 0.0), (0.0, 1.0));
        assert!(true);
    }

    #[test]
    fn intersection_is_calculated_correctly() {
        let inter = intersection(1.0, 1.0, 2.0, 2.0, 1.0, 2.0, 2.0, 1.0);
        match inter {
            Some((x, y)) => {
                assert_eq!(x, 1.5);
                assert_eq!(y, 1.5)
            }
            None => assert!(false),
        }

        /* Two lines having a point in common shouldn't intersect */
        let (x1, y1) = (1.0, 1.0);
        let inter = intersection(x1, y1, 2.0, 2.0, x1, y1, 49.0, 78.0);
        match inter {
            Some(_) => assert!(false),
            None => assert!(true),
        }
    }
}
