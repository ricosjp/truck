//! Benchmark tessellation of all shape JSON files.
//!
//! ```bash
//! cargo run -p truck-meshalgo --example bench-tessellation
//! ```

use std::time::Instant;
use truck_meshalgo::{analyzers::*, filters::*, tessellation::*};
use truck_modeling::{geometry::*, Point3};
use truck_polymesh::TOLERANCE;
use truck_topology::compress::*;

type CShell = CompressedShell<Point3, Curve, Surface>;
type CSolid = CompressedSolid<Point3, Curve, Surface>;

fn tessellate_shape(name: &str, json: &str) {
    let instant = Instant::now();
    let mut poly = {
        if let Ok(solid) = serde_json::from_str::<CSolid>(json) {
            solid.triangulation(0.005).to_polygon()
        } else if let Ok(shell) = serde_json::from_str::<CShell>(json) {
            shell.triangulation(0.005).to_polygon()
        } else {
            eprintln!("  {name}: failed to parse");
            return;
        }
    };
    let elapsed = instant.elapsed();
    poly.put_together_same_attrs(TOLERANCE * 10.0)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    let tri_count = poly.tri_faces().len();
    let condition = poly.shell_condition();
    println!("  {name}: {elapsed:.1?}, {tri_count} triangles, condition={condition:?}",);
}

fn main() {
    let shapes_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/shape");
    println!("=== Shape tessellation benchmarks ===");
    let mut entries: Vec<_> = std::fs::read_dir(shapes_dir)
        .expect("cannot read resources/shape")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let total = Instant::now();
    for entry in &entries {
        let path = entry.path();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let json = std::fs::read_to_string(&path).unwrap();
        tessellate_shape(&name, &json);
    }
    println!("  total: {:.1?}", total.elapsed());
}
