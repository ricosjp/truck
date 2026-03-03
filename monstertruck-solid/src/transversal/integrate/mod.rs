use crate::alternative::Alternative;
use crate::healing::RobustSplitClosedEdgesAndFaces;

use super::*;
use monstertruck_geometry::prelude::*;
use monstertruck_meshing::prelude::*;
use monstertruck_topology::{errors::Error as TopologyError, shell::ShellCondition, *};
use rustc_hash::FxHashSet as HashSet;
use thiserror::Error;

/// Only solids consisting of faces whose surface is implemented this trait can be used for set operations.
pub trait ShapeOpsSurface:
    ParametricSurface3D
    + ParameterDivision2D
    + SearchParameter<D2, Point = Point3>
    + SearchNearestParameter<D2, Point = Point3>
    + Clone
    + Invertible
    + Send
    + Sync {
}
impl<S> ShapeOpsSurface for S where S: ParametricSurface3D
        + ParameterDivision2D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>
        + Clone
        + Invertible
        + Send
        + Sync
{
}

/// Only solids consisting of edges whose curve is implemented this trait can be used for set operations.
pub trait ShapeOpsCurve<S: ShapeOpsSurface>:
    ParametricCurve3D
    + ParameterDivision1D<Point = Point3>
    + Cut
    + Clone
    + TryFrom<ParameterCurve<Line<Point2>, S>>
    + Invertible
    + From<IntersectionCurve<BsplineCurve<Point3>, S, S>>
    + SearchParameter<D1, Point = Point3>
    + SearchNearestParameter<D1, Point = Point3>
    + Send
    + Sync {
}
impl<C, S: ShapeOpsSurface> ShapeOpsCurve<S> for C where C: ParametricCurve3D
        + ParameterDivision1D<Point = Point3>
        + Cut
        + Clone
        + TryFrom<ParameterCurve<Line<Point2>, S>>
        + Invertible
        + From<IntersectionCurve<BsplineCurve<Point3>, S, S>>
        + SearchParameter<D1, Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + Send
        + Sync
{
}

/// Errors for boolean shape operations.
#[derive(Debug, Error)]
pub enum ShapeOpsError {
    /// `tol` was not positive enough for robust meshing and projection.
    #[error("`tol` must be at least `TOLERANCE`.")]
    InvalidTolerance,
    /// Building intersection loops failed.
    #[error("failed to build intersection loops: {source}")]
    CreateLoopsStoreFailed {
        /// Detailed loop-store creation error.
        #[source]
        source: loops_store::CreateLoopsStoreError,
    },
    /// Face division failed for one shell.
    #[error("failed to divide faces for shell {shell_index}.")]
    DivideFacesFailed {
        /// 0 for the first shell, 1 for the second shell.
        shell_index: usize,
    },
    /// Unknown face classification failed for one shell.
    #[error("failed to classify unknown faces for shell {shell_index}.")]
    UnknownClassificationFailed {
        /// 0 for the first shell, 1 for the second shell.
        shell_index: usize,
    },
    /// Converting temporary intersection curves back to target curves failed.
    #[error("failed to convert temporary shell for `{operation}`.")]
    AltShellConversionFailed {
        /// `and` or `or`.
        operation: &'static str,
    },
    /// The generated shell failed manifold checks before solid construction.
    #[error(transparent)]
    InvalidOutputShellCondition(Box<InvalidOutputShellConditionData>),
    /// The output has no boundary shells.
    #[error("invalid output shell for `{operation}`: no boundary shells.")]
    EmptyOutputShell {
        /// Boolean operation name.
        operation: &'static str,
    },
    /// The generated shell is topologically invalid.
    #[error("invalid output shell for `{operation}`: {source}.")]
    InvalidOutputShell {
        /// Boolean operation name.
        operation: &'static str,
        /// Topology validation error.
        #[source]
        source: TopologyError,
    },
}

