use proptest::prelude::*;
use truck_geometry::prelude::*;
use truck_geotrait::polynomial::*;

fn exec_pcurve_derivation(
    curve_coef: &[[f64; 2]],
    surface_ucoef: &[[f64; 3]],
    surface_vcoef: &[[f64; 3]],
    n: usize,
    t: f64,
) -> std::result::Result<(), TestCaseError> {
    let curve_coef = curve_coef.iter().map(|&p| Vector2::from(p)).collect();
    let curve = PolynomialCurve::<Point2>(curve_coef);

    let surface_ucoef = surface_ucoef.iter().map(|&p| Vector3::from(p)).collect();
    let surface_vcoef = surface_vcoef.iter().map(|&p| Vector3::from(p)).collect();
    let ucurve = PolynomialCurve::<Point3>(surface_ucoef);
    let vcurve = PolynomialCurve::<Point3>(surface_vcoef);
    let surface = PolynomialSurface::by_tensor(ucurve, vcurve);

    let pcurve0 = PCurve::new(curve, surface);
    let pcurve1 = pcurve0.surface().composite(pcurve0.curve());

    prop_assert_near!(pcurve0.der_n(n, t), pcurve1.der_n(n, t));

    let ders0 = (0..=n).map(|i| pcurve0.der_n(i, t)).collect::<Vec<_>>();

    let mut ders1 = vec![Vector3::zero(); n + 1];
    pcurve0.ders(t, &mut ders1);

    let ders2 = pcurve0.ders_vec(n, t);

    prop_assert_eq!(ders0.len(), ders1.len());
    prop_assert_eq!(ders1.len(), ders2.len());

    let mut iter = ders0.into_iter().zip(ders1).zip(ders2);
    iter.try_for_each(|((v0, v1), v2)| {
        prop_assert_near!(v0, v1);
        prop_assert_near!(v1, v2);
        Ok(())
    })?;
    Ok(())
}

proptest! {
    #[test]
    fn pcurve_derivation(
        curve_coef in prop::array::uniform6(prop::array::uniform2(-1f64..=1.0)),
        curve_degree in 1usize..6,
        surface_ucoef in prop::array::uniform6(prop::array::uniform3(-1f64..=1.0)),
        surface_vcoef in prop::array::uniform6(prop::array::uniform3(-1f64..=1.0)),
        surface_udegree in 1usize..6,
        surface_vdegree in 1usize..6,
        n in 0usize..=5,
        t in -1f64..=1.0,
    ) {
        exec_pcurve_derivation(
            &curve_coef[..=curve_degree],
            &surface_ucoef[..=surface_udegree],
            &surface_vcoef[..=surface_vdegree],
            n,
            t,
        )?;
    }
}
