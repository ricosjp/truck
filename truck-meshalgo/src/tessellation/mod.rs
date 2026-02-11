use crate::*;
use spade::{iterators::*, *};
use truck_topology::{compress::*, *};

/// Tessellation output primitive preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QuadMode {
    /// Keep triangle output.
    #[default]
    Triangles,
    /// Convert triangles to quads when possible.
    PreferQuads,
    /// Force all triangles into quads.
    AllQuads,
    /// Prefer UV-grid quads and keep non-quad elements near trims.
    IsoQuads,
}

/// Quad generation options.
#[derive(Clone, Copy, Debug)]
pub struct QuadOptions {
    /// Primitive generation mode.
    pub mode: QuadMode,
    /// Coplanarity tolerance used for triangle pairing.
    pub plane_tolerance: f64,
    /// Shape score tolerance used for triangle pairing.
    pub score_tolerance: f64,
    /// Maximum normal blend angle in radians.
    pub normal_blend_angle: f64,
    /// Minimum quad area accepted by the `AllQuads` fallback.
    pub minimum_area: f64,
    /// Maximum corner angle accepted by the `AllQuads` fallback.
    pub maximum_corner_angle: f64,
}

impl Default for QuadOptions {
    fn default() -> Self {
        Self {
            mode: QuadMode::Triangles,
            plane_tolerance: 0.01,
            score_tolerance: 1.0,
            normal_blend_angle: std::f64::consts::PI / 4.0,
            minimum_area: TOLERANCE * TOLERANCE,
            maximum_corner_angle: 175.0 * std::f64::consts::PI / 180.0,
        }
    }
}

/// Options for tessellation.
#[derive(Clone, Copy, Debug)]
pub struct TessellationOptions {
    /// Geometric tolerance for curve and surface approximation.
    pub tolerance: f64,
    /// Maximum number of Newton iterations per parameter search.
    pub search_trials: usize,
    /// Quad generation policy.
    pub quad: QuadOptions,
}

impl Default for TessellationOptions {
    fn default() -> Self {
        Self {
            tolerance: 0.01,
            search_trials: 100,
            quad: QuadOptions::default(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod parallelizable {
    /// Parallelizable by `rayon`.
    pub trait Parallelizable: Send + Sync {}
    impl<T: Send + Sync> Parallelizable for T {}
}

#[cfg(target_arch = "wasm32")]
mod parallelizable {
    /// No parallelization in the case of wasm.
    pub trait Parallelizable {}
    impl<T> Parallelizable for T {}
}

pub use parallelizable::*;

/// Gathered the traits used in tessellation.
pub trait PolylineableCurve:
    ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3> + Parallelizable {
}
impl<
        C: ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3> + Parallelizable,
    > PolylineableCurve for C
{
}

/// It can be meshed, but not necessarily trimmed.
pub trait PreMeshableSurface: ParametricSurface3D + ParameterDivision2D + Parallelizable {}
impl<S: ParametricSurface3D + ParameterDivision2D + Parallelizable> PreMeshableSurface for S {}

/// The generated mesh can be trimmed only if the boundary curves ride strictly on a surface.
pub trait MeshableSurface: PreMeshableSurface + SearchParameter<D2, Point = Point3> {}
impl<S: PreMeshableSurface + SearchParameter<D2, Point = Point3>> MeshableSurface for S {}

/// The generated mesh can be trimmed if the boundary curves does not ride strictly on a surface.
pub trait RobustMeshableSurface:
    MeshableSurface + SearchNearestParameter<D2, Point = Point3> {
}
impl<S: MeshableSurface + SearchNearestParameter<D2, Point = Point3>> RobustMeshableSurface for S {}

type PolylineCurve = truck_polymesh::PolylineCurve<Point3>;

/// Trait for converting tessellated shape into polygon.
pub trait MeshedShape {
    /// Converts tessellated shape into polygon.
    fn to_polygon(&self) -> PolygonMesh;
}

impl MeshedShape for Shell<Point3, PolylineCurve, PolygonMesh> {
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.face_iter().for_each(|face| {
            polygon.merge(face.oriented_surface());
        });
        polygon
    }
}

impl MeshedShape for Shell<Point3, PolylineCurve, Option<PolygonMesh>> {
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.face_iter().for_each(|face| {
            if let Some(mut poly) = face.surface() {
                if !face.orientation() {
                    poly.invert();
                }
                polygon.merge(poly);
            }
        });
        polygon
    }
}

impl<P, C, S> MeshedShape for Solid<P, C, S>
where Shell<P, C, S>: MeshedShape
{
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.boundaries().iter().for_each(|shell| {
            polygon.merge(shell.to_polygon());
        });
        polygon
    }
}

impl MeshedShape for CompressedShell<Point3, PolylineCurve, PolygonMesh> {
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.faces.iter().for_each(|face| match face.orientation {
            true => polygon.merge(face.surface.clone()),
            false => polygon.merge(face.surface.inverse()),
        });
        polygon
    }
}

