use super::{year_fraction, DayCount};
use crate::core::{
    models::{validate, validate_length, DateLike, InvalidPaymentsError},
    optimize::{brentq, newton_raphson_2},
    utils::{fast_pow, initial_guess},
};

pub fn xirr(
    dates: &[DateLike],
    amounts: &[f64],
    guess: Option<f64>,
    day_count: Option<DayCount>,
) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;

    let deltas = &day_count_factor(dates, day_count);

    if amounts.len() == 2 {
        return Ok(xirr_analytical_2(amounts, deltas));
    }

    // XIRR is unchanged by a common positive cash-flow scale. Normalize before
    // evaluating the objective and derivative so absolute solver tolerances do
    // not depend on currency units (or reject a root after large cancellation).
    let scale = amounts.iter().fold(0.0_f64, |acc, p| acc.max(p.abs()));
    let normalized: Vec<f64> = amounts.iter().map(|p| p / scale).collect();
    let amounts = normalized.as_slice();

    let f = |rate| xnpv_result(amounts, deltas, rate);
    let fd = |rate| xnpv_result_with_deriv(amounts, deltas, rate);

    // Normalized residuals can satisfy the absolute stopping tolerance before
    // the rate reaches machine precision. Refine accepted candidates locally.
    let refine = |mut rate: f64| {
        for _ in 0..3 {
            let (value, deriv) = fd(rate);
            let next = rate - value / deriv;
            if !next.is_finite() || next <= -1.0 || f(next).abs() > value.abs() {
                break;
            }
            rate = next;
        }
        rate
    };

    let guess = guess.unwrap_or_else(|| initial_guess(amounts));
    let rate = newton_raphson_2(guess, &fd);

    if rate.is_finite() {
        return Ok(refine(rate));
    }

    let rate = brentq(&f, -0.999999999999999, 100., 100);

    if rate.is_finite() {
        return Ok(refine(rate));
    }

    let mut step = 0.01;
    let mut guess = -0.99999999999999;
    while guess < 1.0 {
        let rate = newton_raphson_2(guess, &fd);
        if rate.is_finite() {
            return Ok(refine(rate));
        }
        guess += step;
        step = (step * 1.1).min(0.1);
    }

    Ok(f64::NAN)
}

fn xirr_analytical_2(amounts: &[f64], deltas: &[f64]) -> f64 {
    // solve analytically:
    // cf[0]/(1+r)^d[0] + cf[1]/(1+r)^d[1] = 0  =>
    // cf[1]/(1+r)^d[1] = -cf[0]/(1+r)^d[0]  => rearrange
    // cf[1]/cf[0] = -(1+r)^d[1]/(1+r)^d[0]  => simplify
    // cf[1]/cf[0] = -(1+r)^(d[1] - d[0])  => take the root
    // (cf[1]/cf[0])^(1/(d[1] - d[0])) = -(1 + r) => multiply by -1 and subtract 1
    // r = -(cf[1]/cf[0])^(1/(d[1] - d[0])) - 1
    (-amounts[1] / amounts[0]).powf(1. / (deltas[1] - deltas[0])) - 1.0
}

/// Calculate the net present value of a series of payments at irregular intervals.
pub fn xnpv(
    rate: f64,
    dates: &[DateLike],
    amounts: &[f64],
    day_count: Option<DayCount>,
) -> Result<f64, InvalidPaymentsError> {
    validate_length(amounts, dates)?;

    let deltas = &day_count_factor(dates, day_count);
    Ok(xnpv_result(amounts, deltas, rate))
}

pub fn sign_changes(v: &[f64]) -> i32 {
    v.windows(2)
        .map(|p| (p[0].is_finite() && p[1].is_finite() && p[0].signum() != p[1].signum()) as i32)
        .sum()
}

pub fn zero_crossing_points(v: &[f64]) -> Vec<usize> {
    v.windows(2)
        .enumerate()
        .filter_map(|(i, p)| {
            (p[0].is_finite() && p[1].is_finite() && p[0].signum() != p[1].signum()).then_some(i)
        })
        .collect()
}

fn day_count_factor(dates: &[DateLike], day_count: Option<DayCount>) -> Vec<f64> {
    let min_date = dates.iter().min().unwrap();
    let dc = day_count.unwrap_or_default();
    dates.iter().map(|d| year_fraction(&min_date, &d, dc)).collect()
}

// \sum_{i=1}^n \frac{P_i}{(1 + rate)^{(d_i - d_0)/365}}
fn xnpv_result(payments: &[f64], deltas: &[f64], rate: f64) -> f64 {
    if rate <= -1.0 {
        // bound newton_raphson
        return f64::INFINITY;
    }
    payments.iter().zip(deltas).map(|(p, &e)| p * fast_pow(1.0 + rate, -e)).sum()
}

