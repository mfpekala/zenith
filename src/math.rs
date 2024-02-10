use bevy::prelude::*;

pub struct MathLine {
    pub p1: Vec2,
    pub p2: Vec2,
}
impl MathLine {
    pub fn from_points(points: &[Vec2]) -> Vec<MathLine> {
        let mut result: Vec<MathLine> = vec![];
        for ix in 0..points.len() {
            let line = MathLine {
                p1: points[ix],
                p2: points[(ix + 1) % points.len()],
            };
            result.push(line);
        }
        result
    }

    pub fn rise(&self) -> f32 {
        self.p2.x - self.p1.x
    }

    pub fn run(&self) -> f32 {
        self.p2.y - self.p1.y
    }

    pub fn with_clockwise_breathing_room(&self, space: f32) -> MathLine {
        let along_line = (self.p2 - self.p1).normalize();
        let offset = Vec2 {
            x: -along_line.y,
            y: along_line.x,
        };
        return MathLine {
            p1: self.p1 + offset * space,
            p2: self.p2 + offset * space,
        };
    }

    pub fn intersect(&self, other: &MathLine) -> Option<Vec2> {
        let a1 = self.p2.y - self.p1.y;
        let b1 = self.p1.x - self.p2.x;
        let c1 = a1 * self.p1.x + b1 * self.p1.y;

        let a2 = other.p2.y - other.p1.y;
        let b2 = other.p1.x - other.p2.x;
        let c2 = a2 * other.p1.x + b2 * other.p1.y;

        let determinant = a1 * b2 - a2 * b1;

        if determinant == 0.0 {
            // Lines are parallel, no intersection
            return None;
        }

        let x = (b2 * c1 - b1 * c2) / determinant;
        let y = (a1 * c2 - a2 * c1) / determinant;
        Some(Vec2 { x, y })
    }

    pub fn closest_point_on_segment(&self, other_point: &Vec2) -> Vec2 {
        let l2 = (self.p2.x - self.p1.x).powi(2) + (self.p2.y - self.p1.y).powi(2);
        let t = ((other_point.x - self.p1.x) * (self.p2.x - self.p1.x)
            + (other_point.y - self.p1.y) * (self.p2.y - self.p1.y))
            / l2;
        if t < 0.0 {
            self.p1
        } else if t > 1.0 {
            self.p2
        } else {
            Vec2 {
                x: self.p1.x + t * (self.p2.x - self.p1.x),
                y: self.p1.y + t * (self.p2.y - self.p1.y),
            }
        }
    }

    pub fn closest_point_on_line(&self, other_point: &Vec2) -> Vec2 {
        let l2 = (self.p2.x - self.p1.x).powi(2) + (self.p2.y - self.p1.y).powi(2);
        let t = ((other_point.x - self.p1.x) * (self.p2.x - self.p1.x)
            + (other_point.y - self.p1.y) * (self.p2.y - self.p1.y))
            / l2;
        Vec2 {
            x: self.p1.x + t * (self.p2.x - self.p1.x),
            y: self.p1.y + t * (self.p2.y - self.p1.y),
        }
    }

    pub fn signed_distance_from_point(&self, other_point: &Vec2) -> f32 {
        let line_diff = self.p2 - self.p1;
        let normal_pointing = Vec2 {
            x: line_diff.y,
            y: -line_diff.x,
        };
        let diff = self.p1 - *other_point;
        let dotprod = diff.dot(normal_pointing);
        let closest_point = self.closest_point_on_line(other_point);
        dotprod.signum() * other_point.distance(closest_point)
    }
}

/// Given a polygon of points GIVEN IN CLOCKWISE ORDER, return the points of the shell
/// that is formed by giving each edge some breathing room
pub fn get_shell(rock_points: &Vec<Vec2>, space: f32) -> Vec<Vec2> {
    let mut result: Vec<Vec2> = vec![];
    for ix in 0..rock_points.len() {
        let l1 = MathLine {
            p1: rock_points[ix],
            p2: rock_points[(ix + 1) % rock_points.len()],
        };
        let l1 = l1.with_clockwise_breathing_room(space);
        let l2 = MathLine {
            p1: rock_points[(ix + 1) % rock_points.len()],
            p2: rock_points[(ix + 2) % rock_points.len()],
        };
        let l2 = l2.with_clockwise_breathing_room(space);
        let Some(point) = l1.intersect(&l2) else {
            continue;
        };
        result.push(point);
    }
    // Re order so the points correspond
    let Some(end) = result.pop() else {
        return result;
    };
    result.insert(0, end);
    result
}
