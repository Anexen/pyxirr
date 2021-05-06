// Copyright 2018 Chandra Sekar S
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # XIRR
//!
//! `xirr` implements the XIRR function found in spreadsheet applications like LibreOffice Calc.
//!
//! # Example
//!
//! ```
//! ```

use chrono::prelude::*;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

const MAX_ERROR: f64 = 1e-10;
const MAX_COMPUTE_WITH_GUESS_ITERATIONS: u32 = 50;

/// A payment made or received on a particular date.
///
/// `amount` must be negative for payment made and positive for payment received.
#[derive(Copy, Clone)]
pub struct Payment {
    pub date: NaiveDate,
    pub amount: f64,
}

/// An error returned when the payments provided to [`compute`](fn.compute.html) do not contain
/// both negative and positive payments.
#[derive(Debug)]
pub struct InvalidPaymentsError;

impl Display for InvalidPaymentsError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        "negative and positive payments are required".fmt(f)
    }
}

impl Error for InvalidPaymentsError {}

/// Calculates the internal rate of return of a series of irregular payments.
///
/// It tries to identify the rate of return using Newton's method with an initial guess of 0.1.
/// If that does not provide a solution, it attempts with guesses from -0.99 to 0.99
/// in increments of 0.01 and returns f64:NAN if not succeeded.
///
/// # Errors
///
/// This function will return [`InvalidPaymentsError`](struct.InvalidPaymentsError.html)
/// if both positive and negative payments are not provided.

pub fn compute(payments: &Vec<Payment>) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let deltas = precalculate_deltas(&payments);
    let mut rate = compute_with_guess(&payments, &deltas, 0.1);
    let mut guess = -0.99;

    while guess < 1.0 && (rate.is_nan() || rate.is_infinite()) {
        rate = compute_with_guess(&payments, &deltas, guess);
        guess += 0.01;
    }

    Ok(rate)
}

pub fn xnpv(rate: f64, payments: &Vec<Payment>) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let deltas = precalculate_deltas(&payments);

    Ok(xirr(payments, &deltas, rate))
}

fn precalculate_deltas(payments: &Vec<Payment>) -> Vec<f64> {
    let min_date = payments.iter().min_by_key(|p| p.date).unwrap().date;
    return payments.iter().map(|p| (p.date - min_date).num_days() as f64 / 365.0).collect();
}

fn compute_with_guess(payments: &Vec<Payment>, deltas: &Vec<f64>, guess: f64) -> f64 {
    let mut rate = guess;

    for _ in 0..MAX_COMPUTE_WITH_GUESS_ITERATIONS {
        let div = xirr(payments, deltas, rate) / xirr_deriv(payments, deltas, rate);
        rate += div;

        if div.abs() <= MAX_ERROR {
            return rate;
        }
    }

    f64::NAN
}

fn xirr(payments: &Vec<Payment>, deltas: &Vec<f64>, rate: f64) -> f64 {
    payments.iter().zip(deltas).map(|(p, exp)| p.amount / (1.0 + rate).powf(*exp)).sum()
}

fn xirr_deriv(payments: &Vec<Payment>, deltas: &Vec<f64>, rate: f64) -> f64 {
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

        let result = compute(&payments).unwrap();
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
