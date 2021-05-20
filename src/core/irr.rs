use super::models::{validate, InvalidPaymentsError};
use super::optimize::find_root_newton_raphson_with_default_deriv;
use std::iter::successors;

// pre calculating powers for performance
fn powers(base: f64, n: usize) -> Vec<f64> {
    successors(Some(base), |x| Some(x * base)).take(n).collect()
}

fn npv_result(rate: f64, values: &[f64]) -> f64 {
    if rate == 0.0 {
        return values.iter().sum();
    }

    powers(1. + rate, values.len()).iter().zip(values.iter()).map(|(p, v)| v / p).sum()
}

pub fn npv(rate: f64, values: &[f64]) -> Result<f64, InvalidPaymentsError> {
    validate(values, None)?;

    Ok(npv_result(rate, values))
}

pub fn irr(values: &[f64], guess: Option<f64>) -> Result<f64, InvalidPaymentsError> {
    validate(values, None)?;

    Ok(find_root_newton_raphson_with_default_deriv(guess.unwrap_or(0.1), |rate| {
        npv_result(rate, values)
    }))
}
