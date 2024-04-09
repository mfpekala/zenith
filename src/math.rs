use bevy::prelude::*;
use linreg::linear_regression;

#[derive(Debug)]
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
        self.p2.y - self.p1.y
    }

    pub fn run(&self) -> f32 {
        self.p2.x - self.p1.x
    }

    pub fn with_clockwise_breathing_room(&self, space: f32) -> MathLine {
        let along_line = (self.p2 - self.p1).normalize();
        let offset = Vec2 {
            x: -along_line.y,
            y: along_line.x,
        };
        MathLine {
            p1: self.p1 + offset * space,
            p2: self.p2 + offset * space,
        }
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

    pub fn slope_fit_points(points: &Vec<Vec2>) -> Self {
        // ALL WE CARE ABOUT IS SLOPE
        let xs: Vec<f64> = points.iter().map(|p| p.x as f64).collect();
        let ys: Vec<f64> = points.iter().map(|p| p.y as f64).collect();
        let (slope, _): (f64, f64) = linear_regression(&xs, &ys).unwrap();
        Self {
            p1: Vec2::ZERO,
            p2: Vec2::new(1.0, slope as f32),
        }
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

pub fn regular_polygon(num_sides: u32, mut angle: f32, radius: f32) -> Vec<Vec2> {
    let mut points: Vec<Vec2> = vec![];
    for _ in 0..num_sides {
        let x = angle.to_radians().cos() * radius;
        let y = angle.to_radians().sin() * radius;
        points.push(Vec2 { x, y });
        angle -= 360.0 / (num_sides as f32);
    }
    points
}

pub fn icenter(points: &Vec<IVec2>) -> IVec2 {
    if points.len() == 0 {
        return IVec2::ZERO;
    }
    let mut center = Vec2::ZERO;
    for point in points.iter() {
        center += point.as_vec2();
    }
    center = center / points.len() as f32;
    IVec2::new(center.x.round() as i32, center.y.round() as i32)
}

pub fn irecenter(points: Vec<IVec2>, center: &IVec2) -> Vec<IVec2> {
    points.into_iter().map(|p| p - *center).collect()
}

pub fn lerp(x: f32, start: f32, end: f32) -> f32 {
    start + x * (end - start)
}

pub fn lerp_color(x: f32, c1: Color, c2: Color) -> Color {
    Color::Hsla {
        hue: c1.h() + x * (c2.h() - c1.h()),
        saturation: c1.s() + x * (c2.s() - c1.s()),
        lightness: c1.l() + x * (c2.l() - c1.l()),
        alpha: c1.a() + x * (c2.a() - c1.a()),
    }
}

pub fn uvec2mesh_points(v: UVec2) -> Vec<Vec2> {
    vec![
        Vec2::new(-(v.x as f32) / 2.0, -(v.y as f32) / 2.0),
        Vec2::new(-(v.x as f32) / 2.0, (v.y as f32) / 2.0),
        Vec2::new((v.x as f32) / 2.0, (v.y as f32) / 2.0),
        Vec2::new((v.x as f32) / 2.0, -(v.y as f32) / 2.0),
    ]
}

#[derive(Debug, Clone, Copy)]
pub enum Spleen {
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInQuartic,
    EaseOutQuartic,
    EaseInOutQuartic,
    EaseInQuintic,
    EaseOutQuintic,
    EaseInOutQuintic,
}

impl Spleen {
    pub fn interp(&self, x: f32) -> f32 {
        match *self {
            Self::EaseInCubic => ease_in_cubic(x),
            Self::EaseOutCubic => ease_out_cubic(x),
            Self::EaseInOutCubic => ease_in_out_cubic(x),
            Self::EaseInQuad => ease_in_quad(x),
            Self::EaseOutQuad => ease_out_quad(x),
            Self::EaseInOutQuad => ease_in_out_quad(x),
            Self::EaseInQuartic => ease_in_quartic(x),
            Self::EaseOutQuartic => ease_out_quartic(x),
            Self::EaseInOutQuartic => ease_in_out_quartic(x),
            Self::EaseInQuintic => ease_in_quintic(x),
            Self::EaseOutQuintic => ease_out_quintic(x),
            Self::EaseInOutQuintic => ease_in_out_quintic(x),
        }
    }
}

fn ease_in_cubic(x: f32) -> f32 {
    x * x * x
}

fn ease_out_cubic(x: f32) -> f32 {
    1.0 - ease_in_cubic(1.0 - x)
}

fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - ((0.0 - 2.0) * x + 2.0).powf(3.0) / 2.0
    }
}

fn ease_in_quad(x: f32) -> f32 {
    x * x
}

fn ease_out_quad(x: f32) -> f32 {
    1.0 - ease_in_quad(1.0 - x)
}

fn ease_in_out_quad(x: f32) -> f32 {
    if x < 0.5 {
        2.0 * x * x
    } else {
        1.0 - ((0.0 - 2.0) * x + 2.0).powf(2.0) / 2.0
    }
}

fn ease_in_quartic(x: f32) -> f32 {
    x * x * x * x
}

fn ease_out_quartic(x: f32) -> f32 {
    1.0 - ease_in_quartic(1.0 - x)
}

fn ease_in_out_quartic(x: f32) -> f32 {
    if x < 0.5 {
        8.0 * x.powi(4)
    } else {
        1.0 - ((-2.0 * x + 2.0).powi(4)) / 2.0
    }
}

fn ease_in_quintic(x: f32) -> f32 {
    x * x * x * x * x
}

fn ease_out_quintic(x: f32) -> f32 {
    1.0 - ease_in_quintic(1.0 - x)
}

fn ease_in_out_quintic(x: f32) -> f32 {
    if x < 0.5 {
        16.0 * x.powi(5)
    } else {
        1.0 - ((-2.0 * x + 2.0).powi(5)) / 2.0
    }
}

#[cfg(test)]
mod math_nerd {
    use linreg::linear_regression;

    #[test]
    fn linear_regression_test() {
        let xs: Vec<f64> = vec![1.0, 2.0, 3.0];
        let ys: Vec<f64> = vec![0.0, 1.0, 2.0];
        let _out: (f64, f64) = linear_regression(&xs, &ys).unwrap();
    }
}