/// Diagnostic data for invalid output shell conditions.
#[derive(Debug, Error)]
#[error(
    "invalid output shell for `{operation}` at index {shell_index}: empty={empty}, connected={connected}, condition={condition:?}, boundary_loops={boundary_loops}, first_boundary_len={first_boundary_len:?}, first_boundary_front={first_boundary_front:?}, first_boundary_back={first_boundary_back:?}, singular_vertices={singular_vertices}, first_singular={first_singular:?}."
)]
pub struct InvalidOutputShellConditionData {
    /// Boolean operation name.
    pub operation: &'static str,
    /// Boundary shell index.
    pub shell_index: usize,
    /// Whether shell has no faces.
    pub empty: bool,
    /// Whether shell is topologically connected.
    pub connected: bool,
    /// Evaluated shell condition.
    pub condition: ShellCondition,
    /// Count of extracted open boundary wires.
    pub boundary_loops: usize,
    /// Number of edges in first open boundary wire.
    pub first_boundary_len: Option<usize>,
    /// Front point of first open boundary wire.
    pub first_boundary_front: Option<Point3>,
    /// Back point of first open boundary wire.
    pub first_boundary_back: Option<Point3>,
    /// Number of singular vertices.
    pub singular_vertices: usize,
    /// First singular vertex point if present.
    pub first_singular: Option<Point3>,
}

type ShapeOpsResult<T> = std::result::Result<T, ShapeOpsError>;

type AltCurveShell<C, S> =
    Shell<Point3, Alternative<C, IntersectionCurve<PolylineCurve<Point3>, S, S>>, S>;

fn classify_inside_with_polyshell(
    poly_shell: &Shell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>,
    pt: Point3,
) -> Option<bool> {
    let offsets = [
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.613, -0.271, 0.149),
        Vector3::new(-0.347, 0.509, -0.221),
        Vector3::new(0.193, 0.401, 0.577),
        Vector3::new(-0.433, -0.127, 0.389),
    ];
    let (inside_votes, outside_votes) = offsets
        .into_iter()
        .map(|offset| hash::take_one_unit(pt + offset))
        .try_fold((0usize, 0usize), |(inside, outside), dir| {
            let count = poly_shell.iter().try_fold(0isize, |count, face| {
                let poly = face.surface()?;
                Some(count + poly.signed_crossing_faces(pt, dir))
            })?;
            match count == 0 {
                true => Some((inside, outside + 1)),
                false => Some((inside + 1, outside)),
            }
        })?;
    Some(inside_votes > outside_votes)
}

fn sample_points_on_face<C, S>(face: &Face<Point3, C, S>) -> Option<Vec<Point3>> {
    let wire = face.absolute_boundaries().first()?;
    let vertices: Vec<_> = wire.vertex_iter().map(|v| v.point()).collect();
    let (sum, count) = vertices
        .iter()
        .fold((Vector3::new(0.0, 0.0, 0.0), 0usize), |(sum, count), pt| {
            (sum + pt.to_vec(), count + 1)
        });
    if count == 0 {
        return None;
    }
    let centroid = Point3::from_vec(sum / count as f64);
    Some(
        std::iter::once(centroid)
            .chain(vertices.into_iter().map(|v| centroid.midpoint(v)))
            .collect(),
    )
}

fn classify_unknown_face<C, S>(
    poly_shell: &Shell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>,
    face: &Face<Point3, C, S>,
) -> Option<bool> {
    let points = sample_points_on_face(face)?;
    let (inside, outside) =
        points
            .into_iter()
            .try_fold((0usize, 0usize), |(inside, outside), pt| {
                if classify_inside_with_polyshell(poly_shell, pt)? {
                    Some((inside + 1, outside))
                } else {
                    Some((inside, outside + 1))
                }
            })?;
    Some(inside >= outside)
}

fn altshell_to_shell<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    altshell: &AltCurveShell<C, S>,
    tol: f64,
) -> Option<Shell<Point3, C, S>> {
    altshell.try_mapped(
        |p| Some(*p),
        |c| match c {
            Alternative::FirstType(c) => Some(c.clone()),
            Alternative::SecondType(ic) => {
                let bsp = BsplineCurve::quadratic_approximation(ic, ic.range_tuple(), tol, 100)?;
                Some(
                    IntersectionCurve::new(ic.surface0().clone(), ic.surface1().clone(), bsp)
                        .into(),
                )
            }
        },
        |s| Some(s.clone()),
    )
}

