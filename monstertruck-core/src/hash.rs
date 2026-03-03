use cgmath::num_traits::{Float, FromPrimitive};

// SAFETY: Converts finite f64 constants to `S`. All constants used in this
// module are well within the representable range of any `Float + FromPrimitive`
// type (f32, f64), so `from_f64` will never return `None`.
fn f64_to<S: Float + FromPrimitive>(val: f64) -> S { S::from_f64(val).unwrap() }

/// Deterministic hash generator
pub trait HashGen<S> {
    /// deterministic hash, output 1-dim
    fn hash1(seed: Self) -> S;
    /// deterministic hash, output 2-dim
    fn hash2(seed: Self) -> [S; 2];
    /// deterministic hash, output 3-dim
    fn hash3(seed: Self) -> [S; 3];
    /// deterministic hash, output 4-dim
    fn hash4(seed: Self) -> [S; 4];
}

impl<S: Float + FromPrimitive> HashGen<S> for S {
    fn hash1(s: S) -> S {
        let a = f64_to(61.909685033545934);
        let b = f64_to(8.436303256302796);
        let c = f64_to(220.6786200836378);
        let x = S::sin(s * a + b) * c;
        x - S::floor(x)
    }

    fn hash2(s: S) -> [S; 2] {
        let a = f64_to(61.909685033545934);
        let b = f64_to(8.436303256302796);
        let c = f64_to(220.6786200836378);
        let x = S::sin(s * a + b) * c;
        let a = f64_to(92.38848345286779);
        let b = f64_to(2.7771476700831443);
        let c = f64_to(573.3044309816089);
        let y = S::sin(s * a + b) * c;
        [x - S::floor(x), y - S::floor(y)]
    }

    fn hash3(s: S) -> [S; 3] {
        let a = f64_to(61.909685033545934);
        let b = f64_to(8.436303256302796);
        let c = f64_to(220.6786200836378);
        let x = S::sin(s * a + b) * c;
        let a = f64_to(92.38848345286779);
        let b = f64_to(2.7771476700831443);
        let c = f64_to(573.3044309816089);
        let y = S::sin(s * a + b) * c;
        let a = f64_to(69.61119030137992);
        let b = f64_to(0.8814422748956363);
        let c = f64_to(176.56179040382136);
        let z = S::sin(s * a + b) * c;
        [x - S::floor(x), y - S::floor(y), z - S::floor(z)]
    }

    fn hash4(s: S) -> [S; 4] {
        let a = f64_to(61.909685033545934);
        let b = f64_to(8.436303256302796);
        let c = f64_to(220.6786200836378);
        let x = S::sin(s * a + b) * c;
        let a = f64_to(92.38848345286779);
        let b = f64_to(2.7771476700831443);
        let c = f64_to(573.3044309816089);
        let y = S::sin(s * a + b) * c;
        let a = f64_to(69.61119030137992);
        let b = f64_to(0.8814422748956363);
        let c = f64_to(176.56179040382136);
        let z = S::sin(s * a + b) * c;
        let a = f64_to(21.423754555191877);
        let b = f64_to(0.2164601136047869);
        let c = f64_to(871.0649084120648);
        let w = S::sin(s * a + b) * c;
        [
            x - S::floor(x),
            y - S::floor(y),
            z - S::floor(z),
            w - S::floor(w),
        ]
    }
}

impl<S: Float + FromPrimitive> HashGen<S> for [S; 2] {
    fn hash1(v: [S; 2]) -> S {
        let a = HashGen::hash1(v[0]);
        let b = HashGen::hash1(v[1]);
        let c = f64_to(9.784225605373198);
        let d = f64_to(68.94807014710901);
        let e = f64_to(81.49907289737997);
        let x = S::sin(a * c + b * d) * e;
        x - S::floor(x)
    }