impl MeshedShape for CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>> {
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.faces.iter().for_each(|face| {
            if let Some(surface) = &face.surface {
                match face.orientation {
                    true => polygon.merge(surface.clone()),
                    false => polygon.merge(surface.inverse()),
                }
            }
        });
        polygon
    }
}

impl<P, C, S> MeshedShape for CompressedSolid<P, C, S>
where CompressedShell<P, C, S>: MeshedShape
{
    fn to_polygon(&self) -> PolygonMesh {
        let mut polygon = PolygonMesh::default();
        self.boundaries.iter().for_each(|shell| {
            polygon.merge(shell.to_polygon());
        });
        polygon
    }
}

/// Trait for tessellating `Shell` and `Solid`.
pub trait MeshableShape {
    /// Shape whose edges are made polylines and faces polygon surface.
    type MeshedShape: MeshedShape;
    /// Tessellates shapes. The division of curves and surfaces are by `ParameterDivision1D` and `ParameterDivision2D`,
    /// and the constrained Delauney triangulation is based on the crate [`spade`](https://crates.io/crates/spade).
    ///
    /// # Panics
    ///
    /// `tolerance` must be greater than or equal to `TOLERANCE`.
    ///
    /// # Remarks
    ///
    /// - The tessellated mesh is not necessarily closed even if `self` is `Solid`.
    ///   If you want to get closed mesh, use [`OptimizingFilter::put_together_same_attrs`].
    /// - This method requires that the curve ride strictly on a surface. If not, try [`RobustMeshableShape`].
    ///
    /// [`OptimizingFilter::put_together_same_attrs`]: crate::filters::OptimizingFilter::put_together_same_attrs
    ///
    /// # Examples
    /// ```
    /// use truck_meshalgo::prelude::*;
    /// use truck_modeling::*;
    /// use truck_topology::shell::ShellCondition;
    ///
    /// // modeling a unit cube
    /// let v = builder::vertex(Point3::origin());
    /// let e = builder::tsweep(&v, Vector3::unit_x());
    /// let f = builder::tsweep(&e, Vector3::unit_y());
    /// let cube: Solid = builder::tsweep(&f, Vector3::unit_z());
    ///
    /// // cube is Solid, however, the tessellated mesh is not closed.
    /// let mut mesh = cube.triangulation(0.01).to_polygon();
    /// assert_ne!(mesh.shell_condition(), ShellCondition::Closed);
    ///
    /// // use optimization filters!
    /// mesh.put_together_same_attrs(TOLERANCE);
    /// assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
    /// ```
    fn triangulation(&self, tolerance: f64) -> Self::MeshedShape;
}

