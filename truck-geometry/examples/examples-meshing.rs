// sample geometric data
const SAMPLE_TGB: &str = "tests/data/examples.tgb";

fn main() {
    let samplefile = std::fs::File::open(SAMPLE_TGB).unwrap();
    let geomdata = truck_io::tgb::read(samplefile).unwrap();

    for (i, surface) in geomdata.surfaces.into_iter().enumerate() {
        let file = std::fs::File::create(format!("sample_{}.obj", i)).unwrap();
        let surface = truck_geometry::NURBSSurface::new(surface);
        let mesh = truck_polymesh::StructuredMesh::from_surface(&surface, 0.01);
        truck_io::obj::write(&mesh.destruct(), file).unwrap();
    }
}
