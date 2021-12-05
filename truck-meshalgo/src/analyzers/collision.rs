use super::*;

/// Find collisions between two polygon meshes and extract interference lines.
///
/// # Details
///
/// The algorithm for mesh extraction is as follows.
/// All the triangles are projected toward the axis to form intervals.
/// By looking at the overlap of these intervals, we can narrow down the interfering triangles.
/// The overlap of the intervals is obtained by sorting the endpoints.
///
/// # Remarks
///
/// We see that the surfaces that are in contact are not interfering with each other.
/// Therefore, polygon meshes included in one plane are not interfering.
pub trait Collision {
    /// If `self` and `other` collide, then returns only one interference line.
    /// Otherewise, returns `None`.
    fn collide_with(&self, other: &PolygonMesh) -> Option<(Point3, Point3)>;
    /// Extract all interference lines between `self` and `other`.
    /// # Remarks
    /// The results is not arranged so that included lines make continuous maximal polyline curve.
    fn extract_interference(&self, other: &PolygonMesh) -> Vec<(Point3, Point3)>;
}

impl Collision for PolygonMesh {
    #[inline(always)]
    fn collide_with(&self, other: &PolygonMesh) -> Option<(Point3, Point3)> {
        are_colliding(self, other)
    }
    #[inline(always)]
    fn extract_interference(&self, other: &PolygonMesh) -> Vec<(Point3, Point3)> {
        collision(self, other)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EndPointType {
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct EndPoint {
    entity: f64,
    r#type: EndPointType,
    segnum: usize,
    index: usize,
}

impl EndPoint {
    #[inline(always)]
    fn new(entity: f64, r#type: EndPointType, segnum: usize, index: usize) -> EndPoint {
        EndPoint {
            entity,
            r#type,
            segnum,
            index,
        }
    }
    #[inline(always)]
    fn from_seg(seg: (f64, f64), segnum: usize, index: usize) -> Vec<EndPoint> {
        vec![
            EndPoint::new(seg.0, EndPointType::Front, segnum, index),
            EndPoint::new(seg.1, EndPointType::Back, segnum, index),
        ]
    }
}

fn tri_to_seg(tri: [Point3; 3], unit: Vector3) -> (f64, f64) {
    let a = tri[0].to_vec().dot(unit);
    let b = tri[1].to_vec().dot(unit);
    let c = tri[2].to_vec().dot(unit);
    (f64::min(f64::min(a, b), c), f64::max(f64::max(a, b), c))
}

fn sorted_endpoints<I, J>(iter0: I, iter1: J, unit: Vector3) -> Vec<EndPoint>
where
    I: IntoIterator<Item = [Point3; 3]>,
    J: IntoIterator<Item = [Point3; 3]>, {
    let mut res: Vec<EndPoint> = iter0
        .into_iter()
        .enumerate()
        .filter(|(_, tri)| !(tri[1] - tri[0]).cross(tri[2] - tri[0]).so_small())
        .flat_map(|(i, tri)| EndPoint::from_seg(tri_to_seg(tri, unit), 0, i))
        .chain(
            iter1
                .into_iter()
                .enumerate()
                .filter(|(_, tri)| !(tri[1] - tri[0]).cross(tri[2] - tri[0]).so_small())
                .flat_map(|(i, tri)| EndPoint::from_seg(tri_to_seg(tri, unit), 1, i)),
        )
        .collect();
    res.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater));
    res
}

fn colliding_segment_pairs(sort_endpoints: Vec<EndPoint>) -> impl Iterator<Item = (usize, usize)> {
    let mut current = [Vec::<usize>::new(), Vec::<usize>::new()];
    sort_endpoints
        .into_iter()
        .filter_map(
            move |EndPoint {
                      r#type,
                      segnum,
                      index,
                      ..
                  }| match r#type {
                EndPointType::Front => {
                    current[segnum].push(index);
                    Some(current[1 - segnum].clone().into_iter().map(move |i| {
                        if segnum == 0 {
                            (index, i)
                        } else {
                            (i, index)
                        }
                    }))
                }
                EndPointType::Back => {
                    let i = current[segnum]
                        .iter()
                        .enumerate()
                        .find(|(_, idx)| **idx == index)
                        .unwrap()
                        .0;
                    current[segnum].swap_remove(i);
                    None
                }
            },
        )
        .flatten()
}

fn disjoint_bdbs(tri0: [Point3; 3], tri1: [Point3; 3]) -> bool {
    let bdb0: BoundingBox<Point3> = tri0.iter().collect();
    let bdb1: BoundingBox<Point3> = tri1.iter().collect();
    bdb0.max()[0] < bdb1.min()[0]
        || bdb1.max()[0] < bdb0.min()[0]
        || bdb0.max()[1] < bdb1.min()[1]
        || bdb1.max()[1] < bdb0.min()[1]
        || bdb0.max()[2] < bdb1.min()[2]
        || bdb1.max()[2] < bdb0.min()[2]
}

