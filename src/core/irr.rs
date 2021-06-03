use super::models::{validate, InvalidPaymentsError};
use super::optimize::{find_root_newton_raphson_with_default_deriv, powers};

pub fn npv(rate: f64, values: &[f64], start_from_zero: Option<bool>) -> f64 {
    if rate == 0.0 {
        return values.iter().sum();
    }

    powers(1. + rate, values.len(), start_from_zero.unwrap_or(true))
        .iter()
        .zip(values.iter())
        .map(|(p, v)| v / p)
        .sum()
}

pub fn irr(values: &[f64], guess: Option<f64>) -> Result<f64, InvalidPaymentsError> {
    validate(values, None)?;

    Ok(find_root_newton_raphson_with_default_deriv(guess.unwrap_or(0.1), |rate| {
        npv(rate, values, Some(true))
    }))
}
