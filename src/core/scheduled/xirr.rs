use super::{year_fraction, DayCount};
use crate::core::{
    models::{validate, DateLike, InvalidPaymentsError},
    optimize::find_root,
};

pub fn xirr(
    dates: &[DateLike],
    amounts: &[f64],
    guess: Option<f64>,
    day_count: Option<DayCount>,
) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;

    let deltas = &day_count_factor(dates, day_count);

    Ok(find_root(
        guess.unwrap_or(0.1),
        &[(-0.99, 1.0, 0.01)],
        |rate| xnpv_result(amounts, deltas, rate),
        |rate| xnpv_result_deriv(amounts, deltas, rate),
    ))
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

// fn smart_guess(amounts: &[f64]) -> f64 {
//     amounts.iter().sum::<f64>() / -amounts.iter().filter(|&x| x < &0.0).sum::<f64>()
// }

fn day_count_factor(dates: &[DateLike], day_count: Option<DayCount>) -> Vec<f64> {
    let min_date = dates.iter().min().unwrap();
    let dc = day_count.unwrap_or_default();
    dates.iter().map(|d| year_fraction(&min_date, &d, dc)).collect()
}

// \sum_{i=1}^n \frac{P_i}{(1 + rate)^{(d_i - d_0)/365}}
fn xnpv_result(payments: &[f64], deltas: &[f64], rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, &e)| p * (1.0 + rate).powf(-e)).sum()
}

// XNPV first derivative
// \sum_{i=1}^n P_i * (d_0 - d_i) / 365 * (1 + rate)^{((d_0 - d_i)/365 - 1)}}
// simplify in order to reuse cached deltas (d_i - d_0)/365
// \sum_{i=1}^n \frac{P_i * -(d_i - d_0) / 365}{(1 + rate)^{((d_i - d_0)/365 + 1)}}
fn xnpv_result_deriv(payments: &[f64], deltas: &[f64], rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, e)| p * -e * (1.0 + rate).powf(-e - 1.0)).sum()
}
