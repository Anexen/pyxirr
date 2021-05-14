use super::models::{InvalidPaymentsError, Payment};

const MAX_ERROR: f64 = 1e-10;
const MAX_COMPUTE_WITH_GUESS_ITERATIONS: u32 = 50;

pub fn xirr(payments: &[Payment], guess: Option<f64>) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let deltas = precalculate_deltas(&payments);
    let mut guess = guess.unwrap_or_else(|| initial_guess(&payments));
    let mut rate = compute_with_guess(&payments, &deltas, guess);

    guess = -0.99;

    while guess < 1.0 && (rate.is_nan() || rate.is_infinite()) {
        rate = compute_with_guess(&payments, &deltas, guess);
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

fn compute_with_guess(payments: &[Payment], deltas: &[f64], guess: f64) -> f64 {
    let mut rate = guess;

    for _ in 0..MAX_COMPUTE_WITH_GUESS_ITERATIONS {
        let div = xirr_result(payments, deltas, rate) / xirr_result_deriv(payments, deltas, rate);
        rate += div;

        if div.abs() <= MAX_ERROR {
            return rate;
        }
    }

    f64::NAN
}

fn xirr_result(payments: &[Payment], deltas: &[f64], rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, exp)| p.amount / (1.0 + rate).powf(*exp)).sum()
}

fn xirr_result_deriv(payments: &[Payment], deltas: &[f64], rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, exp)| p.amount * exp / (1.0 + rate).powf(exp + 1.0)).sum()
}

fn validate(payments: &[Payment]) -> Result<(), InvalidPaymentsError> {
    let positive = payments.iter().any(|p| p.amount > 0.0);
    let negative = payments.iter().any(|p| p.amount < 0.0);

    if positive && negative {
        Ok(())
    } else {
        Err(InvalidPaymentsError)
    }
}