fn heal_shell_if_needed<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    shell: Shell<Point3, C, S>,
    tol: f64,
) -> Option<Shell<Point3, C, S>> {
    if shell.shell_condition() == ShellCondition::Closed && shell.singular_vertices().is_empty() {
        return Some(shell);
    }
    let mut compressed = shell.compress();
    compressed.robust_split_closed_edges_and_faces(tol);
    Shell::extract(compressed).ok()
}

fn shell_condition_rank(condition: ShellCondition) -> usize {
    match condition {
        ShellCondition::Closed => 0,
        ShellCondition::Oriented => 1,
        ShellCondition::Regular => 2,
        ShellCondition::Irregular => 3,
    }
}

fn shell_quality<C, S>(shell: &Shell<Point3, C, S>) -> (usize, usize, usize) {
    (
        shell_condition_rank(shell.shell_condition()),
        shell.extract_boundaries().len(),
        shell.singular_vertices().len(),
    )
}

fn try_cap_shell_with_existing_surfaces<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    shell: Shell<Point3, C, S>,
    tol: f64,
) -> Shell<Point3, C, S> {
    let debug_cap = std::env::var("MT_BOOL_DEBUG_CAP").is_ok();
    let mut capped = shell;
    let boundaries = capped.extract_boundaries();
    boundaries.into_iter().for_each(|wire| {
        let edge_ids: Vec<_> = wire.edge_iter().map(|edge| edge.id()).collect();
        let mut candidate_surfaces = Vec::new();
        capped.iter().for_each(|face| {
            let face_edge_ids: HashSet<_> = face.edge_iter().map(|edge| edge.id()).collect();
            if edge_ids
                .iter()
                .any(|edge_id| face_edge_ids.contains(edge_id))
            {
                candidate_surfaces.push(face.surface().clone());
            }
        });
        let current_quality = shell_quality(&capped);
        let mut candidate_count = 0usize;
        let mut shell_triangulatable_count = 0usize;
        let best = candidate_surfaces
            .into_iter()
            .flat_map(|surface| {
                [wire.clone(), wire.inverse()]
                    .into_iter()
                    .filter_map(move |boundary| Face::try_new(vec![boundary], surface.clone()).ok())
            })
            .inspect(|_| candidate_count += 1)
            .map(|face| {
                let mut candidate = capped.clone();
                candidate.push(face.clone());
                let triangulatable = candidate
                    .triangulation(f64::max(10.0 * TOLERANCE, tol))
                    .face_iter()
                    .all(|f| f.surface().is_some());
                if triangulatable {
                    shell_triangulatable_count += 1;
                }
                (shell_quality(&candidate), triangulatable, face)
            })
            .filter(|(_, triangulatable, _)| *triangulatable)
            .min_by_key(|(quality, _, _)| *quality)
            .filter(|(quality, _, _)| *quality < current_quality)
            .map(|(_, _, face)| face);
        if debug_cap {
            eprintln!(
                "debug cap boundary_len={} current={:?} candidates={} shell_triangulatable={} picked={}",
                wire.len(),
                current_quality,
                candidate_count,
                shell_triangulatable_count,
                best.is_some(),
            );
        }
        if let Some(face) = best {
            capped.push(face);
        }
    });
    capped
}

