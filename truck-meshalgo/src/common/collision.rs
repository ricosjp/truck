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

fn sort_endpoints<I>(iter0: I, iter1: I) -> Vec<EndPoint>
where I: IntoIterator<Item = [Point3; 3]> {
    let unit = take_one_unit();
    let mut res: Vec<EndPoint> = iter0
        .into_iter()
        .enumerate()
        .flat_map(|(i, tri)| {
            let seg = tri_to_seg(tri, unit);
            vec![
                EndPoint {
                    entity: seg.0,
                    r#type: EndPointType::Front,
                    segnum: 0,
                    index: i,
                },
                EndPoint {
                    entity: seg.1,
                    r#type: EndPointType::Back,
                    segnum: 0,
                    index: i,
                },
            ]
        })
        .chain(iter1.into_iter().enumerate().flat_map(|(i, tri)| {
            let seg = tri_to_seg(tri, unit);
            vec![
                EndPoint {
                    entity: seg.0,
                    r#type: EndPointType::Front,
                    segnum: 1,
                    index: i,
                },
                EndPoint {
                    entity: seg.1,
                    r#type: EndPointType::Back,
                    segnum: 1,
                    index: i,
                },
            ]
        }))
        .collect();
    res.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater));
    res
}

fn colliding_segment_pairs(sort_endpoints: Vec<EndPoint>) -> Vec<(usize, usize)> {
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
    res
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

fn collide_seg_triangle(seg: [Point3; 2], tri: [Point3; 3]) -> bool {
    let ab = tri[1] - tri[0];
    let bc = tri[2] - tri[1];
    let ca = tri[0] - tri[2];
    let nor = ab.cross(ca);
    if nor.so_small() {
        return false;
    }
    let ap = seg[0] - tri[0];
    let aq = seg[1] - tri[0];
    let dotapnor = ap.dot(nor);
    let dotaqnor = aq.dot(nor);
    if dotapnor * dotaqnor > 0.0 {
        return false;
    }
    let h = seg[0] + dotapnor / (dotapnor - dotaqnor) * (seg[1] - seg[0]);
    f64::signum(ab.cross(nor).dot(h - tri[0]))
        + f64::signum(bc.cross(nor).dot(h - tri[1]))
        + f64::signum(ca.cross(nor).dot(h - tri[2]))
        > 2.0
}

fn collide_triangles(tri0: [Point3; 3], tri1: [Point3; 3]) -> bool {
    collide_seg_triangle([tri0[0], tri0[1]], tri1)
    || collide_seg_triangle([tri0[1], tri0[2]], tri1)
    || collide_seg_triangle([tri0[2], tri0[0]], tri1)
    || collide_seg_triangle([tri1[0], tri1[1]], tri0)
    || collide_seg_triangle([tri1[1], tri1[2]], tri0)
    || collide_seg_triangle([tri1[2], tri1[0]], tri0)
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
    assert!(collide_triangles(tri0, tri1));
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
    assert!(!collide_triangles(tri0, tri1));
}