    fn hash2(v: [S; 2]) -> [S; 2] {
        let a = HashGen::hash1(v[0]);
        let b = HashGen::hash1(v[1]);
        let c = f64_to(9.784225605373198);
        let d = f64_to(68.94807014710901);
        let e = f64_to(81.49907289737997);
        let x = S::sin(a * c + b * d) * e;
        let c = f64_to(80.84673652708462);
        let d = f64_to(47.747983481580206);
        let e = f64_to(967.6242851986622);
        let y = S::sin(a * c + b * d) * e;
        [x - S::floor(x), y - S::floor(y)]
    }

    fn hash3(v: [S; 2]) -> [S; 3] {
        let a = HashGen::hash1(v[0]);
        let b = HashGen::hash1(v[1]);
        let c = f64_to(9.784225605373198);
        let d = f64_to(68.94807014710901);
        let e = f64_to(81.49907289737997);
        let x = S::sin(a * c + b * d) * e;
        let c = f64_to(80.84673652708462);
        let d = f64_to(47.747983481580206);
        let e = f64_to(967.6242851986622);
        let y = S::sin(a * c + b * d) * e;
        let c = f64_to(75.65061185374819);
        let d = f64_to(0.7529162434507297);
        let e = f64_to(825.2394180776313);
        let z = S::sin(a * c + b * d) * e;
        [x - S::floor(x), y - S::floor(y), z - S::floor(z)]
    }

    fn hash4(v: [S; 2]) -> [S; 4] {
        let a = HashGen::hash1(v[0]);
        let b = HashGen::hash1(v[1]);
        let c = f64_to(9.784225605373198);
        let d = f64_to(68.94807014710901);
        let e = f64_to(81.49907289737997);
        let x = S::sin(a * c + b * d) * e;
        let c = f64_to(80.84673652708462);
        let d = f64_to(47.747983481580206);
        let e = f64_to(967.6242851986622);
        let y = S::sin(a * c + b * d) * e;
        let c = f64_to(75.65061185374819);
        let d = f64_to(0.7529162434507297);
        let e = f64_to(825.2394180776313);
        let z = S::sin(a * c + b * d) * e;
        let c = f64_to(97.2707869579049);
        let d = f64_to(85.3104588821598);
        let e = f64_to(329.18836800713547);
        let w = S::sin(a * c + b * d) * e;
        [
            x - S::floor(x),
            y - S::floor(y),
            z - S::floor(z),
            w - S::floor(w),
        ]
    }
}

impl<S: Float + FromPrimitive> HashGen<S> for [S; 3] {
    fn hash1(v: [S; 3]) -> S {
        let a = HashGen::hash1([v[1], v[2]]);
        let b = HashGen::hash1([v[2], v[0]]);
        let c = HashGen::hash1([v[0], v[1]]);
        let d = f64_to(98.54090184430027);
        let e = f64_to(67.78141684128485);
        let f = f64_to(2.0948068876946224);
        let g = f64_to(280.4659654793662);
        let x = S::sin(a * d + b * e + c * f) * g;
        x - S::floor(x)
    }

    fn hash2(v: [S; 3]) -> [S; 2] {
        let a = HashGen::hash1([v[1], v[2]]);
        let b = HashGen::hash1([v[2], v[0]]);
        let c = HashGen::hash1([v[0], v[1]]);
        let d = f64_to(98.54090184430027);
        let e = f64_to(67.78141684128485);
        let f = f64_to(2.0948068876946224);
        let g = f64_to(280.4659654793662);
        let x = S::sin(a * d + b * e + c * f) * g;
        let d = f64_to(31.298218789251166);
        let e = f64_to(20.629869323489913);
        let f = f64_to(49.45405453340679);
        let g = f64_to(65.41804793662531);
        let y = S::sin(a * d + b * e + c * f) * g;
        [x - S::floor(x), y - S::floor(y)]
    }

