use super::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EndPointType {
    Front,
    Middle,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct EndPoint {
    entity: f64,
    r#type: EndPointType,
    index: usize,
}

impl EndPoint {
    #[inline(always)]
    fn new(entity: f64, r#type: EndPointType, index: usize) -> EndPoint {
        EndPoint {
            entity,
            r#type,
            index,
        }
    }
    #[inline(always)]
    fn from_seg(seg: (f64, f64), index: usize) -> Vec<EndPoint> {
        vec![
            EndPoint::new(seg.0, EndPointType::Front, index),
            EndPoint::new(seg.1, EndPointType::Back, index),
        ]
    }
}

fn tri_to_seg(tri: [Point3; 3], unit: Vector3, tol: f64) -> (f64, f64) {
    let a = tri[0].to_vec().dot(unit);
    let b = tri[1].to_vec().dot(unit);
    let c = tri[2].to_vec().dot(unit);
    (
        f64::min(f64::min(a, b), c) - tol,
        f64::max(f64::max(a, b), c) + tol,
    )
}

fn sorted_endpoints<'a, I, J>(iter0: I, iter1: J, unit: Vector3, tol: f64) -> Vec<EndPoint>
where
    I: IntoIterator<Item = [Point3; 3]>,
    J: IntoIterator<Item = &'a Point3>, {
    let mut res: Vec<EndPoint> = iter0
        .into_iter()
        .enumerate()
        .filter(|(_, tri)| !(tri[1] - tri[0]).cross(tri[2] - tri[0]).so_small())
        .flat_map(|(i, tri)| EndPoint::from_seg(tri_to_seg(tri, unit, tol), i))
        .chain(
            iter1
                .into_iter()
                .enumerate()
                .map(|(i, point)| EndPoint::new(point.to_vec().dot(unit), EndPointType::Middle, i)),
        )
        .collect();
    res.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater));
    res
}

fn sorted_endpoints_by_polymesh_points(
    polygon: &PolygonMesh,
    points: &[Point3],
    tol: f64,
) -> Vec<EndPoint> {
    let unit = if polygon.positions().len() != 0 {
        hash::take_one_unit(polygon.positions()[0])
    } else if points.len() != 0 {
        hash::take_one_unit(points[0])
    } else {
        return Vec::new();
    };
    sorted_endpoints(
        polygon.faces().triangle_iter().map(|tri| {
            [
                polygon.positions()[tri[0].pos],
                polygon.positions()[tri[1].pos],
                polygon.positions()[tri[2].pos],
            ]
        }),
        points.iter(),
        unit,
        tol,
    )
}

pub fn pointcloud_in_polygon_neighborhood(
    polygon: &PolygonMesh,
    points: &[Point3],
    tol: f64,
) -> bool {
    nonpositive_tolerance!(tol, 0.0);
    let mut current = Vec::new();
    let triangles = polygon.faces().triangle_iter().collect::<Vec<_>>();
    sorted_endpoints_by_polymesh_points(polygon, points, tol)
        .into_iter()
        .all(move |EndPoint { r#type, index, .. }| match r#type {
            EndPointType::Front => {
                current.push(index);
                true
            }
            EndPointType::Back => {
                let i = current
                    .iter()
                    .enumerate()
                    .find(|(_, idx)| **idx == index)
                    .unwrap()
                    .0;
                current.swap_remove(i);
                true
            }
            EndPointType::Middle => current.iter().any(|i| {
                let tri = triangles[*i];
                let tri = [
                    polygon.positions()[tri[0].pos],
                    polygon.positions()[tri[1].pos],
                    polygon.positions()[tri[2].pos],
                ];
                distance2_point_triangle(points[index], tri) < tol * tol
            }),
        })
}
