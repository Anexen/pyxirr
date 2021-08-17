use crate::core::models::{validate, DateLike, InvalidPaymentsError};
use crate::core::optimize::find_root_newton_raphson;

pub fn xirr(
    dates: &[DateLike],
    amounts: &[f64],
    guess: Option<f64>,
) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;

    let ref deltas = precalculate_deltas(&dates);

    let guess = guess.unwrap_or(0.1);

    let rate = find_rate(amounts, deltas, guess);

    if is_good_rate(rate, amounts, deltas) {
        return Ok(rate);
    }

    let rate = find_guess_in_range(-0.999, -0.99, 0.001, amounts, deltas);

    if is_good_rate(rate, amounts, deltas) {
        return Ok(rate);
    }

    let rate = find_guess_in_range(-0.99, 1.0, 0.01, amounts, deltas);

    if is_good_rate(rate, amounts, deltas) {
        return Ok(rate);
    }

    Ok(f64::NAN)
}

/// Calculate the net present value of a series of payments at irregular intervals.
pub fn xnpv(rate: f64, dates: &[DateLike], amounts: &[f64]) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;

    let deltas = precalculate_deltas(&dates);

    Ok(xirr_result(amounts, &deltas, rate))
}

// fn smart_guess(amounts: &[f64]) -> f64 {
//     amounts.iter().sum::<f64>() / -amounts.iter().filter(|&x| x < &0.0).sum::<f64>()
// }

fn is_good_rate(rate: f64, amounts: &[f64], deltas: &[f64]) -> bool {
    // rate must be finite and XNPV must be close to zero
    rate.is_finite() && xirr_result(amounts, deltas, rate).abs() < 1e-3
}

fn find_guess_in_range(min: f64, max: f64, step: f64, amounts: &[f64], deltas: &[f64]) -> f64 {
    let mut guess = min;
    while guess < max {
        let rate = find_rate(amounts, deltas, guess);
        if is_good_rate(rate, amounts, deltas) {
            return rate;
        }
        guess += step;
    }
    f64::NAN
}

fn precalculate_deltas(dates: &[DateLike]) -> Vec<f64> {
    let min_date = dates.iter().min().unwrap();
    dates.iter().map(|d| (*d - *min_date) as f64 / 365.0).collect()
}

fn find_rate(amounts: &[f64], deltas: &[f64], guess: f64) -> f64 {
    find_root_newton_raphson(
        guess,
        |rate| xirr_result(amounts, deltas, rate),
        |rate| xirr_result_deriv(amounts, deltas, rate),
    )
}

// \sum_{i=1}^n \frac{P_i}{(1 + rate)^{(d_i - d_0)/365}}
fn xirr_result(payments: &[f64], deltas: &[f64], rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, &e)| p / (1.0 + rate).powf(e)).sum()
}

// XIRR first derivative
// \sum_{i=1}^n P_i * (d_0 - d_i) / 365 * (1 + rate)^{((d_0 - d_i)/365 - 1)}}
// simplify in order to reuse cached deltas (d_i - d_0)/365
// \sum_{i=1}^n \frac{P_i * -(d_i - d_0) / 365}{(1 + rate)^{((d_1 - d_0)/365 + 1)}}
fn xirr_result_deriv(payments: &[f64], deltas: &[f64], rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, e)| p * -e / (1.0 + rate).powf(e + 1.0)).sum()
}
