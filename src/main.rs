// #![feature(slice_patterns)]
use ndarray::prelude::*;

mod utils;
use utils::BinarySeq;

use std::f64::consts::PI;
const N: [usize; 3] = [100, 25, 1];
const DX: [f64; 3] = [5e-9, 5e-9, 3e-9];
static DEMAG_DIM: [usize; 4] = [2 * N[0] - 1, 2 * N[1] - 1, 2 * N[2] - 1, 6];
const _MU0: f64 = 4e-7 * PI;
const _GAMMA: f64 = 2.211e5;
const _MS: f64 = 8e5;
const _A: f64 = 1.3e-11;
const _ALPHA: f64 = 0.02;

const EPS: f64 = std::f64::EPSILON;
// const EPS: f64 = 1e-18;

// use std::iter::FromIterator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prod = DEMAG_DIM.iter().fold(1, |acc, &v| acc * v);
    let mut n_demag = Array::<f64, _>::linspace(0., prod as f64, prod).into_shape(DEMAG_DIM)?;
    let mut n_demag = n_demag.view_mut();

    for (i, t) in [
        (f as fn(&[_]) -> _, [0usize, 1, 2]),
        (g, [0, 1, 2]),
        (g, [0, 2, 1]),
        (f, [1, 2, 0]),
        (g, [1, 2, 0]),
        (f, [2, 0, 1]),
    ]
    .iter()
    .enumerate()
    {
        set_n_demag(&mut n_demag, i, t.0, t.1);
    }

    Ok(())
}

fn set_n_demag(
    demag: &mut ArrayViewMut<f64, Ix4>,
    c: usize,
    func: impl Fn(&[f64]) -> f64,
    permute: [usize; 3],
) {
    for (idx, elem) in demag.slice_mut(s![.., .., .., c]).indexed_iter_mut() {
        let idx = [idx.0, idx.1, idx.2];
        let mut value = 0.;
        for i in BinarySeq::new(6) {
            let idx: Vec<_> = (0..3)
                .map(|k| (idx[k] as usize + N[k] - 1) % (2 * N[k] + 1))
                .collect();
            value += (-1f64).powi(i.iter().fold(0, |acc, &i| acc + i as i32))
                * func(
                    permute
                        .iter()
                        .map(|&j| (idx[j] as f64 + i[j] as f64 - i[j + 3] as f64) * DX[j])
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
        }
        *elem = -value / (4. * PI * DX.iter().fold(1., |acc, v| acc * v));
    }
}

fn _h_eff<D: ndarray::Dimension>(_m: &mut ArrayViewMut<f64, D>) {}

fn f(p: &[f64]) -> f64 {
    let [x, y, z] = [p[0].abs(), p[1].abs(), p[2].abs()];
    // let (x, y, z) = match p {
    //     [x, y, z, ..] => (x.abs(), y.abs(), z.abs()),
    //     _ => panic!(),
    // };

    y / 2.0 * (z * z - x * x) * (y / ((x * x + z * z).sqrt() + EPS)).asinh()
        + z / 2.0 * (y * y - x * x) * (z / ((x * x + y * y).sqrt() + EPS)).asinh()
        - x * y * z * (y * z / (x * (x * x + y * y + z * z).sqrt() + EPS)).atanh()
        + 1.0 / 6.0 * (2. * x * x - y * y - z * z) * (x * x + y * y + z * z).sqrt()
}

fn g(p: &[f64]) -> f64 {
    let [x, y, z] = [p[0], p[1], p[2].abs()];
    // let (x, y, z) = match p {
    //     [x, y, z, ..] => (x, y, z.abs()),
    //     _ => panic!(),
    // };

    x * y * z * (z / ((x * x + y * y).sqrt() + EPS)).asinh()
        + y / 6.0 * (3.0 * z * z - y * y) * (x / ((y * y + z * z).sqrt() + EPS)).asinh()
        + x / 6.0 * (3.0 * z * z - x * x) * (y / ((x * x + z * z).sqrt() + EPS)).asinh()
        - z * z * z / 6.0 * (x * y / (z * (x * x + y * y + z * z).sqrt() + EPS)).atan()
        - z * y * y / 2.0 * (x * z / (y * (x * x + y * y + z * z).sqrt() + EPS)).atan()
        - z * x * x / 2.0 * (y * z / (x * (x * x + y * y + z * z).sqrt() + EPS)).atan()
        - x * y * (x * x + y * y + z * z).sqrt() / 3.0
}

#[cfg(test)]
mod test {
    use super::{f, g};

    fn assert_float(a: f64, b: f64) {
        assert!((a - b).abs() < 2. * std::f64::EPSILON);
    }

    #[test]
    fn test_f() {
        let test_cases = &[
            ([0., 0., 0.], 0.),
            ([-0.1, -0.1, -0.1], -0.000_658_478_948_462_408_4),
            ([0.1, 0.1, 0.1], -0.000_658_478_948_462_408_4),
            ([-1., -1., -1.], -0.658_478_948_462_408_5),
            ([1., 1., 1.], -0.658_478_948_462_408_5),
        ];

        for (input, expected) in test_cases {
            assert_float(f(input), *expected);
        }
    }

    #[test]
    fn test_g() {
        let test_cases = &[
            ([0., 0., 0.], 0.),
            ([-0.1, -0.1, -0.1], -0.000_090_750_593_283_627_15),
            ([0.1, 0.1, 0.1], -0.000_090_750_593_283_627_15),
            ([-1., -1., -1.], -0.090_750_593_283_627_22),
            ([1., 1., 1.], -0.090_750_593_283_627_22),
        ];

        for (input, expected) in test_cases {
            assert_float(g(input), *expected);
        }
    }
}
