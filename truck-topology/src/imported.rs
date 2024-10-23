/// This macro is a replacement for the prelude module. It loads the main public structures in truck-topology,
/// and redefines the topological elements in a way that relates them to geometric elements.
/// For information on the actual structures that are loaded (or aliased), please refer to [`imported`].
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
/// ```compile_fail
/// mod first {
///     pub mod second {
///         truck_topology::prelude!((), (), (), pub(super));
///     }
///     fn inside_public_range() {
///         let _ = second::Vertex::new(());
///     }
/// }
/// // compile fail
/// fn outside_public_range() {
///     let _ = first::second::Vertex::new(());
/// }
/// ```
/// 
/// [`imported`]: crate::imported
#[macro_export]
macro_rules! prelude {
    ($point: ty, $curve: ty, $surface: ty $(, $pub: tt $($super: tt)?)?) => {
        #[allow(unused)]
        $($pub$($super)?)? use $crate::{
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
        $($pub$($super)?)? type Vertex = $crate::Vertex<$point>;
        /// Edge, which consists two vertices.
        #[allow(unused)]
        $($pub$($super)?)? type Edge = $crate::Edge<$point, $curve>;
        /// Wire, a path or cycle which consists some edges.
        #[allow(unused)]
        $($pub$($super)?)? type Wire = $crate::Wire<$point, $curve>;
        /// Face, attached to a simple and closed wire.
        #[allow(unused)]
        $($pub$($super)?)? type Face = $crate::Face<$point, $curve, $surface>;
        /// Shell, a connected compounded faces.
        #[allow(unused)]
        $($pub$($super)?)? type Shell = $crate::Shell<$point, $curve, $surface>;
        /// Solid, attached to a closed shells.
        #[allow(unused)]
        $($pub$($super)?)? type Solid = $crate::Solid<$point, $curve, $surface>;

        /// The id of vertex. `Copy` trait is implemented.
        #[allow(unused)]
        $($pub$($super)?)? type VertexID = $crate::VertexID<$point>;
        /// The id that does not depend on the direction of the edge.
        #[allow(unused)]
        $($pub$($super)?)? type EdgeID = $crate::EdgeID<$curve>;
        /// The id that does not depend on the direction of the face.
        #[allow(unused)]
        $($pub$($super)?)? type FaceID = $crate::FaceID<$surface>;

        /// Serialized compressed edge
        #[allow(unused)]
        $($pub$($super)?)? type CompressedEdge = $crate::compress::CompressedEdge<$curve>;
        /// Serialized compressed face
        #[allow(unused)]
        $($pub$($super)?)? type CompressedFace = $crate::compress::CompressedFace<$surface>;
        /// Serialized compressed shell
        #[allow(unused)]
        $($pub$($super)?)? type CompressedShell = $crate::compress::CompressedShell<$point, $curve, $surface>;
        /// Serialized compressed solid
        #[allow(unused)]
        $($pub$($super)?)? type CompressedSolid = $crate::compress::CompressedSolid<$point, $curve, $surface>;
    };
}

#[doc(hidden)]
pub mod empty_geometries {
    #![allow(missing_debug_implementations)]
    pub struct Point;
    pub struct Curve;
    pub struct Surface;
}
#[doc(hidden)]
pub use empty_geometries::*;
prelude!(Point, Curve, Surface, pub);
