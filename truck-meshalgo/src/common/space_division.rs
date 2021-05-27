#![allow(dead_code)]

use super::*;

#[derive(Clone, Debug)]
pub struct HashedPointCloud {
    space: Vec<Vec<Point3>>,
    size: [usize; 3],
    range: [[f64; 2]; 3],
    num_points: usize,
}

impl HashedPointCloud {
    #[inline(always)]
    pub fn new(size: [usize; 3], range: [[f64; 2]; 3]) -> Self {
        HashedPointCloud {
            space: vec![Vec::new(); size[0] * size[1] * size[2]],
            size,
            range,
            num_points: 0,
        }
    }
    #[inline(always)]
    pub fn from_points<'a, I>(points: I, inf: f64) -> Self
    where
        I: IntoIterator + Clone,
        I::IntoIter: Iterator<Item = &'a Point3>, {
        let mut bdb = BoundingBox::<Point3>::new();
        let len = points.clone().into_iter().fold(0, |counter, pt| {
            bdb.push(pt);
            counter + 1
        });
        let size = 1.0 + f64::powf(len as f64, 1.0 / 3.0);
        let size = bdb.diagonal().map(|a| {
            if a / size < inf {
                1 + (a / inf) as usize
            } else {
                size as usize
            }
        });
        let mut res = HashedPointCloud::new(
            size.into(),
            [
                [bdb.min()[0], bdb.max()[0]],
                [bdb.min()[1], bdb.max()[1]],
                [bdb.min()[2], bdb.max()[2]],
            ],
        );
        points.into_iter().for_each(|pt| res.push(*pt));
        res
    }
    #[inline(always)]
    pub fn size(&self) -> [usize; 3] { self.size }
    #[inline(always)]
    pub fn push(&mut self, point: Point3) {
        self.num_points += 1;
        let idx = point.hash(self);
        self[idx].push(point);
    }
    #[inline(always)]
    pub fn distance(&self, t: impl DistanceWithPointCloud) -> Option<f64> { t.distance(self) }
    #[inline(always)]
    pub fn distance2(&self, t: impl DistanceWithPointCloud) -> Option<f64> { t.distance2(self) }
}

impl std::ops::Index<[usize; 3]> for HashedPointCloud {
    type Output = Vec<Point3>;
    #[inline(always)]
    fn index(&self, idcs: [usize; 3]) -> &Vec<Point3> {
        &self.space[(idcs[0] * self.size[1] + idcs[1]) * self.size[2] + idcs[2]]
    }
}

impl std::ops::IndexMut<[usize; 3]> for HashedPointCloud {
    #[inline(always)]
    fn index_mut(&mut self, idcs: [usize; 3]) -> &mut Vec<Point3> {
        &mut self.space[(idcs[0] * self.size[1] + idcs[1]) * self.size[2] + idcs[2]]
    }
}

pub trait SpaceHash {
    fn hash(self, space: &HashedPointCloud) -> [usize; 3];
}

impl SpaceHash for Point3 {
    #[inline(always)]
    fn hash(self, space: &HashedPointCloud) -> [usize; 3] {
        let x = (self[0] - space.range[0][0]) / (space.range[0][1] - space.range[0][0]);
        let ix = f64::clamp(x * space.size[0] as f64, 0.0, space.size[0] as f64 - 1.0) as usize;
        let y = (self[1] - space.range[1][0]) / (space.range[1][1] - space.range[1][0]);
        let iy = f64::clamp(y * space.size[1] as f64, 0.0, space.size[1] as f64 - 1.0) as usize;
        let z = (self[2] - space.range[2][0]) / (space.range[2][1] - space.range[2][0]);
        let iz = f64::clamp(z * space.size[2] as f64, 0.0, space.size[2] as f64 - 1.0) as usize;
        [ix, iy, iz]
    }
}

impl SpaceHash for usize {
    #[inline(always)]
    fn hash(self, space: &HashedPointCloud) -> [usize; 3] {
        [
            self / (space.size[1] * space.size[2]),
            self % (space.size[1] * space.size[2]) / space.size[2],
            self % space.size[2],
        ]
    }
}

pub trait DistanceWithPointCloud: Sized {
    fn distance2(self, space: &HashedPointCloud) -> Option<f64>;
    fn distance(self, space: &HashedPointCloud) -> Option<f64> {
        self.distance2(space).map(|dist2| f64::sqrt(dist2))
    }
}

