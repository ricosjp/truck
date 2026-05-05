use cgmath::num_traits::{Float, FromPrimitive, ToPrimitive};

/// Deterministic hash generator
pub trait HashGen<S> {
    /// deterministic hash, output 1-dim
    fn hash1(gen: Self) -> S;
    /// deterministic hash, output 2-dim
    fn hash2(gen: Self) -> [S; 2];
    /// deterministic hash, output 3-dim
    fn hash3(gen: Self) -> [S; 3];
    /// deterministic hash, output 4-dim
    fn hash4(gen: Self) -> [S; 4];
}

fn mix64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn float_bits<S: Float + ToPrimitive>(value: S) -> u64 {
    let value = value.to_f64().unwrap();
    if value == 0.0 {
        0
    } else if value.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        value.to_bits()
    }
}

fn hash_value<S: Float + FromPrimitive + ToPrimitive>(values: &[S], channel: u64) -> S {
    let mut state = 0x9e37_79b9_7f4a_7c15 ^ channel.wrapping_mul(0xd1b5_4a32_d192_ed03);
    for (index, value) in values.iter().enumerate() {
        let lane = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        state = mix64(state ^ mix64(float_bits(*value) ^ lane));
    }

    let unit = (mix64(state) >> 11) as f64 * (1.0 / ((1_u64 << 53) as f64));
    S::from_f64(unit).unwrap()
}

fn hash2_value<S: Float + FromPrimitive + ToPrimitive>(values: &[S]) -> [S; 2] {
    [hash_value(values, 0), hash_value(values, 1)]
}

fn hash3_value<S: Float + FromPrimitive + ToPrimitive>(values: &[S]) -> [S; 3] {
    [
        hash_value(values, 0),
        hash_value(values, 1),
        hash_value(values, 2),
    ]
}

fn hash4_value<S: Float + FromPrimitive + ToPrimitive>(values: &[S]) -> [S; 4] {
    [
        hash_value(values, 0),
        hash_value(values, 1),
        hash_value(values, 2),
        hash_value(values, 3),
    ]
}

impl<S: Float + FromPrimitive + ToPrimitive> HashGen<S> for S {
    fn hash1(gen: Self) -> S { hash_value(&[gen], 0) }
    fn hash2(gen: Self) -> [S; 2] { hash2_value(&[gen]) }
    fn hash3(gen: Self) -> [S; 3] { hash3_value(&[gen]) }
    fn hash4(gen: Self) -> [S; 4] { hash4_value(&[gen]) }
}

impl<S: Float + FromPrimitive + ToPrimitive> HashGen<S> for [S; 1] {
    fn hash1(gen: Self) -> S { hash_value(&gen, 0) }
    fn hash2(gen: Self) -> [S; 2] { hash2_value(&gen) }
    fn hash3(gen: Self) -> [S; 3] { hash3_value(&gen) }
    fn hash4(gen: Self) -> [S; 4] { hash4_value(&gen) }
}

impl<S: Float + FromPrimitive + ToPrimitive> HashGen<S> for [S; 2] {
    fn hash1(gen: Self) -> S { hash_value(&gen, 0) }
    fn hash2(gen: Self) -> [S; 2] { hash2_value(&gen) }
    fn hash3(gen: Self) -> [S; 3] { hash3_value(&gen) }
    fn hash4(gen: Self) -> [S; 4] { hash4_value(&gen) }
}

impl<S: Float + FromPrimitive + ToPrimitive> HashGen<S> for [S; 3] {
    fn hash1(gen: Self) -> S { hash_value(&gen, 0) }
    fn hash2(gen: Self) -> [S; 2] { hash2_value(&gen) }
    fn hash3(gen: Self) -> [S; 3] { hash3_value(&gen) }
    fn hash4(gen: Self) -> [S; 4] { hash4_value(&gen) }
}

impl<S: Float + FromPrimitive + ToPrimitive> HashGen<S> for [S; 4] {
    fn hash1(gen: Self) -> S { hash_value(&gen, 0) }
    fn hash2(gen: Self) -> [S; 2] { hash2_value(&gen) }
    fn hash3(gen: Self) -> [S; 3] { hash3_value(&gen) }
    fn hash4(gen: Self) -> [S; 4] { hash4_value(&gen) }
}

macro_rules! derive_hashgen {
    ($from: ty, $into: ty) => {
        impl<S: Float + FromPrimitive + ToPrimitive> HashGen<S> for $from {
            fn hash1(gen: Self) -> S { <$into>::hash1(gen.into()) }
            fn hash2(gen: Self) -> [S; 2] { <$into>::hash2(gen.into()) }
            fn hash3(gen: Self) -> [S; 3] { <$into>::hash3(gen.into()) }
            fn hash4(gen: Self) -> [S; 4] { <$into>::hash4(gen.into()) }
        }
    };
}

use cgmath::*;
derive_hashgen!(Point1<S>, [S; 1]);
derive_hashgen!(Point2<S>, [S; 2]);
derive_hashgen!(Point3<S>, [S; 3]);
derive_hashgen!(Vector1<S>, [S; 1]);
derive_hashgen!(Vector2<S>, [S; 2]);
derive_hashgen!(Vector3<S>, [S; 3]);
derive_hashgen!(Vector4<S>, [S; 4]);

/// Take one random unit vector
pub fn take_one_unit<G: HashGen<f64>>(gen: G) -> Vector3<f64> {
    let u = HashGen::hash2(gen);
    let theta = 2.0 * std::f64::consts::PI * u[0];
    let z = 2.0 * u[1] - 1.0;
    let r = f64::sqrt(1.0 - z * z);
    Vector3::new(r * f64::cos(theta), r * f64::sin(theta), z)
}
