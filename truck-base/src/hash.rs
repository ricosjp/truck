use cgmath::num_traits::{Float, FromPrimitive};

/// deterministic hash, input 1-dim, output 1-dim
pub fn hash11<S: Float + FromPrimitive>(s: S) -> S {
	let a = S::from_f64(61.909685033545934).unwrap();
	let b = S::from_f64(8.436303256302796).unwrap();
	let c = S::from_f64(220.6786200836378).unwrap();
	let x = S::sin(s * a + b) * c;
	x - S::floor(x)
}

/// deterministic hash, input 1-dim, output 2-dim
pub fn hash21<S: Float + FromPrimitive>(s: S) -> [S; 2] {
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

/// deterministic hash, input 1-dim, output 3-dim
pub fn hash31<S: Float + FromPrimitive>(s: S) -> [S; 3] {
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

/// deterministic hash, input 1-dim, output 4-dim
pub fn hash41<S: Float + FromPrimitive>(s: S) -> [S; 4] {
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
	[x - S::floor(x), y - S::floor(y), z - S::floor(z), w - S::floor(w)]
}

/// deterministic hash, input 2-dim, output 1-dim
pub fn hash12<S: Float + FromPrimitive>(v: [S; 2]) -> S {
	let a = hash11(v[0]);
	let b = hash11(v[1]);
	let c = S::from_f64(9.784225605373198).unwrap();
	let d = S::from_f64(68.94807014710901).unwrap();
	let e = S::from_f64(81.49907289737997).unwrap();
	let x = S::sin(a * c + b * d) * e;
	x - S::floor(x)
}

/// deterministic hash, input 2-dim, output 2-dim
pub fn hash22<S: Float + FromPrimitive>(v: [S; 2]) -> [S; 2] {
	let a = hash11(v[0]);
	let b = hash11(v[1]);
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

/// deterministic hash, input 2-dim, output 3-dim
pub fn hash32<S: Float + FromPrimitive>(v: [S; 2]) -> [S; 3] {
	let a = hash11(v[0]);
	let b = hash11(v[1]);
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

/// deterministic hash, input 2-dim, output 4-dim
pub fn hash42<S: Float + FromPrimitive>(v: [S; 2]) -> [S; 4] {
	let a = hash11(v[0]);
	let b = hash11(v[1]);
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
	[x - S::floor(x), y - S::floor(y), z - S::floor(z), w - S::floor(w)]
}

/// deterministic hash, input 3-dim, output 1-dim
pub fn hash13<S: Float + FromPrimitive>(v: [S; 3]) -> S {
	let a = hash12([v[1], v[2]]);
	let b = hash12([v[2], v[0]]);
	let c = hash12([v[0], v[1]]);
	let d = S::from_f64(98.54090184430027).unwrap();
	let e = S::from_f64(67.78141684128485).unwrap();
	let f = S::from_f64(2.0948068876946224).unwrap();
	let g = S::from_f64(280.4659654793662).unwrap();
	let x = S::sin(a * d + b * e + c * f) * g;
	x - S::floor(x)
}

/// deterministic hash, input 3-dim, output 2-dim
pub fn hash23<S: Float + FromPrimitive>(v: [S; 3]) -> [S; 2] {
	let a = hash12([v[1], v[2]]);
	let b = hash12([v[2], v[0]]);
	let c = hash12([v[0], v[1]]);
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

/// deterministic hash, input 3-dim, output 3-dim
pub fn hash33<S: Float + FromPrimitive>(v: [S; 3]) -> [S; 3] {
	let a = hash12([v[1], v[2]]);
	let b = hash12([v[2], v[0]]);
	let c = hash12([v[0], v[1]]);
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

/// deterministic hash, input 3-dim, output 4-dim
pub fn hash43<S: Float + FromPrimitive>(v: [S; 3]) -> [S; 4] {
	let a = hash12([v[1], v[2]]);
	let b = hash12([v[2], v[0]]);
	let c = hash12([v[0], v[1]]);
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
	[x - S::floor(x), y - S::floor(y), z - S::floor(z), w - S::floor(w)]
}

/// deterministic hash, input 4-dim, output 1-dim
pub fn hash14<S: Float + FromPrimitive>(v: [S; 4]) -> S {
	let a = hash13([v[1], v[2], v[3]]);
	let b = hash13([v[2], v[3], v[0]]);
	let c = hash13([v[3], v[0], v[1]]);
	let d = hash13([v[0], v[1], v[2]]);
	let e = S::from_f64(45.48741506500266).unwrap();
	let f = S::from_f64(61.983961049580714).unwrap();
	let g = S::from_f64(71.68710748047283).unwrap();
	let h = S::from_f64(17.018593362218127).unwrap();
	let i = S::from_f64(233.18759725099315).unwrap();
	let x = S::sin(a * e + b * f + c * g + d * h) * i;
	x - S::floor(x)
}

/// deterministic hash, input 4-dim, output 2-dim
pub fn hash24<S: Float + FromPrimitive>(v: [S; 4]) -> [S; 2] {
	let a = hash13([v[1], v[2], v[3]]);
	let b = hash13([v[2], v[3], v[0]]);
	let c = hash13([v[3], v[0], v[1]]);
	let d = hash13([v[0], v[1], v[2]]);
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

/// deterministic hash, input 4-dim, output 3-dim
pub fn hash34<S: Float + FromPrimitive>(v: [S; 4]) -> [S; 3] {
	let a = hash13([v[1], v[2], v[3]]);
	let b = hash13([v[2], v[3], v[0]]);
	let c = hash13([v[3], v[0], v[1]]);
	let d = hash13([v[0], v[1], v[2]]);
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

/// deterministic hash, input 4-dim, output 4-dim
pub fn hash44<S: Float + FromPrimitive>(v: [S; 4]) -> [S; 4] {
	let a = hash13([v[1], v[2], v[3]]);
	let b = hash13([v[2], v[3], v[0]]);
	let c = hash13([v[3], v[0], v[1]]);
	let d = hash13([v[0], v[1], v[2]]);
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
	let f = S::from_f64( 38.3329178217256).unwrap();
	let g = S::from_f64( 95.10984147120259).unwrap();
	let h = S::from_f64(38.51337731313087).unwrap();
	let i = S::from_f64(843.5063860117572).unwrap();
	let w = S::sin(a * e + b * f + c * g + d * h) * i;
	[x - S::floor(x), y - S::floor(y), z - S::floor(z), w - S::floor(w)]
}
