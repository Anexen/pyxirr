use super::models::{validate, InvalidPaymentsError, Payment};
use super::optimize::find_root_newton_raphson;

pub fn xirr(payments: &[Payment], guess: Option<f64>) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let deltas = precalculate_deltas(&payments);
    let mut guess = guess.unwrap_or_else(|| initial_guess(&payments));
    let mut rate = find_rate(&payments, &deltas, guess);

    guess = -0.99;

    while guess < 1.0 && (rate.is_nan() || rate.is_infinite()) {
        rate = find_rate(&payments, &deltas, guess);
        guess += 0.01;
    }

    Ok(rate)
}

/// Calculate the net present value of a series of payments at irregular intervals.
pub fn xnpv(rate: f64, payments: &[Payment]) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let deltas = precalculate_deltas(&payments);

    Ok(xirr_result(payments, &deltas, rate))
}

fn initial_guess(_payments: &[Payment]) -> f64 {
    // TODO smart initial_guess calculation
    0.1
}

fn precalculate_deltas(payments: &[Payment]) -> Vec<f64> {
    let min_date = payments.iter().min_by_key(|p| p.date).unwrap().date;
    return payments.iter().map(|p| (p.date - min_date) as f64 / 365.0).collect();
}

fn find_rate(payments: &[Payment], deltas: &[f64], guess: f64) -> f64 {
    return find_root_newton_raphson(
        guess,
        |rate| xirr_result(payments, deltas, rate),
        |rate| xirr_result_deriv(payments, deltas, rate),
    );
}

// \sum_{i=1}^n \frac{P_i}{(1 + rate)^{(d_i - d_0)/365}}
fn xirr_result(payments: &[Payment], deltas: &[f64], rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, &e)| p.amount / (1.0 + rate).powf(e)).sum()
}

// XIRR first derivative
// \sum_{i=1}^n P_i * (d_0 - d_i) / 365 * (1 + rate)^{((d_0 - d_i)/365 - 1)}}
// simplify in order to reuse cached deltas (d_i - d_0)/365
// \sum_{i=1}^n \frac{P_i * -(d_i - d_0) / 365}{(1 + rate)^{((d_1 - d_0)/365 + 1)}}
fn xirr_result_deriv(payments: &[Payment], deltas: &[f64], rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, e)| p.amount * -e / (1.0 + rate).powf(e + 1.0)).sum()
}
