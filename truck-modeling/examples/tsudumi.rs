mod framework;
use framework::ShapeViewer;
use truck_modeling::*;

fn main() {
    let v0 = builder::vertex(Point3::new(1.0, 1.0, 0.0));
    let v1 = builder::vertex(Point3::new(0.0, -1.0, 1.0));
    let line = builder::line(&v0, &v1);
    let mut shell = builder::rsweep(&line, Point3::origin(), Vector3::unit_y());
    let wires = shell.extract_boundaries();
    shell.push(builder::try_attach_plane(&vec![wires[0].clone()]).unwrap().inverse());
    shell.push(builder::try_attach_plane(&vec![wires[1].clone()]).unwrap().inverse());
    let solid = Solid::new(vec![shell]);
    ShapeViewer::run(solid);
}