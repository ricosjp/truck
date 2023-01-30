use cgmath::*;
use num_traits::FromPrimitive;
use truck_base::hash::*;

const VEC2UNIT: Vector2<f64> = Vector2::new(1.0, 1.0);
const VEC3UNIT: Vector3<f64> = Vector3::new(1.0, 1.0, 1.0);
const VEC4UNIT: Vector4<f64> = Vector4::new(1.0, 1.0, 1.0, 1.0);

#[test]
fn hash11_test() {
    fn exec_test<S: BaseFloat + FromPrimitive>() {
        const N: usize = 10000;
        let mean = (0..N).fold(S::zero(), |sum, i| {
            sum + HashGen::hash1(S::from_usize(i).unwrap())
        }) / S::from_usize(N).unwrap();
        let var = (0..N).fold(S::zero(), |sum, i| {
            let x = HashGen::hash1(S::from_usize(i).unwrap()) - S::from_f64(0.5).unwrap();
            sum + x * x
        }) / S::from_usize(N).unwrap();
        assert!(
            S::abs(mean - S::from_f64(0.5).unwrap()) < S::from_f64(0.05 * 0.5).unwrap(),
            "mean: {mean:?}"
        );
        assert!(
            S::abs(var - S::from_f64(1.0 / 12.0).unwrap()) < S::from_f64(0.05 / 12.0).unwrap(),
            "var: {var:?}"
        );
    }

    exec_test::<f32>();
    exec_test::<f64>();
}

macro_rules! define_hashx1_test {
    ($func: ident, $vec: ty, $unit: expr, $hash: ident) => {
        #[test]
        fn $func() {
            fn exec_test<S: BaseFloat + FromPrimitive>() {
                const N: usize = 10000;
                let unit = $unit.cast().unwrap();
                let mean = (0..N).fold(<$vec>::zero(), |sum, i| {
                    sum + <$vec>::from(HashGen::$hash(S::from_usize(i).unwrap()))
                }) / S::from_usize(N).unwrap();
                let var = (0..N).fold(<$vec>::zero(), |sum, i| {
                    let x = <$vec>::from(HashGen::$hash(S::from_usize(i).unwrap()))
                        - (unit * 0.5).cast().unwrap();
                    sum + x.mul_element_wise(x)
                }) / S::from_usize(N).unwrap();
                assert!(
                    (mean - (unit * 0.5).cast().unwrap()).magnitude()
                        < S::from_f64(0.05 * 0.5).unwrap(),
                    "mean: {:?}",
                    mean,
                );
                assert!(
                    (var - (unit / 12.0).cast().unwrap()).magnitude()
                        < S::from_f64(0.05 / 12.0).unwrap(),
                    "var: {:?}",
                    var,
                );
            }
            exec_test::<f32>();
            exec_test::<f64>();
        }
    };
}

define_hashx1_test!(hash21_test, Vector2<S>, VEC2UNIT, hash2);
define_hashx1_test!(hash31_test, Vector3<S>, VEC3UNIT, hash3);
define_hashx1_test!(hash41_test, Vector4<S>, VEC4UNIT, hash4);

#[test]
fn hash12_test() {
    fn exec_test<S: BaseFloat + FromPrimitive>() {
        const N: usize = 100;
        let mean = (0..(N * N)).fold(S::zero(), |sum, i| {
            let v = [S::from_usize(i / N).unwrap(), S::from_usize(i % N).unwrap()];
            sum + HashGen::hash1(v)
        }) / S::from_usize(N * N).unwrap();
        let var = (0..(N * N)).fold(S::zero(), |sum, i| {
            let v = [S::from_usize(i / N).unwrap(), S::from_usize(i % N).unwrap()];
            let x = HashGen::hash1(v) - S::from_f64(0.5).unwrap();
            sum + x * x
        }) / S::from_usize(N * N).unwrap();
        assert!(
            S::abs(mean - S::from_f64(0.5).unwrap()) < S::from_f64(0.05 * 0.5).unwrap(),
            "mean: {mean:?}"
        );
        assert!(
            S::abs(var - S::from_f64(1.0 / 12.0).unwrap()) < S::from_f64(0.05 / 12.0).unwrap(),
            "var: {var:?}"
        );
    }

    exec_test::<f32>();
    exec_test::<f64>();
}