    fn hash3(v: [S; 3]) -> [S; 3] {
        let a = HashGen::hash1([v[1], v[2]]);
        let b = HashGen::hash1([v[2], v[0]]);
        let c = HashGen::hash1([v[0], v[1]]);
        let d = f64_to(98.54090184430027);
        let e = f64_to(67.78141684128485);
        let f = f64_to(2.0948068876946224);
        let g = f64_to(280.4659654793662);
        let x = S::sin(a * d + b * e + c * f) * g;
        let d = f64_to(31.298218789251166);
        let e = f64_to(20.629869323489913);
        let f = f64_to(49.45405453340679);
        let g = f64_to(65.41804793662531);
        let y = S::sin(a * d + b * e + c * f) * g;
        let d = f64_to(24.301021506408738);
        let e = f64_to(2.287647662619474);
        let f = f64_to(97.795539177359);
        let g = f64_to(738.3773753517622);
        let z = S::sin(a * d + b * e + c * f) * g;
        [x - S::floor(x), y - S::floor(y), z - S::floor(z)]
    }

    fn hash4(v: [S; 3]) -> [S; 4] {
        let a = HashGen::hash1([v[1], v[2]]);
        let b = HashGen::hash1([v[2], v[0]]);
        let c = HashGen::hash1([v[0], v[1]]);
        let d = f64_to(98.54090184430027);
        let e = f64_to(67.78141684128485);
        let f = f64_to(2.0948068876946224);
        let g = f64_to(280.4659654793662);
        let x = S::sin(a * d + b * e + c * f) * g;
        let d = f64_to(31.298218789251166);
        let e = f64_to(20.629869323489913);
        let f = f64_to(49.45405453340679);
        let g = f64_to(65.41804793662531);
        let y = S::sin(a * d + b * e + c * f) * g;
        let d = f64_to(24.301021506408738);
        let e = f64_to(2.287647662619474);
        let f = f64_to(97.795539177359);
        let g = f64_to(738.3773753517622);
        let z = S::sin(a * d + b * e + c * f) * g;
        let d = f64_to(13.335891984218563);
        let e = f64_to(77.76549475370358);
        let f = f64_to(78.43332391527221);
        let g = f64_to(582.2181553770123);
        let w = S::sin(a * d + b * e + c * f) * g;
        [
            x - S::floor(x),
            y - S::floor(y),
            z - S::floor(z),
            w - S::floor(w),
        ]
    }
}

impl<S: Float + FromPrimitive> HashGen<S> for [S; 4] {
    fn hash1(v: [S; 4]) -> S {
        let a = HashGen::hash1([v[1], v[2], v[3]]);
        let b = HashGen::hash1([v[2], v[3], v[0]]);
        let c = HashGen::hash1([v[3], v[0], v[1]]);
        let d = HashGen::hash1([v[0], v[1], v[2]]);
        let e = f64_to(45.48741506500266);
        let f = f64_to(61.983961049580714);
        let g = f64_to(71.68710748047283);
        let h = f64_to(17.018593362218127);
        let i = f64_to(233.18759725099315);
        let x = S::sin(a * e + b * f + c * g + d * h) * i;
        x - S::floor(x)
    }

    fn hash2(v: [S; 4]) -> [S; 2] {
        let a = HashGen::hash1([v[1], v[2], v[3]]);
        let b = HashGen::hash1([v[2], v[3], v[0]]);
        let c = HashGen::hash1([v[3], v[0], v[1]]);
        let d = HashGen::hash1([v[0], v[1], v[2]]);
        let e = f64_to(45.48741506500266);
        let f = f64_to(61.983961049580714);
        let g = f64_to(71.68710748047283);
        let h = f64_to(17.018593362218127);
        let i = f64_to(233.18759725099315);
        let x = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = f64_to(94.89956622422271);
        let f = f64_to(67.23354063041712);
        let g = f64_to(1.7039983839709838);
        let h = f64_to(32.280513717302526);
        let i = f64_to(939.8840745127328);
        let y = S::sin(a * e + b * f + c * g + d * h) * i;
        [x - S::floor(x), y - S::floor(y)]
    }

