use super::*;

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

fn take_one_unit() -> Vector3 {
    loop {
        let normal = Vector3::new(
            2.0 * rand::random::<f64>() - 1.0,
            2.0 * rand::random::<f64>() - 1.0,
            2.0 * rand::random::<f64>() - 1.0,
        );
        if !normal.so_small() {
            return normal.normalize();
        }
    }
}

fn tri_to_seg(tri: [Point3; 3], unit: Vector3) -> (f64, f64) {
    let a = tri[0].to_vec().dot(unit);
    let b = tri[1].to_vec().dot(unit);
    let c = tri[2].to_vec().dot(unit);
    (f64::min(f64::min(a, b), c), f64::max(f64::max(a, b), c))
}

fn sort_endpoints<I, J>(iter0: I, iter1: J) -> Vec<EndPoint>
where
    I: IntoIterator<Item = [Point3; 3]>,
    J: IntoIterator<Item = [Point3; 3]>, {
    let unit = take_one_unit();
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
    let mut res = Vec::new();
    sort_endpoints.into_iter().for_each(
        |EndPoint {
             r#type,
             segnum,
             index,
             ..
         }| match r#type {
            EndPointType::Front => {
                current[segnum].push(index);
                current[1 - segnum].iter().for_each(|i| {
                    if segnum == 0 {
                        res.push((index, *i));
                    } else {
                        res.push((*i, index));
                    }
                })
            }
            EndPointType::Back => {
                let i = current[segnum]
                    .iter()
                    .enumerate()
                    .find(|(_, idx)| **idx == index)
                    .unwrap()
                    .0;
                current[segnum].swap_remove(i);
            }
        },
    );
    res.into_iter()
}

#[allow(dead_code)]
fn colliding_segment_pairs2<I, J>(iter0: I, iter1: J) -> impl Iterator<Item = (usize, usize)>
where
    I: IntoIterator<Item = [Point3; 3]>,
    J: IntoIterator<Item = [Point3; 3]>, {
    let instant = std::time::Instant::now();
    let mut x_segs = Vec::new();
    let mut y_segs = Vec::new();
    let mut z_segs = Vec::new();
    let mut count = 0;
    iter0
        .into_iter()
        .enumerate()
        .filter(|(_, tri)| {
            count += 1;
            !(tri[1] - tri[0]).cross(tri[2] - tri[0]).so_small()
        })
        .for_each(|(i, tri)| {
            let bdb: BoundingBox<Point3> = tri.iter().collect();
            x_segs.extend(EndPoint::from_seg((bdb.min()[0], bdb.max()[0]), 0, i));
            y_segs.extend(EndPoint::from_seg((bdb.min()[1], bdb.max()[1]), 0, i));
            z_segs.extend(EndPoint::from_seg((bdb.min()[2], bdb.max()[2]), 0, i));
        });
    iter1
        .into_iter()
        .enumerate()
        .filter(|(_, tri)| !(tri[1] - tri[0]).cross(tri[2] - tri[0]).so_small())
        .for_each(|(i, tri)| {
            let bdb: BoundingBox<Point3> = tri.iter().collect();
            x_segs.extend(EndPoint::from_seg((bdb.min()[0], bdb.max()[0]), 1, i));
            y_segs.extend(EndPoint::from_seg((bdb.min()[1], bdb.max()[1]), 1, i));
            z_segs.extend(EndPoint::from_seg((bdb.min()[2], bdb.max()[2]), 1, i));
        });
    println!("segs creation:    {}s", instant.elapsed().as_secs_f64());
    let instant = std::time::Instant::now();
    x_segs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater));
    y_segs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater));
    z_segs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater));
    println!("segs sort:        {}s", instant.elapsed().as_secs_f64());
    let instant = std::time::Instant::now();
    let mut tank0 = vec![std::collections::HashSet::new(); count];
    let mut tank1 = vec![std::collections::HashSet::new(); count];
    let mut res = vec![Vec::new(); count];
    let mut current = [Vec::<usize>::new(), Vec::<usize>::new()];
    x_segs.into_iter().for_each(
        |EndPoint {
             r#type,
             segnum,
             index,
             ..
         }| match r#type {
            EndPointType::Front => {
                current[segnum].push(index);
                current[1 - segnum].iter().for_each(|i| {
                    if segnum == 0 {
                        tank0[index].insert(*i);
                    } else {
                        tank1[*i].insert(index);
                    }
                })
            }
            EndPointType::Back => {
                let i = current[segnum]
                    .iter()
                    .enumerate()
                    .find(|(_, idx)| **idx == index)
                    .unwrap()
                    .0;
                current[segnum].swap_remove(i);
            }
        },
    );
    current[0].clear();
    current[1].clear();
    y_segs.into_iter().for_each(
        |EndPoint {
             r#type,
             segnum,
             index,
             ..
         }| match r#type {
            EndPointType::Front => {
                current[segnum].push(index);
                current[1 - segnum].iter().for_each(|i| {
                    let (i, j) = if segnum == 0 {
                        (index, *i)
                    } else {
                        (*i, index)
                    };
                    if tank0[i].remove(&j) {
                        tank1[i].insert(j);
                    }
                })
            }
            EndPointType::Back => {
                let i = current[segnum]
                    .iter()
                    .enumerate()
                    .find(|(_, idx)| **idx == index)
                    .unwrap()
                    .0;
                current[segnum].swap_remove(i);
            }
        },
    );
    current[0].clear();
    current[1].clear();
    z_segs.into_iter().for_each(
        |EndPoint {
             r#type,
             segnum,
             index,
             ..
         }| match r#type {
            EndPointType::Front => {
                current[segnum].push(index);
                current[1 - segnum].iter().for_each(|i| {
                    let (i, j) = if segnum == 0 {
                        (index, *i)
                    } else {
                        (*i, index)
                    };
                    if tank1[i].remove(&j) {
                        res[i].push(j);
                    }
                })
            }
            EndPointType::Back => {
                let i = current[segnum]
                    .iter()
                    .enumerate()
                    .find(|(_, idx)| **idx == index)
                    .unwrap()
                    .0;
                current[segnum].swap_remove(i);
            }
        },
    );
    println!("segs merge:       {}s", instant.elapsed().as_secs_f64());
    res.into_iter()
        .enumerate()
        .flat_map(move |(i, v)| v.into_iter().map(move |j| (i, j)))
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

pub fn collision(poly0: &PolygonMesh, poly1: &PolygonMesh) -> Vec<(Point3, Point3)> {
    let tris0 = Triangulate::new(poly0);
    let tris1 = Triangulate::new(poly1);
    let iter0 = tris0.into_iter().map(|face| {
        [
            poly0.positions()[face[0].pos],
            poly0.positions()[face[1].pos],
            poly0.positions()[face[2].pos],
        ]
    });
    let iter1 = tris1.into_iter().map(|face| {
        [
            poly1.positions()[face[0].pos],
            poly1.positions()[face[1].pos],
            poly1.positions()[face[2].pos],
        ]
    });
    colliding_segment_pairs(sort_endpoints(iter0, iter1))
//    colliding_segment_pairs2(iter0, iter1)
        .filter_map(|(idx0, idx1)| {
            let face0 = tris0.get(idx0);
            let tri0 = [
                poly0.positions()[face0[0].pos],
                poly0.positions()[face0[1].pos],
                poly0.positions()[face0[2].pos],
            ];
            let face1 = tris1.get(idx1);
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