macro_rules! define_hashx2_test {
    ($func: ident, $vec: ty, $unit: expr, $hash: ident) => {
        #[test]
        fn $func() {
            fn exec_test<S: BaseFloat + FromPrimitive>() {
                const N: usize = 100;
                let unit = $unit.cast().unwrap();
                let mean = (0..(N * N)).fold(<$vec>::zero(), |sum, i| {
                    let v = [S::from_usize(i / N).unwrap(), S::from_usize(i % N).unwrap()];
                    sum + <$vec>::from(HashGen::$hash(v))
                }) / S::from_usize(N * N).unwrap();
                let var = (0..(N * N)).fold(<$vec>::zero(), |sum, i| {
                    let v = [S::from_usize(i / N).unwrap(), S::from_usize(i % N).unwrap()];
                    let x = <$vec>::from(HashGen::$hash(v)) - (unit * 0.5).cast().unwrap();
                    sum + x.mul_element_wise(x)
                }) / S::from_usize(N * N).unwrap();
                assert!(
                    (mean - (unit * 0.5).cast().unwrap()).magnitude()
                        < S::from_f64(0.05 * 0.5).unwrap(),
                    "mean: {:?}",
                    mean,
                );
                assert!(
                    (var - (unit / 12.0).cast().unwrap()).magnitude()
                        < S::from_f64(0.05 / 12.0).unwrap(),
                    "var: {:?}",
                    var,
                );
            }
            exec_test::<f32>();
            exec_test::<f64>();
        }
    };
}

define_hashx2_test!(hash22_test, Vector2<S>, VEC2UNIT, hash2);
define_hashx2_test!(hash32_test, Vector3<S>, VEC3UNIT, hash3);
define_hashx2_test!(hash42_test, Vector4<S>, VEC4UNIT, hash4);

#[test]
fn hash13_test() {
    fn exec_test<S: BaseFloat + FromPrimitive>() {
        const N: usize = 21;
        let mean = (0..(N * N * N)).fold(S::zero(), |sum, i| {
            let v = [
                S::from_usize(i / (N * N)).unwrap(),
                S::from_usize((i / N) % N).unwrap(),
                S::from_usize(i % N).unwrap(),
            ];
            sum + HashGen::hash1(v)
        }) / S::from_usize(N * N * N).unwrap();
        let var = (0..(N * N * N)).fold(S::zero(), |sum, i| {
            let v = [
                S::from_usize(i / (N * N)).unwrap(),
                S::from_usize((i / N) % N).unwrap(),
                S::from_usize(i % N).unwrap(),
            ];
            let x = HashGen::hash1(v) - S::from_f64(0.5).unwrap();
            sum + x * x
        }) / S::from_usize(N * N * N).unwrap();
        assert!(
            S::abs(mean - S::from_f64(0.5).unwrap()) < S::from_f64(0.05 * 0.5).unwrap(),
            "mean: {mean:?}"
        );
        assert!(
            S::abs(var - S::from_f64(1.0 / 12.0).unwrap()) < S::from_f64(0.05 / 12.0).unwrap(),
            "var: {var:?}"
        );
    }

    exec_test::<f32>();
    exec_test::<f64>();
}

macro_rules! define_hashx3_test {
    ($func: ident, $vec: ty, $unit: expr, $hash: ident) => {
        #[test]
        fn $func() {
            fn exec_test<S: BaseFloat + FromPrimitive>() {
                const N: usize = 21;
                let unit = $unit.cast().unwrap();
                let mean = (0..(N * N * N)).fold(<$vec>::zero(), |sum, i| {
                    let v = [
                        S::from_usize(i / (N * N)).unwrap(),
                        S::from_usize((i / N) % N).unwrap(),
                        S::from_usize(i % N).unwrap(),
                    ];
                    sum + <$vec>::from(HashGen::$hash(v))
                }) / S::from_usize(N * N * N).unwrap();
                let var = (0..(N * N * N)).fold(<$vec>::zero(), |sum, i| {
                    let v = [
                        S::from_usize(i / (N * N)).unwrap(),
                        S::from_usize((i / N) % N).unwrap(),
                        S::from_usize(i % N).unwrap(),
                    ];
                    let x = <$vec>::from(HashGen::$hash(v)) - unit * S::from_f64(0.5).unwrap();
                    sum + x.mul_element_wise(x)
                }) / S::from_usize(N * N * N).unwrap();
                assert!(
                    (mean - unit * S::from_f64(0.5).unwrap()).magnitude()
                        < S::from_f64(0.05 * 0.5).unwrap(),
                    "mean: {:?}",
                    mean,
                );
                assert!(
                    (var - unit * S::from_f64(1.0 / 12.0).unwrap()).magnitude()
                        < S::from_f64(0.05 / 12.0).unwrap(),
                    "var: {:?}",
                    var,
                );
            }

            exec_test::<f32>();
            exec_test::<f64>();
        }
    };
}

