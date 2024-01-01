use super::periodic::convert_pmt_at_beginning;
use crate::{broadcast_together, broadcasting::BroadcastingError};
use std::mem::MaybeUninit;

use ndarray::{ArrayD, ArrayViewD};

pub fn fv_vec(
    rate: &ArrayViewD<f64>,
    nper: &ArrayViewD<f64>,
    pmt: &ArrayViewD<f64>,
    pv: &ArrayViewD<f64>,
    pmt_at_beginning: &ArrayViewD<bool>,
) -> Result<ArrayD<f64>, BroadcastingError> {
    let pmt_at_beginning = pmt_at_beginning.mapv(convert_pmt_at_beginning);
    let (rate, nper, pmt, pv, pmt_at_beginning) =
        broadcast_together!(rate, nper, pmt, pv, pmt_at_beginning)?;

    let mut result = ArrayD::uninit(rate.shape());

    ndarray::Zip::from(&mut result)
        .and(rate)
        .and(nper)
        .and(pmt)
        .and(pv)
        .and(pmt_at_beginning)
        .for_each(|result, rate, nper, pmt, pv, pmt_at_beginning| {
            let value = if rate == &0.0 {
                -(pv + pmt * nper)
            } else {
                let f = (rate + 1.0).powf(*nper);
                -pv * f - pmt * (1.0 + rate * pmt_at_beginning) / rate * (f - 1.0)
            };
            *result = MaybeUninit::new(value);
        });

    Ok(unsafe { result.assume_init() })
}

pub fn pv_vec(
    rate: &ArrayViewD<f64>,
    nper: &ArrayViewD<f64>,
    pmt: &ArrayViewD<f64>,
    fv: &ArrayViewD<f64>,
    pmt_at_beginning: &ArrayViewD<bool>,
) -> Result<ArrayD<f64>, BroadcastingError> {
    let pmt_at_beginning = pmt_at_beginning.mapv(convert_pmt_at_beginning);
    let (rate, nper, pmt, fv, pmt_at_beginning) =
        broadcast_together!(rate, nper, pmt, fv, pmt_at_beginning)?;

    let mut result = ArrayD::uninit(rate.shape());

    ndarray::Zip::from(&mut result)
        .and(rate)
        .and(nper)
        .and(pmt)
        .and(fv)
        .and(pmt_at_beginning)
        .for_each(|result, rate, nper, pmt, fv, pmt_at_beginning| {
            let value = if rate == &0.0 {
                -(fv + pmt * nper)
            } else {
                let exp = (rate + 1.0).powf(*nper);
                let f = (1.0 + rate * pmt_at_beginning) * (exp - 1.0) / rate;
                -(fv + pmt * f) / exp
            };
            *result = MaybeUninit::new(value);
        });

    Ok(unsafe { result.assume_init() })
}

pub fn pmt_vec(
    rate: &ArrayViewD<f64>,
    nper: &ArrayViewD<f64>,
    pv: &ArrayViewD<f64>,
    fv: &ArrayViewD<f64>,
    pmt_at_beginning: &ArrayViewD<bool>,
) -> Result<ArrayD<f64>, BroadcastingError> {
    let pmt_at_beginning = pmt_at_beginning.mapv(convert_pmt_at_beginning);
    let (rate, nper, pv, fv, pmt_at_beginning) =
        broadcast_together!(rate, nper, pv, fv, pmt_at_beginning)?;

    let mut result = ArrayD::uninit(rate.shape());

    ndarray::Zip::from(&mut result)
        .and(rate)
        .and(nper)
        .and(pv)
        .and(fv)
        .and(pmt_at_beginning)
        .for_each(|result, rate, nper, pv, fv, pmt_at_beginning| {
            let value = if rate == &0.0 {
                -(fv + pv) / nper
            } else {
                let exp = (rate + 1.0).powf(*nper);
                let f = (1.0 + rate * pmt_at_beginning) * (exp - 1.0) / rate;
                -(fv + pv * exp) / f
            };
            *result = MaybeUninit::new(value);
        });

    Ok(unsafe { result.assume_init() })
}

pub fn ipmt_vec(
    rate: &ArrayViewD<f64>,
    per: &ArrayViewD<f64>,
    nper: &ArrayViewD<f64>,
    pv: &ArrayViewD<f64>,
    fv: &ArrayViewD<f64>,
    pmt_at_beginning: &ArrayViewD<bool>,
) -> Result<ArrayD<f64>, BroadcastingError> {
    let (rate, per, nper, pv, fv, pmt_at_beginning) =
        broadcast_together!(rate, per, nper, pv, fv, pmt_at_beginning)?;

    let f1 = ndarray::Zip::from(&rate).and(&per).map_collect(|rate, &per| (rate + 1.).powf(per));
    let f2 =
        ndarray::Zip::from(&rate).and(&nper).map_collect(|rate, &nper| (rate + 1.0).powf(nper));

    let mut result = (&rate * (&pv + &fv) * &f1 - &rate * (&rate + 1.0) * (&fv + &pv * &f2))
        / ((&rate + 1.0) * (&f2 - 1.0));

    for (ref idx, r) in result.indexed_iter_mut() {
        if rate[idx] == 0.0 {
            *r = 0.0;
        } else if per[idx] < 1.0 || per[idx] > nper[idx] {
            *r = f64::NAN;
        } else if per[idx] == 1.0 && pmt_at_beginning[idx] {
            *r = 0.0;
        } else if pmt_at_beginning[idx] {
            *r /= rate[idx] + 1.0;
        }
    }

    Ok(result)
}