// XNPV first derivative
// \sum_{i=1}^n P_i * (d_0 - d_i) / 365 * (1 + rate)^{((d_0 - d_i)/365 - 1)}}
// simplify in order to reuse cached deltas (d_i - d_0)/365
// \sum_{i=1}^n \frac{P_i * -(d_i - d_0) / 365}{(1 + rate)^{((d_i - d_0)/365 + 1)}}
// fn xnpv_result_deriv(payments: &[f64], deltas: &[f64], rate: f64) -> f64 {
//     payments.iter().zip(deltas).map(|(p, e)| p * -e * fast_pow(1.0 + rate, -e - 1.0)).sum()
// }

fn xnpv_result_with_deriv(payments: &[f64], deltas: &[f64], rate: f64) -> (f64, f64) {
    if rate <= -1.0 {
        return (f64::INFINITY, f64::INFINITY);
    }
    // pow is an expensive function.
    // we can re-use the result of pow for derivative calculation
    payments.iter().zip(deltas).fold((0.0, 0.0), |acc, (p, e)| {
        let y0 = p * fast_pow(1.0 + rate, -e);
        let y1 = y0 * -e / (1.0 + rate);
        (acc.0 + y0, acc.1 + y1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(1e-12)]
    #[case(1.0)]
    #[case(1e6)]
    #[case(1e12)]
    fn test_xirr_scale_invariance(#[case] scale: f64) {
        let dates: Vec<DateLike> =
            ["2021-01-01", "2022-01-01", "2023-01-01"].iter().map(|d| d.parse().unwrap()).collect();
        // An independently soluble quadratic: -100 + 144/(1+r)^2 = 0.
        let amounts = [-100.0 * scale, 0.0, 144.0 * scale];
        let rate = xirr(&dates, &amounts, None, None).unwrap();
        assert!((rate - 0.2).abs() < 1e-8, "scale={scale}, rate={rate}");
        // Preserve selection of both roots with explicit guesses.
        let amounts = [-100.0 * scale, 230.0 * scale, -132.0 * scale];
        for (guess, expected) in [(0.08, 0.1), (0.23, 0.2)] {
            let rate = xirr(&dates, &amounts, Some(guess), None).unwrap();
            assert!((rate - expected).abs() < 1e-8);
        }
    }

    #[rstest]
    #[case(1.0)]
    #[case(0.001)]
    #[case(1e6)]
    fn test_xirr_issue_75(#[case] scale: f64) {
        let dates: Vec<DateLike> = [
            "2018-06-30",
            "2019-03-31",
            "2019-06-30",
            "2019-09-30",
            "2019-12-31",
            "2020-03-31",
            "2020-06-30",
            "2020-09-30",
            "2022-03-31",
            "2023-03-31",
            "2024-09-30",
            "2024-09-30",
            "2024-12-31",
            "2024-12-31",
            "2025-03-31",
            "2025-12-31",
            "2026-03-31",
            "2026-03-31",
        ]
        .iter()
        .map(|d| d.parse().unwrap())
        .collect();
        let amounts: Vec<f64> = [
            -84000000., 800000., 760000., 680000., 800000., 280000., 740000., 520000., -2100000.,
            -13000000., -3000000., -4100000., -250000., -1800000., -8400000., 1800000., 43000.,
            -6600.,
        ]
        .iter()
        .map(|p| p * scale)
        .collect();
        let rate = xirr(&dates, &amounts, None, None).unwrap();
        assert!((rate - (-0.912781382229958)).abs() < 1e-10, "scale={scale}, rate={rate}");
    }

    #[rstest]
    fn test_sign_changes() {
        assert_eq!(sign_changes(&[1., 2., 3.]), 0);
        assert_eq!(sign_changes(&[1., 2., -3.]), 1);
        assert_eq!(sign_changes(&[1., -2., 3.]), 2);
        assert_eq!(sign_changes(&[-1., 2., -3.]), 2);
        assert_eq!(sign_changes(&[-1., -2., -3.]), 0);
        assert_eq!(sign_changes(&[1., f64::NAN, 3.]), 0);
    }

    #[rstest]
    fn test_zero_crossing_points() {
        assert_eq!(zero_crossing_points(&[1., 2., 3.]), Vec::<usize>::new());
        assert_eq!(zero_crossing_points(&[1., -2., -3.]), vec![0]);
        assert_eq!(zero_crossing_points(&[1., -2., 3.]), vec![0, 1]);
        assert_eq!(zero_crossing_points(&[-1., -2., 3.]), vec![1]);
        assert_eq!(zero_crossing_points(&[1., f64::NAN, 3.]), Vec::<usize>::new());

        assert_eq!(
            zero_crossing_points(&[7., 6., -3., -4., -7., 8., 3., -6., 7., 8.]),
            vec![1, 4, 6, 7],
        );
    }
}
