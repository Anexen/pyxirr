use super::models::{validate, DateLike, InvalidPaymentsError};
use super::optimize::find_root_newton_raphson;

pub fn xirr(
    dates: &[DateLike],
    amounts: &[f64],
    guess: Option<f64>,
) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;

    let deltas = precalculate_deltas(&dates);
    let mut guess = guess.unwrap_or_else(|| initial_guess(&dates, &amounts));
    let mut rate = find_rate(&amounts, &deltas, guess);

    guess = -0.99;

    while guess < 1.0 && (rate.is_nan() || rate.is_infinite()) {
        rate = find_rate(&amounts, &deltas, guess);
        guess += 0.01;
    }

    Ok(rate)
}

/// Calculate the net present value of a series of payments at irregular intervals.
pub fn xnpv(rate: f64, dates: &[DateLike], amounts: &[f64]) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;

    let deltas = precalculate_deltas(&dates);

    Ok(xirr_result(amounts, &deltas, rate))
}

fn initial_guess(_dates: &[DateLike], _amounts: &[f64]) -> f64 {
    // TODO smart initial_guess calculation
    0.1
}

fn precalculate_deltas(dates: &[DateLike]) -> Vec<f64> {
    let min_date = dates.iter().min().unwrap();
    return dates.iter().map(|d| (*d - *min_date) as f64 / 365.0).collect();
}

fn find_rate(amounts: &[f64], deltas: &[f64], guess: f64) -> f64 {
    return find_root_newton_raphson(
        guess,
        |rate| xirr_result(amounts, deltas, rate),
        |rate| xirr_result_deriv(amounts, deltas, rate),
    );
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
