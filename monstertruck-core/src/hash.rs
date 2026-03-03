use cgmath::num_traits::{Float, FromPrimitive};

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

impl<S: Float + FromPrimitive> HashGen<S> for S {
    fn hash1(s: S) -> S {
        let a = S::from_f64(61.909685033545934).unwrap();
        let b = S::from_f64(8.436303256302796).unwrap();
        let c = S::from_f64(220.6786200836378).unwrap();
        let x = S::sin(s * a + b) * c;
        x - S::floor(x)
    }

    fn hash2(s: S) -> [S; 2] {
        let a = S::from_f64(61.909685033545934).unwrap();
        let b = S::from_f64(8.436303256302796).unwrap();
        let c = S::from_f64(220.6786200836378).unwrap();
        let x = S::sin(s * a + b) * c;
        let a = S::from_f64(92.38848345286779).unwrap();
        let b = S::from_f64(2.7771476700831443).unwrap();
        let c = S::from_f64(573.3044309816089).unwrap();
        let y = S::sin(s * a + b) * c;
        [x - S::floor(x), y - S::floor(y)]
    }

    fn hash3(s: S) -> [S; 3] {
        let a = S::from_f64(61.909685033545934).unwrap();
        let b = S::from_f64(8.436303256302796).unwrap();
        let c = S::from_f64(220.6786200836378).unwrap();
        let x = S::sin(s * a + b) * c;
        let a = S::from_f64(92.38848345286779).unwrap();
        let b = S::from_f64(2.7771476700831443).unwrap();
        let c = S::from_f64(573.3044309816089).unwrap();
        let y = S::sin(s * a + b) * c;
        let a = S::from_f64(69.61119030137992).unwrap();
        let b = S::from_f64(0.8814422748956363).unwrap();
        let c = S::from_f64(176.56179040382136).unwrap();
        let z = S::sin(s * a + b) * c;
        [x - S::floor(x), y - S::floor(y), z - S::floor(z)]
    }

    fn hash4(s: S) -> [S; 4] {
        let a = S::from_f64(61.909685033545934).unwrap();
        let b = S::from_f64(8.436303256302796).unwrap();
        let c = S::from_f64(220.6786200836378).unwrap();
        let x = S::sin(s * a + b) * c;
        let a = S::from_f64(92.38848345286779).unwrap();
        let b = S::from_f64(2.7771476700831443).unwrap();
        let c = S::from_f64(573.3044309816089).unwrap();
        let y = S::sin(s * a + b) * c;
        let a = S::from_f64(69.61119030137992).unwrap();
        let b = S::from_f64(0.8814422748956363).unwrap();
        let c = S::from_f64(176.56179040382136).unwrap();
        let z = S::sin(s * a + b) * c;
        let a = S::from_f64(21.423754555191877).unwrap();
        let b = S::from_f64(0.2164601136047869).unwrap();
        let c = S::from_f64(871.0649084120648).unwrap();
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
        let c = S::from_f64(9.784225605373198).unwrap();
        let d = S::from_f64(68.94807014710901).unwrap();
        let e = S::from_f64(81.49907289737997).unwrap();
        let x = S::sin(a * c + b * d) * e;
        x - S::floor(x)
    }

    fn hash2(v: [S; 2]) -> [S; 2] {
        let a = HashGen::hash1(v[0]);
        let b = HashGen::hash1(v[1]);
        let c = S::from_f64(9.784225605373198).unwrap();
        let d = S::from_f64(68.94807014710901).unwrap();
        let e = S::from_f64(81.49907289737997).unwrap();
        let x = S::sin(a * c + b * d) * e;
        let c = S::from_f64(80.84673652708462).unwrap();
        let d = S::from_f64(47.747983481580206).unwrap();
        let e = S::from_f64(967.6242851986622).unwrap();
        let y = S::sin(a * c + b * d) * e;
        [x - S::floor(x), y - S::floor(y)]
    }

    fn hash3(v: [S; 2]) -> [S; 3] {
        let a = HashGen::hash1(v[0]);
        let b = HashGen::hash1(v[1]);
        let c = S::from_f64(9.784225605373198).unwrap();
        let d = S::from_f64(68.94807014710901).unwrap();
        let e = S::from_f64(81.49907289737997).unwrap();
        let x = S::sin(a * c + b * d) * e;
        let c = S::from_f64(80.84673652708462).unwrap();
        let d = S::from_f64(47.747983481580206).unwrap();
        let e = S::from_f64(967.6242851986622).unwrap();
        let y = S::sin(a * c + b * d) * e;
        let c = S::from_f64(75.65061185374819).unwrap();
        let d = S::from_f64(0.7529162434507297).unwrap();
        let e = S::from_f64(825.2394180776313).unwrap();
        let z = S::sin(a * c + b * d) * e;
        [x - S::floor(x), y - S::floor(y), z - S::floor(z)]
    }