define_hashx3_test!(hash23_test, Vector2<S>, VEC2UNIT, hash2);
define_hashx3_test!(hash33_test, Vector3<S>, VEC3UNIT, hash3);
define_hashx3_test!(hash43_test, Vector4<S>, VEC4UNIT, hash4);

#[test]
fn hash14_test() {
    fn exec_test<S: BaseFloat + FromPrimitive>() {
        const N: usize = 10;
        let mean = (0..(N * N * N * N)).fold(S::zero(), |sum, i| {
            let v = [
                S::from_usize(i / (N * N * N)).unwrap(),
                S::from_usize((i / (N * N)) % N).unwrap(),
                S::from_usize((i / N) % N).unwrap(),
                S::from_usize(i % N).unwrap(),
            ];
            sum + HashGen::hash1(v)
        }) / S::from_usize(N * N * N * N).unwrap();
        let var = (0..(N * N * N * N)).fold(S::zero(), |sum, i| {
            let v = [
                S::from_usize(i / (N * N * N)).unwrap(),
                S::from_usize((i / (N * N)) % N).unwrap(),
                S::from_usize((i / N) % N).unwrap(),
                S::from_usize(i % N).unwrap(),
            ];
            let x = HashGen::hash1(v) - S::from_f64(0.5).unwrap();
            sum + x * x
        }) / S::from_usize(N * N * N * N).unwrap();
        assert!(
            S::abs(mean - S::from_f64(0.5).unwrap()) < S::from_f64(0.05 * 0.5).unwrap(),
            "mean: {mean:?}"
        );
        assert!(
            S::abs(var - S::from_f64(1.0 / 12.0).unwrap()) < S::from_f64(0.05 / 12.0).unwrap(),
            "var: {var:?}"
        );
    }

    exec_test::<f32>();
    exec_test::<f64>();
}

macro_rules! define_hashx4_test {
    ($func: ident, $vec: ty, $unit: expr, $hash: ident) => {
        #[test]
        fn $func() {
            fn exec_test<S: BaseFloat + FromPrimitive>() {
                const N: usize = 10;
                let unit = $unit.cast().unwrap();
                let mean = (0..(N * N * N * N)).fold(<$vec>::zero(), |sum, i| {
                    let v = [
                        S::from_usize(i / (N * N * N)).unwrap(),
                        S::from_usize((i / (N * N)) % N).unwrap(),
                        S::from_usize((i / N) % N).unwrap(),
                        S::from_usize(i % N).unwrap(),
                    ];
                    sum + <$vec>::from(HashGen::$hash(v))
                }) / S::from_usize(N * N * N * N).unwrap();
                let var = (0..(N * N * N * N)).fold(<$vec>::zero(), |sum, i| {
                    let v = [
                        S::from_usize(i / (N * N * N)).unwrap(),
                        S::from_usize((i / (N * N)) % N).unwrap(),
                        S::from_usize((i / N) % N).unwrap(),
                        S::from_usize(i % N).unwrap(),
                    ];
                    let x = <$vec>::from(HashGen::$hash(v)) - unit * S::from_f64(0.5).unwrap();
                    sum + x.mul_element_wise(x)
                }) / S::from_usize(N * N * N * N).unwrap();
                assert!(
                    (mean - unit * S::from_f64(0.5).unwrap()).magnitude()
                        < S::from_f64(0.05 * 0.5).unwrap(),
                    "mean: {:?}",
                    mean
                );
                assert!(
                    (var - unit * S::from_f64(1.0 / 12.0).unwrap()).magnitude()
                        < S::from_f64(0.05 / 12.0).unwrap(),
                    "var: {:?}",
                    var
                );
            }

            exec_test::<f32>();
            exec_test::<f64>();
        }
    };
}

define_hashx4_test!(hash24_test, Vector2<S>, VEC2UNIT, hash2);
define_hashx4_test!(hash34_test, Vector3<S>, VEC3UNIT, hash3);
define_hashx4_test!(hash44_test, Vector4<S>, VEC4UNIT, hash4);
