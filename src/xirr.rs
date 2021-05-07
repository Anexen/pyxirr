use chrono::prelude::*;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

const MAX_ERROR: f64 = 1e-10;
const MAX_COMPUTE_WITH_GUESS_ITERATIONS: u32 = 50;

/// A payment made or received on a particular date.
/// `amount` must be negative for payment made and positive for payment received.
/// TODO: FromPyObject trait
#[derive(Copy, Clone)]
pub struct Payment {
    pub date: NaiveDate,
    pub amount: f64,
}

/// An error returned when the payments do not contain both negative and positive payments.
#[derive(Debug)]
pub struct InvalidPaymentsError;

impl Display for InvalidPaymentsError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        "negative and positive payments are required".fmt(f)
    }
}

impl Error for InvalidPaymentsError {}

pub fn xirr(payments: &Vec<Payment>, guess: Option<f64>) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let deltas = precalculate_deltas(&payments);
    let mut rate = compute_with_guess(&payments, &deltas, guess.unwrap_or(0.1));
    let mut guess = -0.99;

    while guess < 1.0 && (rate.is_nan() || rate.is_infinite()) {
        rate = compute_with_guess(&payments, &deltas, guess);
        guess += 0.01;
    }

    Ok(rate)
}

/// Calculate the net present value of a series of payments at irregular intervals.
pub fn xnpv(rate: f64, payments: &Vec<Payment>) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let deltas = precalculate_deltas(&payments);

    Ok(xirr_result(payments, &deltas, rate))
}

fn precalculate_deltas(payments: &Vec<Payment>) -> Vec<f64> {
    let min_date = payments.iter().min_by_key(|p| p.date).unwrap().date;
    return payments.iter().map(|p| (p.date - min_date).num_days() as f64 / 365.0).collect();
}

fn compute_with_guess(payments: &Vec<Payment>, deltas: &Vec<f64>, guess: f64) -> f64 {
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

fn xirr_result(payments: &Vec<Payment>, deltas: &Vec<f64>, rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, exp)| p.amount / (1.0 + rate).powf(*exp)).sum()
}

fn xirr_result_deriv(payments: &Vec<Payment>, deltas: &Vec<f64>, rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, exp)| p.amount * exp / (1.0 + rate).powf(exp + 1.0)).sum()
}

fn validate(payments: &Vec<Payment>) -> Result<(), InvalidPaymentsError> {
    let positive = payments.iter().any(|p| p.amount > 0.0);
    let negative = payments.iter().any(|p| p.amount < 0.0);

    if positive && negative {
        Ok(())
    } else {
        Err(InvalidPaymentsError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xirr_unordered() {
        let payments = vec![
            Payment { date: "2015-06-11".parse().unwrap(), amount: -1000.0 },
            Payment { date: "2015-07-21".parse().unwrap(), amount: -9000.0 },
            Payment { date: "2018-06-10".parse().unwrap(), amount: 20000.0 },
            Payment { date: "2015-10-17".parse().unwrap(), amount: -3000.0 },
        ];

        let result = xirr(&payments, None).unwrap();
        let expected = 0.1635371584432640;

        assert!((result - expected).abs() <= MAX_ERROR);
    }

    #[test]
    fn test_xnpv() {
        let payments = vec![
            Payment { date: "2015-06-11".parse().unwrap(), amount: -1000.0 },
            Payment { date: "2015-07-21".parse().unwrap(), amount: -9000.0 },
            Payment { date: "2018-06-10".parse().unwrap(), amount: 20000.0 },
            Payment { date: "2015-10-17".parse().unwrap(), amount: -3000.0 },
        ];
        let result = xnpv(0.1, &payments).unwrap();
        let expected = 2218.42566365671;

        assert!((result - expected).abs() <= MAX_ERROR);
    }
}
