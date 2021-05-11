use chrono::prelude::*;
use pyo3::types::{PyAny, PyDate, PyDateAccess};
use pyo3::{exceptions, FromPyObject, PyResult};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

const MAX_ERROR: f64 = 1e-10;
const MAX_COMPUTE_WITH_GUESS_ITERATIONS: u32 = 50;

/// A payment made or received on a particular date.
/// `amount` must be negative for payment made and positive for payment received.
#[derive(Debug, Clone, PartialEq)]
pub struct Payment {
    pub date: NaiveDate,
    pub amount: f64,
}

fn extract_date(py_date: &PyDate) -> NaiveDate {
    NaiveDate::from_ymd(py_date.get_year(), py_date.get_month() as u32, py_date.get_day() as u32)
}

impl<'s> FromPyObject<'s> for Payment {
    fn extract(obj: &'s PyAny) -> PyResult<Self> {
        let date: &PyAny = obj.get_item(0)?;
        let amount: f64 = obj.get_item(1)?.extract()?;

        let date = if date.is_instance::<PyDate>()? {
            extract_date(date.downcast::<PyDate>()?)
        } else {
            match date.get_type().name()? {
                "datetime64" => NaiveDate::from_num_days_from_ce(
                    date.call_method1("astype", ("datetime64[D]",))?
                        .call_method1("astype", ("int64",))?
                        .extract()?,
                ),
                "Timestamp" => extract_date(date.call_method0("date")?.downcast::<PyDate>()?),
                other => {
                    return Err(exceptions::PyTypeError::new_err(format!(
                        "Type {:?} is not understood",
                        other
                    )))
                }
            }
        };

        Ok(Payment { date, amount })
    }
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

pub fn xirr(payments: &[Payment], guess: Option<f64>) -> Result<f64, InvalidPaymentsError> {
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
pub fn xnpv(rate: f64, payments: &[Payment]) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let deltas = precalculate_deltas(&payments);

    Ok(xirr_result(payments, &deltas, rate))
}

fn precalculate_deltas(payments: &[Payment]) -> Vec<f64> {
    let min_date = payments.iter().min_by_key(|p| p.date).unwrap().date;
    return payments.iter().map(|p| (p.date - min_date).num_days() as f64 / 365.0).collect();
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