pub fn ppmt_vec(
    rate: &ArrayViewD<f64>,
    per: &ArrayViewD<f64>,
    nper: &ArrayViewD<f64>,
    pv: &ArrayViewD<f64>,
    fv: &ArrayViewD<f64>,
    pmt_at_beginning: &ArrayViewD<bool>,
) -> Result<ArrayD<f64>, BroadcastingError> {
    let (rate, per, nper, pv, fv, pmt_at_beginning) =
        broadcast_together!(rate, per, nper, pv, fv, pmt_at_beginning)?;

    let when = pmt_at_beginning.mapv(convert_pmt_at_beginning);

    let f1 =
        ndarray::Zip::from(&rate).and(&per).map_collect(|rate, per| (rate + 1.).powf(per - 1.0));

    let f2 = ndarray::Zip::from(&rate)
        .and(&nper)
        .and(&when)
        .map_collect(|rate, nper, when| (rate + 1.0).powf(nper + when));

    let mut result = -&rate * (&fv + &pv) * &f1 / (&f2 - &rate * &when - 1.0);

    for (ref idx, r) in result.indexed_iter_mut() {
        if rate[idx] == 0.0 {
            *r = -(fv[idx] + pv[idx]) / nper[idx];
        } else if per[idx] < 1.0 || per[idx] > nper[idx] {
            *r = f64::NAN;
        }
    }

    Ok(result)
}

pub fn nper_vec(
    rate: &ArrayViewD<f64>,
    pmt: &ArrayViewD<f64>,
    pv: &ArrayViewD<f64>,
    fv: &ArrayViewD<f64>,
    pmt_at_beginning: &ArrayViewD<bool>,
) -> Result<ArrayD<f64>, BroadcastingError> {
    let pmt_at_beginning = pmt_at_beginning.mapv(convert_pmt_at_beginning);
    let (rate, pmt, pv, fv, pmt_at_beginning) =
        broadcast_together!(rate, pmt, pv, fv, pmt_at_beginning)?;

    let mut result = ArrayD::uninit(rate.shape());

    ndarray::Zip::from(&mut result)
        .and(rate)
        .and(pmt)
        .and(pv)
        .and(fv)
        .and(pmt_at_beginning)
        .for_each(|result, rate, pmt, pv, fv, pmt_at_beginning| {
            let value = if rate == &0.0 {
                -(fv + pv) / pmt
            } else {
                let z = pmt * (1. + rate * pmt_at_beginning) / rate;
                f64::log10((-fv + z) / (pv + z)) / f64::log10(1. + rate)
            };
            *result = MaybeUninit::new(value);
        });

    Ok(unsafe { result.assume_init() })
}

pub fn rate_vec(
    nper: &ArrayViewD<f64>,
    pmt: &ArrayViewD<f64>,
    pv: &ArrayViewD<f64>,
    fv: &ArrayViewD<f64>,
    pmt_at_beginning: &ArrayViewD<bool>,
    guess: Option<f64>,
) -> Result<ArrayD<f64>, BroadcastingError> {
    let pmt_at_beginning = pmt_at_beginning.mapv(convert_pmt_at_beginning);
    let (nper, pmt, pv, fv, pmt_at_beginning) =
        broadcast_together!(nper, pmt, pv, fv, pmt_at_beginning)?;

    let mut rn = ArrayD::from_elem(nper.shape(), guess.unwrap_or(0.1));
    let mut diff = ArrayD::ones(nper.shape());

    for _ in 0..100 {
        let rnp1 = &rn - _g_div_gp(&rn.view(), &nper, &pmt, &pv, &fv, &pmt_at_beginning.view());
        diff = &rnp1 - &rn;
        let all_close = diff.iter().all(|x| x.abs() < 1e-6);
        if all_close {
            return Ok(rnp1);
        }
        rn = rnp1;
    }

    rn.zip_mut_with(&diff, |x, &d| {
        if d > 1e-6 {
            *x = f64::NAN
        }
    });

    Ok(rn)
}

fn _g_div_gp(
    rate: &ArrayViewD<f64>,
    nper: &ArrayViewD<f64>,
    pmt: &ArrayViewD<f64>,
    pv: &ArrayViewD<f64>,
    fv: &ArrayViewD<f64>,
    when: &ArrayViewD<f64>,
) -> ArrayD<f64> {
    // Evaluate g(r_n)/g'(r_n), where g =
    // fv + pv*(1+rate)**nper + pmt*(1+rate*when)/rate * ((1+rate)**nper - 1)
    let mut t1 = rate + 1.0;
    t1.zip_mut_with(nper, |x, &nper| *x = x.powf(nper));

    let mut t2 = rate + 1.0;
    t2.zip_mut_with(nper, |x, &nper| *x = x.powf(nper - 1.0));

    let r2 = rate.mapv(|x| x.powf(2.0));

    let g = fv + &t1 * pv + pmt * (&t1 - 1.) * (rate * when + 1.) / rate;

    let gp = nper * &t2 * pv - pmt * (&t1 - 1.) * (rate * when + 1.) / r2
        + nper * pmt * &t2 * (rate * when + 1.) / rate
        + pmt * (&t1 - 1.0) * when / rate;

    g / gp
}
