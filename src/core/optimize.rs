const MAX_ERROR: f64 = 1e-9;
const MAX_ITERATIONS: u32 = 50;

pub fn newton_raphson<Func, Deriv>(start: f64, f: &Func, d: &Deriv) -> f64
where
    Func: Fn(f64) -> f64,
    Deriv: Fn(f64) -> f64,
{
    // x[n + 1] = x[n] - f(x[n])/f'(x[n])

    let mut x = start;

    for _ in 0..MAX_ITERATIONS {
        let res = f(x);

        if res.abs() < MAX_ERROR {
            return x;
        }

        let delta = res / d(x);

        if delta.abs() < MAX_ERROR {
            return x - delta;
        }

        x -= delta;
    }

    f64::NAN
}

pub fn newton_raphson_with_default_deriv<Func>(start: f64, f: Func) -> f64
where
    Func: Fn(f64) -> f64,
{
    // deriv = (f(x + e) - f(x - e))/((x + e) - x)
    // multiply denominator by 2 for faster convergence

    // https://programmingpraxis.com/2012/01/13/excels-xirr-function/

    let df = |x| (f(x + MAX_ERROR) - f(x - MAX_ERROR)) / (2.0 * MAX_ERROR);
    newton_raphson(start, &f, &df)
}

// https://github.com/scipy/scipy/blob/39bf11b96f771dcecf332977fb2c7843a9fd55f2/scipy/optimize/Zeros/brentq.c
pub fn brentq<Func>(f: &Func, xa: f64, xb: f64, iter: usize) -> f64
where
    Func: Fn(f64) -> f64,
{
    let xtol = 2e-14;
    let rtol = 8.881784197001252e-16;

    let mut xpre = xa;
    let mut xcur = xb;
    let (mut xblk, mut fblk, mut spre, mut scur) = (0., 0., 0., 0.);
    /* the tolerance is 2*delta */

    let mut fpre = f(xpre);
    let mut fcur = f(xcur);

    if fpre.signum() == fcur.signum() {
        return f64::NAN; // sign error
    }
    if fpre == 0. {
        return xpre;
    }
    if fcur == 0. {
        return xcur;
    }

    for _ in 0..iter {
        if fpre != 0. && fcur != 0. && fpre.signum() != fcur.signum() {
            xblk = xpre;
            fblk = fpre;
            spre = xcur - xpre;
            scur = spre;
        }

        if fblk.abs() < fcur.abs() {
            xpre = xcur;
            xcur = xblk;
            xblk = xpre;

            fpre = fcur;
            fcur = fblk;
            fblk = fpre;
        }

        let delta = (xtol + rtol * xcur.abs()) / 2.;
        let sbis = (xblk - xcur) / 2.;

        if fcur == 0. || sbis.abs() < delta {
            return xcur;
        }

        if spre.abs() > delta && fcur.abs() < fpre.abs() {
            let stry = if xpre == xblk {
                /* interpolate */
                -fcur * (xcur - xpre) / (fcur - fpre)
            } else {
                /* extrapolate */
                let dpre = (fpre - fcur) / (xpre - xcur);
                let dblk = (fblk - fcur) / (xblk - xcur);
                -fcur * (fblk * dblk - fpre * dpre) / (dblk * dpre * (fblk - fpre))
            };

            if 2. * stry.abs() < spre.abs().min(3. * sbis.abs() - delta) {
                /* good short step */
                spre = scur;
                scur = stry;
            } else {
                /* bisect */
                spre = sbis;
                scur = sbis;
            }
        } else {
            /* bisect */
            spre = sbis;
            scur = sbis;
        }

        xpre = xcur;
        fpre = fcur;
        if scur.abs() > delta {
            xcur += scur;
        } else {
            xcur += if sbis > 0. {
                delta
            } else {
                -delta
            }
        }

        fcur = f(xcur);
    }

    f64::NAN
}

pub fn brentq_grid_search<'a, Func>(
    breakpoints: &'a [&[f64]],
    f: &'a Func,
) -> impl Iterator<Item = f64> + 'a
where
    Func: Fn(f64) -> f64 + 'a,
{
    breakpoints
        .iter()
        .flat_map(|x| x.windows(2).map(|pair| brentq(f, pair[0], pair[1], 100)))
        .filter(|r| r.is_finite() && f(*r).abs() < 1e-3)
}

// use std::f64::consts::PI;
//
// use num_complex::Complex;

// pub fn durand_kerner(coefficients: &[f64]) -> Vec<f64> {
//     // https://github.com/TheAlgorithms/C-Plus-Plus/blob/master/numerical_methods/durand_kerner_roots.cpp#L109
//
//     // numerical errors less when the first coefficient is "1"
//     // hence, we normalize the first coefficient
//     let coefficients: Vec<_> = coefficients.iter().map(|x| x / coefficients[0]).collect();
//     let degree = coefficients.len() - 1;
//     let accuracy = 1e-10;
//
//     let mut roots: Vec<_> = (0..degree)
//         .into_iter()
//         .map(|i| Complex::<f64>::new(PI * (i as f64 / degree as f64), 0.0))
//         .collect();
//
//     let mut prev_delta = f64::INFINITY;
//
//     for _ in 0..MAX_ITERATIONS {
//         let mut tol_condition = 0.0f64;
//
//         for n in 0..degree {
//             let numerator = polyval(&coefficients, roots[n]);
//
//             let mut denominator = Complex::new(1.0, 0.0);
//             for i in 0..degree {
//                 if i != n {
//                     denominator *= roots[n] - roots[i];
//                 }
//             }
//
//             let delta = numerator / denominator;
//
//             if !delta.norm().is_finite() {
//                 break;
//             }
//
//             roots[n] -= delta;
//
//             tol_condition = tol_condition.max(delta.norm())
//         }
//
//         if (prev_delta - tol_condition).abs() <= accuracy || tol_condition < accuracy {
//             break;
//         }
//
//         prev_delta = tol_condition
//     }
//
//     roots.into_iter().map(|x| x.norm()).collect()
// }

// valuate a polynomial at specific values.
// fn polyval(coefficients: &[f64], x: Complex<f64>) -> Complex<f64> {
//     let degree = coefficients.len() - 1;
//     coefficients.iter().enumerate().map(|(i, c)| c * x.powf((degree - i) as f64)).sum()
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use assert_approx_eq::assert_approx_eq;
//     use rstest::rstest;
//
//     #[rstest]
//     fn test_durand_kerner() {
//         let cf = &[-1e6, 5000., -3.];
//         let roots = durand_kerner(cf);
//
//         dbg!(&roots);
//         for root in roots {
//             let guess = root - 1.;
//             dbg!(guess);
//             let rate = crate::core::irr(cf, Some(guess)).unwrap();
//             assert_approx_eq!(crate::core::npv(rate, cf, None), 0.0);
//         }
//     }
// }
