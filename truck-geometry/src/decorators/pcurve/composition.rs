use super::*;

fn can_init(len: usize, n: usize, max: usize) -> bool { !(len > n || max * len < n) }

fn init(array: &mut [usize], n: usize, max: usize) {
    if array.is_empty() {
        return;
    }
    array[0] = (n - array.len() + 1).min(max);
    let (n, max) = (n - array[0], array[0]);
    init(&mut array[1..], n, max)
}

fn next(array: &mut [usize]) -> bool {
    let n = array[1..].iter().sum::<usize>() + 1;
    let max = array[0] - 1;
    if array.len() == 1 {
        false
    } else if next(&mut array[1..]) {
        true
    } else if can_init(array.len() - 1, n, max) {
        array[0] -= 1;
        init(&mut array[1..], n, max);
        true
    } else {
        false
    }
}

#[derive(Clone, Debug)]
pub struct CompositionIter<const MAX: usize> {
    current: [usize; MAX],
    end: bool,
    len: usize,
}

impl<const MAX: usize> CompositionIter<MAX> {
    pub fn try_new(n: usize, len: usize) -> Option<Self> {
        if !(len < MAX && can_init(len, n, n)) {
            return None;
        }
        let mut current = [0; MAX];
        init(&mut current[..len], n, n);
        Some(Self {
            current,
            len,
            end: false,
        })
    }
}

impl<const MAX: usize> Iterator for CompositionIter<MAX> {
    type Item = [usize; MAX];
    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        let current = self.current;
        self.end = !next(&mut self.current[..self.len]);
        Some(current)
    }
}

fn factorial(n: usize) -> u128 { (2..=n).fold(1, |f, i| f * i as u128) }

pub fn multiplicity(array: &[usize]) -> u128 {
    let n = array.iter().sum::<usize>();
    let mut res = factorial(n);
    array.iter().for_each(|&a| res /= factorial(a));
    let mut mult = 1;
    array.windows(2).for_each(|x| {
        if x[0] == x[1] {
            mult += 1;
        } else {
            res /= factorial(mult);
            mult = 1;
        }
    });
    res / factorial(mult)
}

pub fn tensor<V, A>(sder: &[A], cder: &[Vector2], idx: &[usize]) -> V
where
    V: VectorSpace<Scalar = f64>,
    A: AsRef<[V]>, {
    let n: u128 = 2u128.pow(idx.len() as u32);
    (0..n).fold(V::zero(), |sum, mut i| {
        let (t, mult) = idx.iter().fold((0, 1.0), |(t, mult), &j| {
            let k = (i % 2) as usize;
            i /= 2;
            (t + k, mult * cder[j][k])
        });
        sum + sder[idx.len() - t].as_ref()[t] * mult
    })
}

#[test]
fn test_composition_iter() {
    let iter = CompositionIter::<8>::try_new(10, 4).unwrap();
    let vec: Vec<_> = iter.collect();
    let iter = vec.iter().map(|idx| {
        idx[..4].iter().for_each(|&i| assert_ne!(i, 0));
        idx[4..].iter().for_each(|&i| assert_eq!(i, 0));
        &idx[..4]
    });
    let vec: Vec<_> = iter.collect();

    assert_eq!(vec.len(), 9);
    assert_eq!(vec[0], &[7, 1, 1, 1]);
    assert_eq!(vec[1], &[6, 2, 1, 1]);
    assert_eq!(vec[2], &[5, 3, 1, 1]);
    assert_eq!(vec[3], &[5, 2, 2, 1]);
    assert_eq!(vec[4], &[4, 4, 1, 1]);
    assert_eq!(vec[5], &[4, 3, 2, 1]);
    assert_eq!(vec[6], &[4, 2, 2, 2]);
    assert_eq!(vec[7], &[3, 3, 3, 1]);
    assert_eq!(vec[8], &[3, 3, 2, 2]);
}
