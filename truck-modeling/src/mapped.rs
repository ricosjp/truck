use crate::topo_traits::*;
use truck_topology::*;

impl<P, T> Mapped<T> for Vertex<P>
where T: GeometricMapping<P> + Copy
{
    #[inline(always)]
    fn mapped(&self, trans: T) -> Self { self.mapped(trans.mapping()) }
}

impl<P, C, T> Mapped<T> for Edge<P, C>
where T: GeometricMapping<P> + GeometricMapping<C> + Copy
{
    #[inline(always)]
    fn mapped(&self, trans: T) -> Self {
        let point_mapping = GeometricMapping::<P>::mapping(trans);
        let curve_mapping = GeometricMapping::<C>::mapping(trans);
        self.mapped(point_mapping, curve_mapping)
    }
}

impl<P, C, T> Mapped<T> for Wire<P, C>
where T: GeometricMapping<P> + GeometricMapping<C> + Copy
{
    #[inline(always)]
    fn mapped(&self, trans: T) -> Self {
        let point_mapping = GeometricMapping::<P>::mapping(trans);
        let curve_mapping = GeometricMapping::<C>::mapping(trans);
        self.mapped(point_mapping, curve_mapping)
    }
}

impl<P, C, S, T> Mapped<T> for Face<P, C, S>
where T: GeometricMapping<P> + GeometricMapping<C> + GeometricMapping<S> + Copy
{
    #[inline(always)]
    fn mapped(&self, trans: T) -> Self {
        let point_mapping = GeometricMapping::<P>::mapping(trans);
        let curve_mapping = GeometricMapping::<C>::mapping(trans);
        let surface_mapping = GeometricMapping::<S>::mapping(trans);
        self.mapped(point_mapping, curve_mapping, surface_mapping)
    }
}

impl<P, C, S, T> Mapped<T> for Shell<P, C, S>
where T: GeometricMapping<P> + GeometricMapping<C> + GeometricMapping<S> + Copy
{
    #[inline(always)]
    fn mapped(&self, trans: T) -> Self {
        let point_mapping = GeometricMapping::<P>::mapping(trans);
        let curve_mapping = GeometricMapping::<C>::mapping(trans);
        let surface_mapping = GeometricMapping::<S>::mapping(trans);
        self.mapped(point_mapping, curve_mapping, surface_mapping)
    }
}

impl<P, C, S, T> Mapped<T> for Solid<P, C, S>
where T: GeometricMapping<P> + GeometricMapping<C> + GeometricMapping<S> + Copy
{
    #[inline(always)]
    fn mapped(&self, trans: T) -> Self {
        let point_mapping = GeometricMapping::<P>::mapping(trans);
        let curve_mapping = GeometricMapping::<C>::mapping(trans);
        let surface_mapping = GeometricMapping::<S>::mapping(trans);
        self.mapped(point_mapping, curve_mapping, surface_mapping)
    }
}