impl DistanceWithPointCloud for Point3 {
    fn distance2(self, space: &HashedPointCloud) -> Option<f64> {
        let idcs = self.hash(space);
        let closure = |dist2: f64, pt: &Point3| f64::min(dist2, MetricSpace::distance2(self, *pt));
        let mut dist2 = space[idcs].iter().fold(std::f64::INFINITY, closure);
        if idcs[0] > 0 {
            dist2 = space[[idcs[0] - 1, idcs[1], idcs[2]]]
                .iter()
                .fold(dist2, closure);
        }
        if idcs[0] + 1 < space.size[0] {
            dist2 = space[[idcs[0] + 1, idcs[1], idcs[2]]]
                .iter()
                .fold(dist2, closure);
        }
        if idcs[1] > 0 {
            dist2 = space[[idcs[0], idcs[1] - 1, idcs[2]]]
                .iter()
                .fold(dist2, closure);
        }
        if idcs[1] + 1 < space.size[1] {
            dist2 = space[[idcs[0], idcs[1] + 1, idcs[2]]]
                .iter()
                .fold(dist2, closure);
        }
        if idcs[2] > 0 {
            dist2 = space[[idcs[0], idcs[1], idcs[2] - 1]]
                .iter()
                .fold(dist2, closure);
        }
        if idcs[2] + 1 < space.size[2] {
            dist2 = space[[idcs[0], idcs[1], idcs[2] + 1]]
                .iter()
                .fold(dist2, closure);
        }
        if dist2 == std::f64::INFINITY {
            None
        } else {
            Some(dist2)
        }
    }
}

impl DistanceWithPointCloud for [Point3; 3] {
    fn distance2(self, space: &HashedPointCloud) -> Option<f64> {
        let bdd: BoundingBox<Point3> = self.iter().collect();
        let mut range: [[usize; 3]; 2] = [bdd.min().hash(space), bdd.max().hash(space)];
        range[0][0] = usize::max(range[0][0], 1) - 1;
        range[0][1] = usize::max(range[0][1], 1) - 1;
        range[0][2] = usize::max(range[0][2], 1) - 1;
        range[1][0] = usize::min(range[1][0] + 1, space.size[0] - 1);
        range[1][1] = usize::min(range[1][1] + 1, space.size[1] - 1);
        range[1][2] = usize::min(range[1][2] + 1, space.size[2] - 1);
        let mut dist2 = std::f64::INFINITY;
        (range[0][0]..=range[1][0]).for_each(|ix| {
            (range[0][1]..=range[1][1]).for_each(|iy| {
                (range[0][2]..=range[1][2]).for_each(|iz| {
                    dist2 = space[[ix, iy, iz]].iter().fold(dist2, |dist2, pt| {
                        f64::min(dist2, distance2_point_triangle(*pt, self))
                    });
                })
            })
        });
        if dist2 == std::f64::INFINITY {
            None
        } else {
            Some(dist2)
        }
    }
}

impl<'a> DistanceWithPointCloud for &'a PolygonMesh {
    fn distance2(self, space: &HashedPointCloud) -> Option<f64> {
        Triangulate(self).into_iter().fold(None, |dist2, tri| {
            let tri = [
                self.positions()[tri[0].pos],
                self.positions()[tri[1].pos],
                self.positions()[tri[2].pos],
            ];
            match (dist2, tri.distance2(space)) {
                (Some(a), Some(b)) => Some(f64::max(a, b)),
                (None, Some(b)) => Some(b),
                _ => dist2,
            }
        })
    }
}

// https://iquilezles.org/www/articles/distfunctions/distfunctions.htm
fn distance2_point_triangle(point: Point3, triangle: [Point3; 3]) -> f64 {
    let ab = triangle[1] - triangle[0];
    let ap = point - triangle[0];
    let bc = triangle[2] - triangle[1];
    let bp = point - triangle[1];
    let ca = triangle[0] - triangle[2];
    let cp = point - triangle[2];
    let nor = ab.cross(ca);

    let coef = f64::signum(ab.cross(nor).dot(ap))
        + f64::signum(bc.cross(nor).dot(bp))
        + f64::signum(ca.cross(nor).dot(cp));
    if coef < 2.0 || nor.magnitude().so_small() {
        let a = (ap - ab * f64::clamp(ab.dot(ap) / ab.dot(ab), 0.0, 1.0)).magnitude2();
        let b = (bp - bc * f64::clamp(bc.dot(bp) / bc.dot(bc), 0.0, 1.0)).magnitude2();
        let c = (cp - ca * f64::clamp(ca.dot(cp) / ca.dot(ca), 0.0, 1.0)).magnitude2();
        f64::min(f64::min(a, b), c)
    } else {
        nor.dot(ap) * nor.dot(ap) / nor.magnitude2()
    }
}

