use crate::core::models::{validate, DateLike, InvalidPaymentsError};
use crate::core::optimize::find_root;

pub fn xirr(
    dates: &[DateLike],
    amounts: &[f64],
    guess: Option<f64>,
) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;

    let ref deltas = precalculate_deltas(&dates);

    Ok(find_root(
        guess.unwrap_or(0.1),
        &[(-0.99, 1.0, 0.01)],
        |rate| xirr_result(amounts, deltas, rate),
        |rate| xirr_result_deriv(amounts, deltas, rate),
    ))
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

fn precalculate_deltas(dates: &[DateLike]) -> Vec<f64> {
    let min_date = dates.iter().min().unwrap();
    dates.iter().map(|d| (*d - *min_date) as f64 / 365.0).collect()
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
