use crate::*;
use truck_meshalgo::tessellation::*;
use truck_stepio::r#in::step_geometry::*;
use truck_topology::compress::*;

/// step parse table
#[derive(Clone, Debug, Deref, DerefMut, From, Into)]
#[wasm_bindgen]
pub struct Table(truck_stepio::r#in::Table);

#[derive(Clone, Debug)]
enum SubShapeFromStep {
    Shell(CompressedShell<Point3, Curve3D, Surface>),
    #[allow(dead_code)]
    Solid(CompressedSolid<Point3, Curve3D, Surface>),
}

/// Shell and Solid parsed from step
#[derive(Clone, Debug, From, Into)]
#[wasm_bindgen]
pub struct ShapeFromStep(SubShapeFromStep);

#[wasm_bindgen]
impl ShapeFromStep {
    /// meshing shape from step
    pub fn to_polygon(&self, tol: f64) -> crate::PolygonMesh {
        use SubShapeFromStep::*;
        match &self.0 {
            Shell(x) => x.robust_triangulation(tol).to_polygon().into(),
            Solid(x) => x.robust_triangulation(tol).to_polygon().into(),
        }
    }
}

#[wasm_bindgen]
impl Table {
    /// read step file
    pub fn from_step(step_str: &str) -> Option<Table> {
        Some(Table(truck_stepio::r#in::Table::from_step(step_str)?))
    }
    /// get shell indices
    pub fn shell_indices(&self) -> Vec<u64> { self.0.shell.keys().copied().collect() }
    /// get shape from indices
    pub fn get_shape(&self, idx: u64) -> Option<ShapeFromStep> {
        let stepshell = self.shell.get(&idx)?;
        let shell = self
            .to_compressed_shell(stepshell)
            .map_err(|e| gloo::console::error!(format!("{e}")))
            .ok()?;
        Some(SubShapeFromStep::Shell(shell).into())
    }
}