    fn hash4(v: [S; 2]) -> [S; 4] {
        let a = HashGen::hash1(v[0]);
        let b = HashGen::hash1(v[1]);
        let c = S::from_f64(9.784225605373198).unwrap();
        let d = S::from_f64(68.94807014710901).unwrap();
        let e = S::from_f64(81.49907289737997).unwrap();
        let x = S::sin(a * c + b * d) * e;
        let c = S::from_f64(80.84673652708462).unwrap();
        let d = S::from_f64(47.747983481580206).unwrap();
        let e = S::from_f64(967.6242851986622).unwrap();
        let y = S::sin(a * c + b * d) * e;
        let c = S::from_f64(75.65061185374819).unwrap();
        let d = S::from_f64(0.7529162434507297).unwrap();
        let e = S::from_f64(825.2394180776313).unwrap();
        let z = S::sin(a * c + b * d) * e;
        let c = S::from_f64(97.2707869579049).unwrap();
        let d = S::from_f64(85.3104588821598).unwrap();
        let e = S::from_f64(329.18836800713547).unwrap();
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
        let d = S::from_f64(98.54090184430027).unwrap();
        let e = S::from_f64(67.78141684128485).unwrap();
        let f = S::from_f64(2.0948068876946224).unwrap();
        let g = S::from_f64(280.4659654793662).unwrap();
        let x = S::sin(a * d + b * e + c * f) * g;
        x - S::floor(x)
    }

    fn hash2(v: [S; 3]) -> [S; 2] {
        let a = HashGen::hash1([v[1], v[2]]);
        let b = HashGen::hash1([v[2], v[0]]);
        let c = HashGen::hash1([v[0], v[1]]);
        let d = S::from_f64(98.54090184430027).unwrap();
        let e = S::from_f64(67.78141684128485).unwrap();
        let f = S::from_f64(2.0948068876946224).unwrap();
        let g = S::from_f64(280.4659654793662).unwrap();
        let x = S::sin(a * d + b * e + c * f) * g;
        let d = S::from_f64(31.298218789251166).unwrap();
        let e = S::from_f64(20.629869323489913).unwrap();
        let f = S::from_f64(49.45405453340679).unwrap();
        let g = S::from_f64(65.41804793662531).unwrap();
        let y = S::sin(a * d + b * e + c * f) * g;
        [x - S::floor(x), y - S::floor(y)]
    }

    fn hash3(v: [S; 3]) -> [S; 3] {
        let a = HashGen::hash1([v[1], v[2]]);
        let b = HashGen::hash1([v[2], v[0]]);
        let c = HashGen::hash1([v[0], v[1]]);
        let d = S::from_f64(98.54090184430027).unwrap();
        let e = S::from_f64(67.78141684128485).unwrap();
        let f = S::from_f64(2.0948068876946224).unwrap();
        let g = S::from_f64(280.4659654793662).unwrap();
        let x = S::sin(a * d + b * e + c * f) * g;
        let d = S::from_f64(31.298218789251166).unwrap();
        let e = S::from_f64(20.629869323489913).unwrap();
        let f = S::from_f64(49.45405453340679).unwrap();
        let g = S::from_f64(65.41804793662531).unwrap();
        let y = S::sin(a * d + b * e + c * f) * g;
        let d = S::from_f64(24.301021506408738).unwrap();
        let e = S::from_f64(2.287647662619474).unwrap();
        let f = S::from_f64(97.795539177359).unwrap();
        let g = S::from_f64(738.3773753517622).unwrap();
        let z = S::sin(a * d + b * e + c * f) * g;
        [x - S::floor(x), y - S::floor(y), z - S::floor(z)]
    }

    fn hash4(v: [S; 3]) -> [S; 4] {
        let a = HashGen::hash1([v[1], v[2]]);
        let b = HashGen::hash1([v[2], v[0]]);
        let c = HashGen::hash1([v[0], v[1]]);
        let d = S::from_f64(98.54090184430027).unwrap();
        let e = S::from_f64(67.78141684128485).unwrap();
        let f = S::from_f64(2.0948068876946224).unwrap();
        let g = S::from_f64(280.4659654793662).unwrap();
        let x = S::sin(a * d + b * e + c * f) * g;
        let d = S::from_f64(31.298218789251166).unwrap();
        let e = S::from_f64(20.629869323489913).unwrap();
        let f = S::from_f64(49.45405453340679).unwrap();
        let g = S::from_f64(65.41804793662531).unwrap();
        let y = S::sin(a * d + b * e + c * f) * g;
        let d = S::from_f64(24.301021506408738).unwrap();
        let e = S::from_f64(2.287647662619474).unwrap();
        let f = S::from_f64(97.795539177359).unwrap();
        let g = S::from_f64(738.3773753517622).unwrap();
        let z = S::sin(a * d + b * e + c * f) * g;
        let d = S::from_f64(13.335891984218563).unwrap();
        let e = S::from_f64(77.76549475370358).unwrap();
        let f = S::from_f64(78.43332391527221).unwrap();
        let g = S::from_f64(582.2181553770123).unwrap();
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
        let e = S::from_f64(45.48741506500266).unwrap();
        let f = S::from_f64(61.983961049580714).unwrap();
        let g = S::from_f64(71.68710748047283).unwrap();
        let h = S::from_f64(17.018593362218127).unwrap();
        let i = S::from_f64(233.18759725099315).unwrap();
        let x = S::sin(a * e + b * f + c * g + d * h) * i;
        x - S::floor(x)
    }

