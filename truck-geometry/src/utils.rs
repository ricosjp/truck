use crate::*;

pub fn solve_quadratic(a: f64, b: f64, c: f64) -> Vec<f64> {
    match a.so_small() {
        true => match b.so_small() {
            true => Vec::new(),
            false => vec![-b / c],
        },
        false => {
            let det = b * b - 4.0 * a * c;
            match det >= 0.0 {
                true => {
                    let sqrt_det = f64::sqrt(det);
                    vec![(-b - sqrt_det) / (2.0 * a), (-b + sqrt_det) / (2.0 * a)]
                }
                false => Vec::new(),
            }
        }
    }
}

pub fn solve_cubic(a: f64, b: f64, c: f64, d: f64) -> Vec<f64> {
    match a.so_small() {
        true => solve_quadratic(b, c, d),
        false => {
            let (b, c, d) = (b / a, c / a, d / a);
            let p = c - b * b / 3.0;
            let q = d - b * c / 3.0 + 2.0 * b * b * b / 27.0;
            let mut res = pre_solve_cubic(p, q);
            res.iter_mut().for_each(|x| {
                *x -= b / 3.0;
            });
            res
        }
    }
}

/// solve equation: t^3 + p t + q = 0.
pub fn pre_solve_cubic(p: f64, q: f64) -> Vec<f64> {
    use num::complex::Complex;
    const OMEGA: Complex<f64> = Complex::new(-0.5, 0.86602540378);
    const OMEGA2: Complex<f64> = Complex::new(-0.5, -0.86602540378);
    let p_3 = p / 3.0;
    let q_2 = q / 2.0;
    let alpha2 = q_2 * q_2 + p_3 * p_3 * p_3;
    let (x, y) = match alpha2 > -TOLERANCE {
        true => {
            let alpha = f64::sqrt(f64::max(alpha2, 0.0));
            let tmpx = -q_2 - alpha;
            let tmpy = -q_2 + alpha;
            (
                Complex::new(
                    f64::signum(tmpx) * f64::powf(f64::abs(tmpx), 1.0 / 3.0),
                    0.0,
                ),
                Complex::new(
                    f64::signum(tmpy) * f64::powf(f64::abs(tmpy), 1.0 / 3.0),
                    0.0,
                ),
            )
        }
        false => {
            let alphai = f64::sqrt(-alpha2);
            (
                Complex::powf(Complex::new(-q_2, alphai), 1.0 / 3.0),
                Complex::powf(Complex::new(-q_2, -alphai), 1.0 / 3.0),
            )
        }
    };
    [x + y, OMEGA * x + OMEGA2 * y, OMEGA2 * x + OMEGA * y]
        .iter()
        .filter_map(|z| match z.im.so_small() {
            true => Some(z.re),
            false => None,
        })
        .collect()
}

#[test]
fn solve_cubic_test() {
    // example
    let res = solve_cubic(2.0, -3.0, -23.0, 12.0);
    assert_eq!(res.len(), 3);
    let mut ans = vec![0.5, -3.0, 4.0];
    res.into_iter().for_each(|x| {
        let idx = ans
            .iter()
            .enumerate()
            .find_map(|(i, y)| match x.near(y) {
                true => Some(i),
                false => None,
            })
            .unwrap();
        ans.swap_remove(idx);
    });

    // random
    (0..50).for_each(|_| {
        let a = 100.0 * rand::random::<f64>() - 50.0;
        let b = 100.0 * rand::random::<f64>() - 50.0;
        let c = 100.0 * rand::random::<f64>() - 50.0;
        let d = 100.0 * rand::random::<f64>() - 50.0;
        let vec = solve_cubic(a, b, c, d);
        assert!(a.so_small() || !vec.is_empty(), "{a} {b} {c} {d}");
        vec.into_iter()
            .for_each(|t| assert!((a * t * t * t + b * t * t + c * t + d).so_small()));
    });
}

#[test]
fn pre_solve_cubic_test() {
    // example
    let res = pre_solve_cubic(-7.0, -6.0);
    assert_eq!(res.len(), 3);
    let mut ans = vec![-1.0, -2.0, 3.0];
    res.into_iter().for_each(|x| {
        let idx = ans
            .iter()
            .enumerate()
            .find_map(|(i, y)| match x.near(y) {
                true => Some(i),
                false => None,
            })
            .unwrap();
        ans.swap_remove(idx);
    });

    // random
    (0..50).for_each(|_| {
        let p = 100.0 * rand::random::<f64>() - 50.0;
        let q = 100.0 * rand::random::<f64>() - 50.0;
        let vec = pre_solve_cubic(p, q);
        assert!(!vec.is_empty(), "{p} {q}");
        vec.into_iter()
            .for_each(|t| assert!((t * t * t + p * t + q).so_small()));
    })
}