fn process_one_pair_of_shells<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    shell0: &Shell<Point3, C, S>,
    shell1: &Shell<Point3, C, S>,
    tol: f64,
) -> ShapeOpsResult<[Shell<Point3, C, S>; 2]> {
    type ShellQualityScore = (usize, usize, usize, usize);
    type BooleanQualityScore = (ShellQualityScore, ShellQualityScore);

    let debug_bool = std::env::var("MT_BOOL_DEBUG_COUNTS").is_ok();
    if tol < TOLERANCE {
        return Err(ShapeOpsError::InvalidTolerance);
    }

    let poly_tol = f64::max(tol * 0.25, 2.0 * TOLERANCE);
    let poly_shell0 = shell0.triangulation(poly_tol);
    let poly_shell1 = shell1.triangulation(poly_tol);
    let altshell0: AltCurveShell<C, S> =
        shell0.mapped(|x| *x, |c| Alternative::FirstType(c.clone()), Clone::clone);
    let altshell1: AltCurveShell<C, S> =
        shell1.mapped(|x| *x, |c| Alternative::FirstType(c.clone()), Clone::clone);

    let loops_store::LoopsStoreQuadruple {
        geom_loops_store0: loops_store0,
        geom_loops_store1: loops_store1,
        ..
    } = loops_store::create_loops_stores_with_tolerance(
        &altshell0,
        &poly_shell0,
        &altshell1,
        &poly_shell1,
        poly_tol,
    )
    .map_err(|source| ShapeOpsError::CreateLoopsStoreFailed { source })?;

    let mut cls0 = divide_face::divide_faces(&altshell0, &loops_store0, tol)
        .ok_or(ShapeOpsError::DivideFacesFailed { shell_index: 0 })?;
    cls0.integrate_by_component();

    let mut cls1 = divide_face::divide_faces(&altshell1, &loops_store1, tol)
        .ok_or(ShapeOpsError::DivideFacesFailed { shell_index: 1 })?;
    cls1.integrate_by_component();

    let [and0, or0, unknown0] = cls0.and_or_unknown();
    if debug_bool {
        eprintln!(
            "debug class0 and={} or={} unknown={}",
            and0.len(),
            or0.len(),
            unknown0.len()
        );
    }
    let mut unknown_faces = Vec::new();
    unknown0
        .into_iter()
        .try_for_each(|face| {
            unknown_faces.push((face.clone(), classify_unknown_face(&poly_shell1, &face)?));
            Some(())
        })
        .ok_or(ShapeOpsError::UnknownClassificationFailed { shell_index: 0 })?;

    let [mut and1, mut or1, unknown1] = cls1.and_or_unknown();
    if debug_bool {
        eprintln!(
            "debug class1 and={} or={} unknown={}",
            and1.len(),
            or1.len(),
            unknown1.len()
        );
    }
    unknown1
        .into_iter()
        .try_for_each(|face| {
            unknown_faces.push((face.clone(), classify_unknown_face(&poly_shell0, &face)?));
            Some(())
        })
        .ok_or(ShapeOpsError::UnknownClassificationFailed { shell_index: 1 })?;

    let mut known_and = and0;
    known_and.append(&mut and1);
    let mut known_or = or0;
    known_or.append(&mut or1);
    let build_faces = |assignments: &[bool]| {
        let mut and_faces = known_and.clone();
        let mut or_faces = known_or.clone();
        unknown_faces
            .iter()
            .zip(assignments.iter().copied())
            .for_each(|((face, _), is_and)| {
                if is_and {
                    and_faces.push(face.clone());
                } else {
                    or_faces.push(face.clone());
                }
            });
        (and_faces, or_faces)
    };
    let build_raw_shells = |assignments: &[bool]| -> Option<[Shell<Point3, C, S>; 2]> {
        let (and_faces, or_faces) = build_faces(assignments);
        let and_shell = altshell_to_shell(&and_faces, tol)?;
        let or_shell = altshell_to_shell(&or_faces, tol)?;
        Some([and_shell, or_shell])
    };
    let build_shells = |assignments: &[bool]| -> Option<[Shell<Point3, C, S>; 2]> {
        let [and_shell, or_shell] = build_raw_shells(assignments)?;
        let and_shell = heal_shell_if_needed(and_shell, tol)?;
        let or_shell = heal_shell_if_needed(or_shell, tol)?;
        Some([and_shell, or_shell])
    };
    let score = |shell: &Shell<Point3, C, S>| -> ShellQualityScore {
        (
            usize::from(shell.is_empty()),
            shell_condition_rank(shell.shell_condition()),
            shell.extract_boundaries().len(),
            shell.singular_vertices().len(),
        )
    };
    let evaluate = |assignments: &[bool]| -> Option<BooleanQualityScore> {
        let [and_shell, or_shell] = build_raw_shells(assignments)?;
        Some((score(&and_shell), score(&or_shell)))
    };
    let mut assignments: Vec<bool> = unknown_faces.iter().map(|(_, is_and)| *is_and).collect();
    let mut best_score = evaluate(&assignments)
        .ok_or(ShapeOpsError::AltShellConversionFailed { operation: "and" })?;
    let exact_unknown = std::env::var("MT_BOOL_EXACT_UNKNOWN").is_ok();
    if exact_unknown && unknown_faces.len() <= 12 {
        let total = 1usize << unknown_faces.len();
        let mut best_assignments = assignments.clone();
        (0..total).for_each(|mask| {
            let candidate: Vec<bool> = (0..unknown_faces.len())
                .map(|i| ((mask >> i) & 1) == 1)
                .collect();
            if let Some(candidate_score) = evaluate(&candidate)
                && candidate_score < best_score {
                    best_score = candidate_score;
                    best_assignments = candidate;
                }
        });
        assignments = best_assignments;
    } else if unknown_faces.len() <= 24 {
        let mut improved = true;
        while improved {
            improved = false;
            (0..assignments.len()).for_each(|index| {
                let mut candidate = assignments.clone();
                candidate[index] = !candidate[index];
                if let Some(candidate_score) = evaluate(&candidate)
                    && candidate_score < best_score {
                        assignments = candidate;
                        best_score = candidate_score;
                        improved = true;
                    }
            });
        }
    }
    let [and_shell, or_shell] = build_shells(&assignments)
        .ok_or(ShapeOpsError::AltShellConversionFailed { operation: "and" })?;
    if debug_bool {
        let and_count =
            known_and.len() + assignments.iter().copied().filter(|is_and| *is_and).count();
        let or_count = known_or.len() + assignments.len() - (and_count - known_and.len());
        eprintln!(
            "debug class-final and={} or={} score_and={:?} score_or={:?}",
            and_count, or_count, best_score.0, best_score.1,
        );
    }
    if debug_bool {
        eprintln!(
            "debug shell-final and_faces={} and_condition={:?} and_boundary={} and_singular={} | or_faces={} or_condition={:?} or_boundary={} or_singular={}",
            and_shell.len(),
            and_shell.shell_condition(),
            and_shell.extract_boundaries().len(),
            and_shell.singular_vertices().len(),
            or_shell.len(),
            or_shell.shell_condition(),
            or_shell.extract_boundaries().len(),
            or_shell.singular_vertices().len(),
        );
    }

    Ok([and_shell, or_shell])
}

