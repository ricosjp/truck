#[macro_export]
macro_rules! toporep {
    ($mod: tt, $toporep_mod: ident) => {
        mod $toporep_mod {
            use super::$mod;
            use std::convert::TryFrom;
            use std::result::Result;
            use truck_topology::*;
            use $crate::alias::*;
            $crate::sub_topo_rep!($mod);
        }
    };
}

#[macro_export]
macro_rules! sub_topo_rep {
    ($mod: tt) => {
        impl Empty for $mod::TopologicalRepresentationItem {
            fn empty() -> Self {
                Self::new(Empty::empty())
            }
        }

        impl<'a, P> TryFrom<&'a $mod::VertexPoint> for Vertex<P>
        where
            P: TryFrom<&'a $mod::PointAny, Error = ExpressParseError>,
        {
            type Error = ExpressParseError;
            fn try_from(v: &'a $mod::VertexPoint) -> Result<Vertex<P>, ExpressParseError> {
                Ok(Vertex::new(P::try_from(&v.vertex_geometry)?))
            }
        }

        impl<'a, P> TryFrom<&'a $mod::VertexAny> for Vertex<P>
        where
            P: TryFrom<&'a $mod::PointAny, Error = ExpressParseError>,
        {
            type Error = ExpressParseError;
            fn try_from(v: &'a $mod::VertexAny) -> Result<Vertex<P>, ExpressParseError> {
                use $mod::VertexAny;
                match v {
                    VertexAny::Vertex(_) => Err("not enough data!".to_string()),
                    VertexAny::VertexPoint(x) => Vertex::try_from(&**x),
                }
            }
        }
    };
}