#[test]
fn distance_point_triangle_test() {
    let triangle = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    assert_eq!(
        12.45 * 12.45,
        distance2_point_triangle(Point3::new(0.25, 0.25, 12.45), triangle)
    );
    assert_eq!(
        12.45 * 12.45,
        distance2_point_triangle(Point3::new(0.25, 0.25, -12.45), triangle)
    );
    assert_eq!(
        2.0,
        distance2_point_triangle(Point3::new(-1.0, 0.5, -1.0), triangle)
    );
    assert_eq!(
        2.0,
        distance2_point_triangle(Point3::new(0.5, -1.0, 1.0), triangle)
    );
    assert_eq!(
        1.5,
        distance2_point_triangle(Point3::new(1.0, 1.0, 1.0), triangle)
    );
    assert_eq!(
        3.0,
        distance2_point_triangle(Point3::new(-1.0, -1.0, -1.0), triangle)
    );
}

#[cfg(test)]
fn exec_space_division_distance() {
    const SPACE_SIZE: f64 = 100.0;
    const NUM_POINTS: usize = 1000;
    const NUM_TRIANGLES: usize = 10;
    const TRIANGLE_DISPLACEMENT: f64 = 1.0;

    let points = (0..NUM_POINTS)
        .map(|_| {
            Point3::new(
                SPACE_SIZE * rand::random::<f64>() - SPACE_SIZE / 2.0,
                SPACE_SIZE * rand::random::<f64>() - SPACE_SIZE / 2.0,
                SPACE_SIZE * rand::random::<f64>() - SPACE_SIZE / 2.0,
            )
        })
        .collect::<Vec<_>>();
    let triangles = (0..NUM_TRIANGLES)
        .map(|_| {
            let pt = Point3::new(
                SPACE_SIZE * rand::random::<f64>() - SPACE_SIZE / 2.0,
                SPACE_SIZE * rand::random::<f64>() - SPACE_SIZE / 2.0,
                SPACE_SIZE * rand::random::<f64>() - SPACE_SIZE / 2.0,
            );
            [
                pt,
                pt + Vector3::new(
                    TRIANGLE_DISPLACEMENT * 2.0 * rand::random::<f64>() - TRIANGLE_DISPLACEMENT,
                    TRIANGLE_DISPLACEMENT * 2.0 * rand::random::<f64>() - TRIANGLE_DISPLACEMENT,
                    TRIANGLE_DISPLACEMENT * 2.0 * rand::random::<f64>() - TRIANGLE_DISPLACEMENT,
                ),
                pt + Vector3::new(
                    TRIANGLE_DISPLACEMENT * 2.0 * rand::random::<f64>() - TRIANGLE_DISPLACEMENT,
                    TRIANGLE_DISPLACEMENT * 2.0 * rand::random::<f64>() - TRIANGLE_DISPLACEMENT,
                    TRIANGLE_DISPLACEMENT * 2.0 * rand::random::<f64>() - TRIANGLE_DISPLACEMENT,
                ),
            ]
        })
        .collect::<Vec<_>>();
    let hashed = HashedPointCloud::from_points(&points, 1.0);
    let dist_0 = triangles.iter().fold(None, |dist, triangle| {
        match (dist, hashed.distance(*triangle)) {
            (Some(a), Some(b)) => Some(f64::min(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    });
    let dist_12 = points.iter().fold(std::f64::INFINITY, |dist2, pt| {
        triangles.iter().fold(dist2, |dist2, triangle| {
            f64::min(dist2, distance2_point_triangle(*pt, *triangle))
        })
    });
    let dist_1 = f64::sqrt(dist_12);
    match dist_0 {
        Some(dist_0) => assert_eq!(dist_0, dist_1),
        None => assert!(dist_1 > 10.0),
    }
}

#[test]
fn space_division_distance() { (0..10).for_each(|_| exec_space_division_distance()) }