fn try_build_solid<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    operation: &'static str,
    boundaries: Vec<Shell<Point3, C, S>>,
    tol: f64,
) -> ShapeOpsResult<Solid<Point3, C, S>> {
    let boundaries: Vec<_> = boundaries
        .into_iter()
        .map(|shell| try_cap_shell_with_existing_surfaces(shell, tol))
        .collect();
    if boundaries.is_empty() {
        return Err(ShapeOpsError::EmptyOutputShell { operation });
    }
    let is_valid = |shell: &Shell<Point3, C, S>| {
        !shell.is_empty()
            && shell.is_connected()
            && shell.shell_condition() == ShellCondition::Closed
            && shell.singular_vertices().is_empty()
    };
    let valid_boundaries: Vec<_> = boundaries
        .iter()
        .filter(|shell| is_valid(shell))
        .cloned()
        .collect();
    if valid_boundaries.is_empty() {
        let (shell_index, shell) = boundaries
            .iter()
            .enumerate()
            .find(|(_, shell)| !is_valid(shell))
            .ok_or(ShapeOpsError::EmptyOutputShell { operation })?;
        let boundary_loops = shell.extract_boundaries();
        if std::env::var("MT_BOOL_DEBUG_BOUNDARY").is_ok() {
            boundary_loops.iter().enumerate().for_each(|(index, wire)| {
                let points: Vec<_> = wire.vertex_iter().map(|vertex| vertex.point()).collect();
                eprintln!(
                    "debug boundary loop index={index} len={} points={points:?}",
                    wire.len()
                );
            });
        }
        let first_boundary = boundary_loops.first();
        return Err(ShapeOpsError::InvalidOutputShellCondition(Box::new(
            InvalidOutputShellConditionData {
                operation,
                shell_index,
                empty: shell.is_empty(),
                connected: shell.is_connected(),
                condition: shell.shell_condition(),
                boundary_loops: boundary_loops.len(),
                first_boundary_len: first_boundary.map(|wire| wire.len()),
                first_boundary_front: first_boundary
                    .and_then(Wire::front_vertex)
                    .map(Vertex::point),
                first_boundary_back: first_boundary
                    .and_then(Wire::back_vertex)
                    .map(Vertex::point),
                singular_vertices: shell.singular_vertices().len(),
                first_singular: shell.singular_vertices().first().map(Vertex::point),
            },
        )));
    }
    let output_boundaries = if valid_boundaries.len() < boundaries.len() {
        valid_boundaries
    } else {
        boundaries
    };
    Solid::try_new(output_boundaries)
        .map_err(|source| ShapeOpsError::InvalidOutputShell { operation, source })
}

