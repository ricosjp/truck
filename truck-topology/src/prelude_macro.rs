/// This macro is a replacement for the prelude module. It loads the main public structures in truck-topology,
/// and then redefines the topological elements with the specific geometric elements.
///
/// # Examples
/// ```
/// struct Point;
/// struct Curve;
/// struct Surface;
/// truck_topology::prelude!(Point, Curve, Surface);
///
/// use std::any::type_name;
/// assert_eq!(
///     type_name::<Face>(),
///     type_name::<truck_topology::Face<Point, Curve, Surface>>(),
/// );
/// ```
///
/// By adding the `pub` option to the end, one can also add `pub` to the generated code.
/// ```
/// mod reexport_topology {
///     truck_topology::prelude!((), (), (), pub);
/// }
///
/// use std::any::type_name;
/// assert_eq!(
///     type_name::<reexport_topology::Shell>(),
///     type_name::<truck_topology::Shell<(), (), ()>>(),
/// );
/// ```
#[macro_export]
macro_rules! prelude {
    ($point: ty, $curve: ty, $surface: ty $(, $pub: tt)?) => {
        #[allow(unused)]
        $($pub)? use $crate::{
            compress::CompressedEdgeIndex,
            shell::ShellCondition,
            VertexDisplayFormat,
            EdgeDisplayFormat,
            WireDisplayFormat,
            FaceDisplayFormat,
            ShellDisplayFormat,
            SolidDisplayFormat,
        };

        /// Vertex, the minimum topological unit.
        #[allow(unused)]
        $($pub)? type Vertex = $crate::Vertex<$point>;
        /// Edge, which consists two vertices.
        #[allow(unused)]
        $($pub)? type Edge = $crate::Edge<$point, $curve>;
        /// Wire, a path or cycle which consists some edges.
        #[allow(unused)]
        $($pub)? type Wire = $crate::Wire<$point, $curve>;
        /// Face, attached to a simple and closed wire.
        #[allow(unused)]
        $($pub)? type Face = $crate::Face<$point, $curve, $surface>;
        /// Shell, a connected compounded faces.
        #[allow(unused)]
        $($pub)? type Shell = $crate::Shell<$point, $curve, $surface>;
        /// Solid, attached to a closed shells.
        #[allow(unused)]
        $($pub)? type Solid = $crate::Solid<$point, $curve, $surface>;

        /// The id of vertex. `Copy` trait is implemented.
        #[allow(unused)]
        $($pub)? type VertexID = $crate::VertexID<$point>;
        /// The id that does not depend on the direction of the edge.
        #[allow(unused)]
        $($pub)? type EdgeID = $crate::EdgeID<$curve>;
        /// The id that does not depend on the direction of the face.
        #[allow(unused)]
        $($pub)? type FaceID = $crate::FaceID<$surface>;

        /// Serialized compressed edge
        #[allow(unused)]
        $($pub)? type CompressedEdge = $crate::compress::CompressedEdge<$curve>;
        /// Serialized compressed face
        #[allow(unused)]
        $($pub)? type CompressedFace = $crate::compress::CompressedFace<$surface>;
        /// Serialized compressed shell
        #[allow(unused)]
        $($pub)? type CompressedShell = $crate::compress::CompressedShell<$point, $curve, $surface>;
        /// Serialized compressed solid
        #[allow(unused)]
        $($pub)? type CompressedSolid = $crate::compress::CompressedSolid<$point, $curve, $surface>;
    };
}

#[doc(hidden)]
pub mod empty_geometries {
    #![allow(missing_debug_implementations)]
    pub struct Point;
    pub struct Curve;
    pub struct Surface;
}
pub use empty_geometries::*;
prelude!(Point, Curve, Surface, pub);