/// Trait for tessellating `Shell` and `Solid` in `truck-modeling`.
pub trait RobustMeshableShape {
    /// Shape whose edges are made polylines and faces polygon surface.
    type MeshedShape: MeshedShape;
    /// Tessellates shapes. The division of curves and surfaces are by `ParameterDivision1D` and `ParameterDivision2D`,
    /// and the constrained Delauney triangulation is based on the crate [`spade`](https://crates.io/crates/spade).
    ///
    /// # Panics
    ///
    /// `tolerance` must be greater than or equal to `TOLERANCE`.
    ///
    /// # Remarks
    ///
    /// Since polyline vertices are projected onto the surface, processing speed is often slower than with [`MeshableShape::triangulation`].
    /// It also does not close the mesh of a solid even if one uses [`OptimizingFilter::put_together_same_attrs`].
    ///
    /// [`OptimizingFilter::put_together_same_attrs`]: crate::filters::OptimizingFilter::put_together_same_attrs
    ///
    /// # Examples
    /// ```
    /// use truck_meshalgo::prelude::*;
    /// use truck_modeling::*;
    /// use truck_topology::shell::ShellCondition;
    ///
    /// // manual modeling an open half cylinder
    ///
    /// // points
    /// let p = [
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(-1.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 1.0),
    ///     Point3::new(-1.0, 0.0, 1.0),
    /// ];
    /// // vertices
    /// let v = Vertex::news(&p);
    /// // Curves that do not ride on a cylinder
    /// let bsp0: Curve = BSplineCurve::new(
    ///     KnotVec::bezier_knot(3),
    ///     vec![
    ///         p[0],
    ///         Point3::new(1.0, 4.0 / 3.0, 0.0),
    ///         Point3::new(-1.0, 4.0 / 3.0, 0.0),
    ///         p[1],
    ///     ],
    /// )
    /// .into();
    /// let bsp1: Curve = BSplineCurve::new(
    ///     KnotVec::bezier_knot(3),
    ///     vec![
    ///         p[3],
    ///         Point3::new(-1.0, 4.0 / 3.0, 1.0),
    ///         Point3::new(1.0, 4.0 / 3.0, 1.0),
    ///         p[2],
    ///     ],
    /// )
    /// .into();
    /// // wire
    /// let w: Wire = vec![
    ///     builder::line(&v[2], &v[0]),
    ///     Edge::new(&v[0], &v[1], bsp0),
    ///     builder::line(&v[1], &v[3]),
    ///     Edge::new(&v[3], &v[2], bsp1),
    /// ]
    /// .into();
    /// // revoluted curve
    /// let surface_raw = RevolutedCurve::by_revolution(
    ///     Curve::Line(Line(p[2], p[0])),
    ///     Point3::origin(),
    ///     Vector3::unit_z(),
    /// );
    /// let surface: Surface = Processor::new(surface_raw).into();
    /// // shell
    /// let shell: Shell = vec![Face::new(vec![w], surface)].into();
    ///
    /// // Simple triangulation fails since some edges do not ride on a cylinder
    /// let poly_shell = shell.triangulation(0.01);
    /// assert!(poly_shell[0].surface().is_none());
    ///
    /// // Robust triangulation!
    /// let poly_shell = shell.robust_triangulation(0.01);
    /// let poly = poly_shell[0].surface().unwrap();
    /// assert!(!poly.positions().is_empty());
    /// ```
    fn robust_triangulation(&self, tolerance: f64) -> Self::MeshedShape;
}

/// Tessellates a [`Shell`] with a [`TessellationOptions`].
pub fn triangulation_with<C: PolylineableCurve, S: MeshableSurface>(
    shell: &Shell<Point3, C, S>,
    options: TessellationOptions,
) -> Shell<Point3, PolylineCurve, Option<PolygonMesh>> {
    nonpositive_tolerance!(options.tolerance);
    let sp = triangulation::search_parameter_sp::<S>(options.search_trials);
    #[cfg(not(target_arch = "wasm32"))]
    let res = triangulation::shell_tessellation(shell, options.tolerance, sp, options.quad);
    #[cfg(target_arch = "wasm32")]
    let res =
        triangulation::shell_tessellation_single_thread(shell, options.tolerance, sp, options.quad);
    res
}

/// Tessellates a [`Shell`] with robust parameter search and a [`TessellationOptions`].
pub fn robust_triangulation_with<C: PolylineableCurve, S: RobustMeshableSurface>(
    shell: &Shell<Point3, C, S>,
    options: TessellationOptions,
) -> Shell<Point3, PolylineCurve, Option<PolygonMesh>> {
    nonpositive_tolerance!(options.tolerance);
    let sp = triangulation::search_nearest_parameter_sp::<S>(options.search_trials);
    #[cfg(not(target_arch = "wasm32"))]
    let res = triangulation::shell_tessellation(shell, options.tolerance, sp, options.quad);
    #[cfg(target_arch = "wasm32")]
    let res =
        triangulation::shell_tessellation_single_thread(shell, options.tolerance, sp, options.quad);
    res
}