    fn hash2(v: [S; 4]) -> [S; 2] {
        let a = HashGen::hash1([v[1], v[2], v[3]]);
        let b = HashGen::hash1([v[2], v[3], v[0]]);
        let c = HashGen::hash1([v[3], v[0], v[1]]);
        let d = HashGen::hash1([v[0], v[1], v[2]]);
        let e = S::from_f64(45.48741506500266).unwrap();
        let f = S::from_f64(61.983961049580714).unwrap();
        let g = S::from_f64(71.68710748047283).unwrap();
        let h = S::from_f64(17.018593362218127).unwrap();
        let i = S::from_f64(233.18759725099315).unwrap();
        let x = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = S::from_f64(94.89956622422271).unwrap();
        let f = S::from_f64(67.23354063041712).unwrap();
        let g = S::from_f64(1.7039983839709838).unwrap();
        let h = S::from_f64(32.280513717302526).unwrap();
        let i = S::from_f64(939.8840745127328).unwrap();
        let y = S::sin(a * e + b * f + c * g + d * h) * i;
        [x - S::floor(x), y - S::floor(y)]
    }

    fn hash3(v: [S; 4]) -> [S; 3] {
        let a = HashGen::hash1([v[1], v[2], v[3]]);
        let b = HashGen::hash1([v[2], v[3], v[0]]);
        let c = HashGen::hash1([v[3], v[0], v[1]]);
        let d = HashGen::hash1([v[0], v[1], v[2]]);
        let e = S::from_f64(45.48741506500266).unwrap();
        let f = S::from_f64(61.983961049580714).unwrap();
        let g = S::from_f64(71.68710748047283).unwrap();
        let h = S::from_f64(17.018593362218127).unwrap();
        let i = S::from_f64(233.18759725099315).unwrap();
        let x = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = S::from_f64(94.89956622422271).unwrap();
        let f = S::from_f64(67.23354063041712).unwrap();
        let g = S::from_f64(1.7039983839709838).unwrap();
        let h = S::from_f64(32.280513717302526).unwrap();
        let i = S::from_f64(939.8840745127328).unwrap();
        let y = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = S::from_f64(8.536774960497063).unwrap();
        let f = S::from_f64(88.78943466077406).unwrap();
        let g = S::from_f64(57.999707672724874).unwrap();
        let h = S::from_f64(93.95289146038817).unwrap();
        let i = S::from_f64(310.70624415491886).unwrap();
        let z = S::sin(a * e + b * f + c * g + d * h) * i;
        [x - S::floor(x), y - S::floor(y), z - S::floor(z)]
    }

    fn hash4(v: [S; 4]) -> [S; 4] {
        let a = HashGen::hash1([v[1], v[2], v[3]]);
        let b = HashGen::hash1([v[2], v[3], v[0]]);
        let c = HashGen::hash1([v[3], v[0], v[1]]);
        let d = HashGen::hash1([v[0], v[1], v[2]]);
        let e = S::from_f64(45.48741506500266).unwrap();
        let f = S::from_f64(61.983961049580714).unwrap();
        let g = S::from_f64(71.68710748047283).unwrap();
        let h = S::from_f64(17.018593362218127).unwrap();
        let i = S::from_f64(233.18759725099315).unwrap();
        let x = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = S::from_f64(94.89956622422271).unwrap();
        let f = S::from_f64(67.23354063041712).unwrap();
        let g = S::from_f64(1.7039983839709838).unwrap();
        let h = S::from_f64(32.280513717302526).unwrap();
        let i = S::from_f64(939.8840745127328).unwrap();
        let y = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = S::from_f64(8.536774960497063).unwrap();
        let f = S::from_f64(88.78943466077406).unwrap();
        let g = S::from_f64(57.999707672724874).unwrap();
        let h = S::from_f64(93.95289146038817).unwrap();
        let i = S::from_f64(310.70624415491886).unwrap();
        let z = S::sin(a * e + b * f + c * g + d * h) * i;
        let e = S::from_f64(46.900144129289025).unwrap();
        let f = S::from_f64(38.3329178217256).unwrap();
        let g = S::from_f64(95.10984147120259).unwrap();
        let h = S::from_f64(38.51337731313087).unwrap();
        let i = S::from_f64(843.5063860117572).unwrap();
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
    fn hash1(gen: Self) -> S { S::hash1(gen[0]) }
    fn hash2(gen: Self) -> [S; 2] { S::hash2(gen[0]) }
    fn hash3(gen: Self) -> [S; 3] { S::hash3(gen[0]) }
    fn hash4(gen: Self) -> [S; 4] { S::hash4(gen[0]) }
}

macro_rules! derive_hashgen {
    ($from: ty, $into: ty) => {
        impl<S: Float + FromPrimitive> HashGen<S> for $from {
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