fn collide_seg_triangle(seg: [Point3; 2], tri: [Point3; 3]) -> Option<Point3> {
    let ab = tri[1] - tri[0];
    let bc = tri[2] - tri[1];
    let ca = tri[0] - tri[2];
    let nor = ab.cross(ca);
    if nor.so_small() {
        return None;
    }
    let ap = seg[0] - tri[0];
    let aq = seg[1] - tri[0];
    let dotapnor = ap.dot(nor);
    let dotaqnor = aq.dot(nor);
    if dotapnor * dotaqnor > 0.0 {
        return None;
    }
    let h = seg[0] + dotapnor / (dotapnor - dotaqnor) * (seg[1] - seg[0]);
    if f64::signum(ab.cross(nor).dot(h - tri[0]) + TOLERANCE2)
        + f64::signum(bc.cross(nor).dot(h - tri[1]) + TOLERANCE2)
        + f64::signum(ca.cross(nor).dot(h - tri[2]) + TOLERANCE2)
        >= 2.0
    {
        Some(h)
    } else {
        None
    }
}

fn collide_triangles(tri0: [Point3; 3], tri1: [Point3; 3]) -> Option<(Point3, Point3)> {
    let mut tuple = (None, None);
    [
        collide_seg_triangle([tri0[0], tri0[1]], tri1),
        collide_seg_triangle([tri0[1], tri0[2]], tri1),
        collide_seg_triangle([tri0[2], tri0[0]], tri1),
        collide_seg_triangle([tri1[0], tri1[1]], tri0),
        collide_seg_triangle([tri1[1], tri1[2]], tri0),
        collide_seg_triangle([tri1[2], tri1[0]], tri0),
    ]
    .iter()
    .for_each(|pt| match tuple {
        (None, _) => tuple.0 = *pt,
        (Some(_), None) => tuple.1 = *pt,
        (Some(ref mut p), Some(ref mut q)) => {
            if let Some(pt) = pt {
                let dist0 = pt.distance2(*p);
                let dist1 = pt.distance2(*q);
                let dist2 = p.distance2(*q);
                if dist2 < dist0 {
                    *q = *pt;
                } else if dist2 < dist1 {
                    *p = *pt;
                }
            }
        }
    });
    match tuple {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None,
    }
}

fn collision(poly0: &PolygonMesh, poly1: &PolygonMesh) -> Vec<(Point3, Point3)> {
    let unit = match poly0.positions().is_empty() {
        true => return Vec::new(),
        false => hash::take_one_unit(poly0.positions()[0]),
    };
    let tris0 = poly0.faces().triangle_iter().collect::<Vec<_>>();
    let tris1 = poly1.faces().triangle_iter().collect::<Vec<_>>();
    let iter0 = tris0.iter().map(|face| {
        [
            poly0.positions()[face[0].pos],
            poly0.positions()[face[1].pos],
            poly0.positions()[face[2].pos],
        ]
    });
    let iter1 = tris1.iter().map(|face| {
        [
            poly1.positions()[face[0].pos],
            poly1.positions()[face[1].pos],
            poly1.positions()[face[2].pos],
        ]
    });
    colliding_segment_pairs(sorted_endpoints(iter0, iter1, unit))
        .filter_map(|(idx0, idx1)| {
            let face0 = tris0[idx0];
            let tri0 = [
                poly0.positions()[face0[0].pos],
                poly0.positions()[face0[1].pos],
                poly0.positions()[face0[2].pos],
            ];
            let face1 = tris1[idx1];
            let tri1 = [
                poly1.positions()[face1[0].pos],
                poly1.positions()[face1[1].pos],
                poly1.positions()[face1[2].pos],
            ];
            if disjoint_bdbs(tri0, tri1) {
                None
            } else {
                collide_triangles(tri0, tri1)
            }
        })
        .collect()
}

fn are_colliding(poly0: &PolygonMesh, poly1: &PolygonMesh) -> Option<(Point3, Point3)> {
    let unit = match poly0.positions().is_empty() {
        true => return None,
        false => hash::take_one_unit(poly0.positions()[0]),
    };
    let tris0 = poly0.faces().triangle_iter().collect::<Vec<_>>();
    let tris1 = poly1.faces().triangle_iter().collect::<Vec<_>>();
    let iter0 = tris0.iter().map(|face| {
        [
            poly0.positions()[face[0].pos],
            poly0.positions()[face[1].pos],
            poly0.positions()[face[2].pos],
        ]
    });
    let iter1 = tris1.iter().map(|face| {
        [
            poly1.positions()[face[0].pos],
            poly1.positions()[face[1].pos],
            poly1.positions()[face[2].pos],
        ]
    });
    colliding_segment_pairs(sorted_endpoints(iter0, iter1, unit)).find_map(|(idx0, idx1)| {
        let face0 = tris0[idx0];
        let tri0 = [
            poly0.positions()[face0[0].pos],
            poly0.positions()[face0[1].pos],
            poly0.positions()[face0[2].pos],
        ];
        let face1 = tris1[idx1];
        let tri1 = [
            poly1.positions()[face1[0].pos],
            poly1.positions()[face1[1].pos],
            poly1.positions()[face1[2].pos],
        ];
        if disjoint_bdbs(tri0, tri1) {
            None
        } else {
            collide_triangles(tri0, tri1)
        }
    })
}

#[test]
fn collide_triangles_test() {
    let tri0 = [
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let tri1 = [
        Point3::new(0.0, 0.0, -1.0),
        Point3::new(-1.0, -1.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
    ];
    assert!(collide_triangles(tri0, tri1).is_some());
    let tri0 = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let tri1 = [
        Point3::new(0.0, 0.0, 0.5),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
    ];
    assert!(collide_triangles(tri0, tri1).is_none());
}