/// Tessellates a [`CompressedShell`] with a [`TessellationOptions`].
pub fn cshell_triangulation_with<C: PolylineableCurve, S: MeshableSurface>(
    shell: &CompressedShell<Point3, C, S>,
    options: TessellationOptions,
) -> CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>> {
    nonpositive_tolerance!(options.tolerance);
    let sp = triangulation::search_parameter_sp::<S>(options.search_trials);
    triangulation::cshell_tessellation(shell, options.tolerance, sp, options.quad)
}

/// Tessellates a [`CompressedShell`] with robust parameter search and a [`TessellationOptions`].
pub fn robust_cshell_triangulation_with<C: PolylineableCurve, S: RobustMeshableSurface>(
    shell: &CompressedShell<Point3, C, S>,
    options: TessellationOptions,
) -> CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>> {
    nonpositive_tolerance!(options.tolerance);
    let sp = triangulation::search_nearest_parameter_sp::<S>(options.search_trials);
    triangulation::cshell_tessellation(shell, options.tolerance, sp, options.quad)
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for Shell<Point3, C, S> {
    type MeshedShape = Shell<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn triangulation(&self, tolerance: f64) -> Self::MeshedShape {
        triangulation_with(
            self,
            TessellationOptions {
                tolerance,
                ..Default::default()
            },
        )
    }
}

impl<C: PolylineableCurve, S: RobustMeshableSurface> RobustMeshableShape for Shell<Point3, C, S> {
    type MeshedShape = Shell<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn robust_triangulation(&self, tolerance: f64) -> Self::MeshedShape {
        robust_triangulation_with(
            self,
            TessellationOptions {
                tolerance,
                ..Default::default()
            },
        )
    }
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for Solid<Point3, C, S> {
    type MeshedShape = Solid<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn triangulation(&self, tolerance: f64) -> Self::MeshedShape {
        let boundaries = self
            .boundaries()
            .iter()
            .map(|shell| shell.triangulation(tolerance))
            .collect::<Vec<_>>();
        Solid::new(boundaries)
    }
}

impl<C: PolylineableCurve, S: RobustMeshableSurface> RobustMeshableShape for Solid<Point3, C, S> {
    type MeshedShape = Solid<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn robust_triangulation(&self, tolerance: f64) -> Self::MeshedShape {
        let boundaries = self
            .boundaries()
            .iter()
            .map(|shell| shell.robust_triangulation(tolerance))
            .collect::<Vec<_>>();
        Solid::new(boundaries)
    }
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for CompressedShell<Point3, C, S> {
    type MeshedShape = CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn triangulation(&self, tolerance: f64) -> Self::MeshedShape {
        cshell_triangulation_with(
            self,
            TessellationOptions {
                tolerance,
                ..Default::default()
            },
        )
    }
}

impl<C: PolylineableCurve, S: RobustMeshableSurface> RobustMeshableShape
    for CompressedShell<Point3, C, S>
{
    type MeshedShape = CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn robust_triangulation(&self, tolerance: f64) -> Self::MeshedShape {
        robust_cshell_triangulation_with(
            self,
            TessellationOptions {
                tolerance,
                ..Default::default()
            },
        )
    }
}

impl<C: PolylineableCurve, S: MeshableSurface> MeshableShape for CompressedSolid<Point3, C, S> {
    type MeshedShape = CompressedSolid<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn triangulation(&self, tolerance: f64) -> Self::MeshedShape {
        let boundaries = self
            .boundaries
            .iter()
            .map(|shell| shell.triangulation(tolerance))
            .collect::<Vec<_>>();
        CompressedSolid { boundaries }
    }
}

impl<C: PolylineableCurve, S: RobustMeshableSurface> RobustMeshableShape
    for CompressedSolid<Point3, C, S>
{
    type MeshedShape = CompressedSolid<Point3, PolylineCurve, Option<PolygonMesh>>;
    fn robust_triangulation(&self, tolerance: f64) -> Self::MeshedShape {
        let boundaries = self
            .boundaries
            .iter()
            .map(|shell| shell.robust_triangulation(tolerance))
            .collect::<Vec<_>>();
        CompressedSolid { boundaries }
    }
}

mod triangulation;