    fn hash3(v: [S; 4]) -> [S; 3] {
        let a = HashGen::hash1([v[1], v[2], v[3]]);
        let b = HashGen::hash1([v[2], v[3], v[0]]);
        let c = HashGen::hash1([v[3], v[0], v[1]]);
        let d = HashGen::hash1([v[0], v[1], v[2]]);
        let e = f64_to(45.48741506500266);
        let f = f64_to(61.983961049580714);
        let g = f64_to(71.68710748047283);
        let h = f64_to(17.018593362218127);
        let i = f64_to(233.18759725099315);
        let x = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = f64_to(94.89956622422271);
        let f = f64_to(67.23354063041712);
        let g = f64_to(1.7039983839709838);
        let h = f64_to(32.280513717302526);
        let i = f64_to(939.8840745127328);
        let y = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = f64_to(8.536774960497063);
        let f = f64_to(88.78943466077406);
        let g = f64_to(57.999707672724874);
        let h = f64_to(93.95289146038817);
        let i = f64_to(310.70624415491886);
        let z = S::sin(a * e + b * f + c * g + d * h) * i;
        [x - S::floor(x), y - S::floor(y), z - S::floor(z)]
    }

    fn hash4(v: [S; 4]) -> [S; 4] {
        let a = HashGen::hash1([v[1], v[2], v[3]]);
        let b = HashGen::hash1([v[2], v[3], v[0]]);
        let c = HashGen::hash1([v[3], v[0], v[1]]);
        let d = HashGen::hash1([v[0], v[1], v[2]]);
        let e = f64_to(45.48741506500266);
        let f = f64_to(61.983961049580714);
        let g = f64_to(71.68710748047283);
        let h = f64_to(17.018593362218127);
        let i = f64_to(233.18759725099315);
        let x = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = f64_to(94.89956622422271);
        let f = f64_to(67.23354063041712);
        let g = f64_to(1.7039983839709838);
        let h = f64_to(32.280513717302526);
        let i = f64_to(939.8840745127328);
        let y = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = f64_to(8.536774960497063);
        let f = f64_to(88.78943466077406);
        let g = f64_to(57.999707672724874);
        let h = f64_to(93.95289146038817);
        let i = f64_to(310.70624415491886);
        let z = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = f64_to(46.900144129289025);
        let f = f64_to(38.3329178217256);
        let g = f64_to(95.10984147120259);
        let h = f64_to(38.51337731313087);
        let i = f64_to(843.5063860117572);
        let w = S::sin(a * e + b * f + c * g + d * h) * i;
        [
            x - S::floor(x),
            y - S::floor(y),
            z - S::floor(z),
            w - S::floor(w),
        ]
    }
}

impl<S: Float + FromPrimitive> HashGen<S> for [S; 1] {
    fn hash1(seed: Self) -> S { S::hash1(seed[0]) }
    fn hash2(seed: Self) -> [S; 2] { S::hash2(seed[0]) }
    fn hash3(seed: Self) -> [S; 3] { S::hash3(seed[0]) }
    fn hash4(seed: Self) -> [S; 4] { S::hash4(seed[0]) }
}

macro_rules! derive_hashgen {
    ($from: ty, $into: ty) => {
        impl<S: Float + FromPrimitive> HashGen<S> for $from {
            fn hash1(seed: Self) -> S { <$into>::hash1(seed.into()) }
            fn hash2(seed: Self) -> [S; 2] { <$into>::hash2(seed.into()) }
            fn hash3(seed: Self) -> [S; 3] { <$into>::hash3(seed.into()) }
            fn hash4(seed: Self) -> [S; 4] { <$into>::hash4(seed.into()) }
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
pub fn take_one_unit<G: HashGen<f64>>(seed: G) -> Vector3<f64> {
    let u = HashGen::hash2(seed);
    let theta = 2.0 * std::f64::consts::PI * u[0];
    let z = 2.0 * u[1] - 1.0;
    let r = f64::sqrt(1.0 - z * z);
    Vector3::new(r * f64::cos(theta), r * f64::sin(theta), z)
}