/// AND operation between two solids.
pub fn and<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> ShapeOpsResult<Solid<Point3, C, S>> {
    let debug_components = std::env::var("MT_BOOL_DEBUG_COMPONENTS").is_ok();
    let mut iter0 = solid0.boundaries().iter();
    let mut iter1 = solid1.boundaries().iter();
    // SAFETY: a `Solid` always has at least one boundary shell.
    let shell0 = iter0.next().unwrap();
    // SAFETY: a `Solid` always has at least one boundary shell.
    let shell1 = iter1.next().unwrap();

    let [mut and_shell, _] = process_one_pair_of_shells(shell0, shell1, tol)?;
    for shell in iter0 {
        let [res, _] = process_one_pair_of_shells(&and_shell, shell, tol)?;
        and_shell = res;
    }
    for shell in iter1 {
        let [res, _] = process_one_pair_of_shells(&and_shell, shell, tol)?;
        and_shell = res;
    }

    let boundaries = {
        let comps = and_shell.connected_components();
        if comps.is_empty() {
            vec![and_shell.clone()]
        } else {
            comps
        }
    };
    if debug_components {
        boundaries.iter().enumerate().for_each(|(i, shell)| {
            eprintln!(
                "debug and component[{i}] faces={} condition={:?} boundary={} singular={}",
                shell.len(),
                shell.shell_condition(),
                shell.extract_boundaries().len(),
                shell.singular_vertices().len(),
            );
        });
    }
    try_build_solid("and", boundaries, tol)
}

/// OR operation between two solids.
pub fn or<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> ShapeOpsResult<Solid<Point3, C, S>> {
    let debug_components = std::env::var("MT_BOOL_DEBUG_COMPONENTS").is_ok();
    let mut iter0 = solid0.boundaries().iter();
    let mut iter1 = solid1.boundaries().iter();
    // SAFETY: a `Solid` always has at least one boundary shell.
    let shell0 = iter0.next().unwrap();
    // SAFETY: a `Solid` always has at least one boundary shell.
    let shell1 = iter1.next().unwrap();

    let [_, mut or_shell] = process_one_pair_of_shells(shell0, shell1, tol)?;
    for shell in iter0 {
        let [_, res] = process_one_pair_of_shells(&or_shell, shell, tol)?;
        or_shell = res;
    }
    for shell in iter1 {
        let [_, res] = process_one_pair_of_shells(&or_shell, shell, tol)?;
        or_shell = res;
    }

    let boundaries = {
        let comps = or_shell.connected_components();
        if comps.is_empty() {
            vec![or_shell.clone()]
        } else {
            comps
        }
    };
    if debug_components {
        boundaries.iter().enumerate().for_each(|(i, shell)| {
            eprintln!(
                "debug or component[{i}] faces={} condition={:?} boundary={} singular={}",
                shell.len(),
                shell.shell_condition(),
                shell.extract_boundaries().len(),
                shell.singular_vertices().len(),
            );
        });
    }
    try_build_solid("or", boundaries, tol)
}

/// Difference: the region inside `solid0` but outside `solid1`.
pub fn difference<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> ShapeOpsResult<Solid<Point3, C, S>> {
    let mut neg = solid1.clone();
    neg.not();
    and(solid0, &neg, tol)
}

/// Symmetric difference (XOR): the region inside exactly one of the solids.
pub fn symmetric_difference<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> ShapeOpsResult<Solid<Point3, C, S>> {
    let d0 = difference(solid0, solid1, tol)?;
    let d1 = difference(solid1, solid0, tol)?;
    or(&d0, &d1, tol)
}

#[cfg(test)]
mod tests;
