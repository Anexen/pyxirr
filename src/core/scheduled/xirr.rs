use super::{year_fraction, DayCount};
use crate::core::{
    models::{validate, DateLike, InvalidPaymentsError},
    optimize::{brentq, newton_raphson},
    utils::{self, fast_pow},
};

pub fn xirr(
    dates: &[DateLike],
    amounts: &[f64],
    guess: Option<f64>,
    day_count: Option<DayCount>,
) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;

    let deltas = &day_count_factor(dates, day_count);

    let f = |rate| {
        if rate <= -1.0 {
            // bound newton_raphson
            return f64::INFINITY;
        }
        xnpv_result(amounts, deltas, rate)
    };
    let df = |rate| xnpv_result_deriv(amounts, deltas, rate);

    let rate = newton_raphson(guess.unwrap_or(0.1), &f, &df);

    if utils::is_a_good_rate(rate, f) {
        return Ok(rate);
    }

    let rate = brentq(&f, -0.999999999999999, 100., 100);

    if utils::is_a_good_rate(rate, f) {
        return Ok(rate);
    }

    let mut step = 0.01;
    let mut guess = -0.99999999999999;
    while guess < 1.0 {
        let rate = newton_raphson(guess, &f, &df);
        if utils::is_a_good_rate(rate, f) {
            return Ok(rate);
        }
        guess += step;
        step = (step * 1.1).min(0.1);
    }

    Ok(f64::NAN)
}

/// Calculate the net present value of a series of payments at irregular intervals.
pub fn xnpv(
    rate: f64,
    dates: &[DateLike],
    amounts: &[f64],
    day_count: Option<DayCount>,
) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;

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
    payments.iter().zip(deltas).map(|(p, &e)| p * fast_pow(1.0 + rate, -e)).sum()
}

// XNPV first derivative
// \sum_{i=1}^n P_i * (d_0 - d_i) / 365 * (1 + rate)^{((d_0 - d_i)/365 - 1)}}
// simplify in order to reuse cached deltas (d_i - d_0)/365
// \sum_{i=1}^n \frac{P_i * -(d_i - d_0) / 365}{(1 + rate)^{((d_i - d_0)/365 + 1)}}
fn xnpv_result_deriv(payments: &[f64], deltas: &[f64], rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, e)| p * -e * fast_pow(1.0 + rate, -e - 1.0)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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
        assert_eq!(zero_crossing_points(&[1., 2., 3.]), vec![]);
        assert_eq!(zero_crossing_points(&[1., -2., -3.]), vec![0]);
        assert_eq!(zero_crossing_points(&[1., -2., 3.]), vec![0, 1]);
        assert_eq!(zero_crossing_points(&[-1., -2., 3.]), vec![1]);
        assert_eq!(zero_crossing_points(&[1., f64::NAN, 3.]), vec![]);

        assert_eq!(
            zero_crossing_points(&[7., 6., -3., -4., -7., 8., 3., -6., 7., 8.]),
            vec![1, 4, 6, 7],
        );
    }
}
