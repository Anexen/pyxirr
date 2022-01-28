use std::iter::successors;

use super::models::{validate, InvalidPaymentsError};
use super::optimize::{brentq, find_root, newton_raphson_with_default_deriv};

// pre calculating powers for performance
pub fn powers(base: f64, n: usize, start_from_zero: bool) -> Vec<f64> {
    let (start, n) = if start_from_zero { (1.0, n + 1) } else { (base, n) };
    successors(Some(start), |x| Some(x * base)).take(n).collect()
}

fn convert_pmt_at_begining(pmt_at_begining: Option<bool>) -> f64 {
    if pmt_at_begining.unwrap_or(false) {
        1.
    } else {
        0.
    }
}

pub fn fv(rate: f64, nper: f64, pmt: f64, pv: f64, pmt_at_begining: Option<bool>) -> f64 {
    if rate == 0.0 {
        return -(pv + pmt * nper);
    }

    let pmt_at_begining = convert_pmt_at_begining(pmt_at_begining);
    let factor = f64::powf(1.0 + rate, nper);

    -pv * factor - pmt * (1.0 + rate * pmt_at_begining) / rate * (factor - 1.0)
}

pub fn pv(rate: f64, nper: f64, pmt: f64, fv: Option<f64>, pmt_at_begining: Option<bool>) -> f64 {
    let fv = fv.unwrap_or(0.);

    if rate == 0.0 {
        return -(fv + pmt * nper);
    }

    let pmt_at_begining = convert_pmt_at_begining(pmt_at_begining);
    let exp = f64::powf(1. + rate, nper);
    let factor = (1. + rate * pmt_at_begining) * (exp - 1.) / rate;
    -(fv + pmt * factor) / exp
}

pub fn pmt(rate: f64, nper: f64, pv: f64, fv: Option<f64>, pmt_at_begining: Option<bool>) -> f64 {
    let fv = fv.unwrap_or(0.0);

    if rate == 0.0 {
        return -(fv + pv) / nper;
    }

    let pmt_at_begining = convert_pmt_at_begining(pmt_at_begining);

    let exp = f64::powf(1.0 + rate, nper);
    let factor = (1. + rate * pmt_at_begining) * (exp - 1.) / rate;

    -(fv + pv * exp) / factor
}

pub fn ipmt(
    rate: f64,
    per: f64,
    nper: f64,
    pv: f64,
    fv: Option<f64>,
    pmt_at_begining: Option<bool>,
) -> f64 {
    // payments before first period don't make any sense.
    if per < 1.0 {
        return f64::NAN;
    }

    // no interest if payment occurs at the beginning
    // of a period and this is the first period
    if per == 1.0 && pmt_at_begining.unwrap_or(false) {
        return 0.0;
    }

    let total_pmt = self::pmt(rate, nper, pv, fv, pmt_at_begining);
    let result = rate * self::fv(rate, per - 1.0, total_pmt, pv, pmt_at_begining);

    if pmt_at_begining.unwrap_or(false) {
        // if paying at the beginning we need to discount by one period.
        result / (1.0 + rate)
    } else {
        result
    }
}

pub fn ppmt(
    rate: f64,
    per: f64,
    nper: f64,
    pv: f64,
    fv: Option<f64>,
    pmt_at_begining: Option<bool>,
) -> f64 {
    self::pmt(rate, nper, pv, fv, pmt_at_begining)
        - self::ipmt(rate, per, nper, pv, fv, pmt_at_begining)
}

pub fn nper(rate: f64, pmt: f64, pv: f64, fv: Option<f64>, pmt_at_begining: Option<bool>) -> f64 {
    let fv = fv.unwrap_or(0.0);

    if rate == 0.0 {
        return -(fv + pv) / pmt;
    }

    let pmt_at_begining = convert_pmt_at_begining(pmt_at_begining);

    let z = pmt * (1. + rate * pmt_at_begining) / rate;
    f64::log10((-fv + z) / (pv + z)) / f64::log10(1. + rate)
}

pub fn rate(
    nper: f64,
    pmt: f64,
    pv: f64,
    fv: Option<f64>,
    pmt_at_begining: Option<bool>,
    guess: Option<f64>,
) -> f64 {
    let fv = fv.unwrap_or(0.0);
    newton_raphson_with_default_deriv(guess.unwrap_or(0.1), |rate| {
        fv - self::fv(rate, nper, pmt, pv, pmt_at_begining)
    })
}

// http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-NFV-function
pub fn nfv(rate: f64, nper: f64, amounts: &[f64]) -> f64 {
    let pv = self::npv(rate, amounts, Some(false));
    self::fv(rate, nper, 0.0, -pv, None)
}

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

fn npv_deriv(rate: f64, values: &[f64]) -> f64 {
    values.iter().enumerate().map(|(i, v)| -(i as f64) * v / (rate + 1.0).powf(i as f64 + 1.)).sum()
}

pub fn irr(values: &[f64], guess: Option<f64>) -> Result<f64, InvalidPaymentsError> {
    // must contain at least one positive and one negative value
    validate(values, None)?;

    let f = |rate| self::npv(rate, values, Some(true));
    let df = |rate| self::npv_deriv(rate, values);

    // IRR > 0 is preferred
    let rate = brentq(f, 0.0, 1e9, 1000);

    if rate.is_finite() && f(rate).abs() < 1e-3 {
        return Ok(rate);
    }

    Ok(find_root(guess.unwrap_or(0.1), &[(-0.99, 1.0, 0.01)], f, df))
}

pub fn mirr(
    values: &[f64],
    finance_rate: f64,
    reinvest_rate: f64,
) -> Result<f64, InvalidPaymentsError> {
    // must contain at least one positive and one negative value
    validate(values, None)?;

    let positive: f64 = powers(1. + reinvest_rate, values.len(), true)
        .iter()
        .zip(values.iter().rev())
        .filter(|(_r, &v)| v > 0.0)
        .map(|(r, v)| v * r)
        .sum();

    let negative: f64 = powers(1. + finance_rate, values.len(), true)
        .iter()
        .zip(values.iter())
        .filter(|(_r, &v)| v < 0.0)
        .map(|(&r, &v)| v / r)
        .sum();

    Ok((positive / -negative).powf(1.0 / (values.len() - 1) as f64) - 1.0)
}
