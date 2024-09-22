use std::{iter::successors};

use super::{
    models::{validate, InvalidPaymentsError},
    optimize::{brentq, brentq_grid_search, newton_raphson, newton_raphson_with_default_deriv},
    utils,
};

// pre calculating powers for performance
pub fn powers(base: f64, n: usize, start_from_zero: bool) -> Vec<f64> {
    let (start, n) = if start_from_zero {
        (1.0, n + 1)
    } else {
        (base, n)
    };
    successors(Some(start), |x| Some(x * base)).take(n).collect()
}

fn convert_pmt_at_beginning(pmt_at_beginning: bool) -> f64 {
    if pmt_at_beginning {
        1.
    } else {
        0.
    }
}

pub fn fv(rate: f64, nper: f64, pmt: f64, pv: f64, pmt_at_beginning: bool) -> f64 {
    if rate == 0.0 {
        return -(pv + pmt * nper);
    }

    let pmt_at_beginning = convert_pmt_at_beginning(pmt_at_beginning);
    let factor = f64::powf(1.0 + rate, nper);

    -pv * factor - pmt * (1.0 + rate * pmt_at_beginning) / rate * (factor - 1.0)
}

pub fn pv(rate: f64, nper: f64, pmt: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    if rate == 0.0 {
        return -(fv + pmt * nper);
    }

    let pmt_at_beginning = convert_pmt_at_beginning(pmt_at_beginning);
    let exp = f64::powf(1. + rate, nper);
    let factor = (1. + rate * pmt_at_beginning) * (exp - 1.) / rate;
    -(fv + pmt * factor) / exp
}

pub fn pmt(rate: f64, nper: f64, pv: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    if rate == 0.0 {
        return -(fv + pv) / nper;
    }

    let pmt_at_beginning = convert_pmt_at_beginning(pmt_at_beginning);

    let exp = f64::powf(1.0 + rate, nper);
    let factor = (1. + rate * pmt_at_beginning) * (exp - 1.) / rate;

    -(fv + pv * exp) / factor
}

pub fn ipmt(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    // let total_pmt = self::pmt(rate, nper, pv, fv, pmt_at_beginning);
    // let result = rate * self::fv(rate, per - 1.0, total_pmt, pv, pmt_at_beginning);
    //
    // simplify r*(-P*(1+r)**(p-1)-(-(F+P*(1+r)**n)*r/((1+r*t)*((1+r)**n-1)))*(1+r*t)/r*((1+r)**(p-1)-1))

    // payments before first period don't make any sense.
    if per < 1.0 || per > nper {
        return f64::NAN;
    }

    // no interest if payment occurs at the beginning
    // of a period and this is the first period
    if per == 1.0 && pmt_at_beginning {
        return 0.0;
    }

    // no interest if rate == 0
    if rate == 0.0 {
        return 0.0;
    }

    let f1 = (rate + 1.0).powf(per);
    let f2 = (rate + 1.0).powf(nper);

    let result = (rate * (pv + fv) * f1 - rate * (rate + 1.0) * (fv + pv * f2))
        / ((rate + 1.0) * (f2 - 1.0));

    if pmt_at_beginning {
        // if paying at the beginning we need to discount by one period.
        result / (1.0 + rate)
    } else {
        result
    }
}

pub fn ppmt(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    // assuming type = 1 if pmt_at_beginning else 0
    // assuming P=pv;F=fv;r=rate;n=nper;p=per;t=type, type in {1;0}
    // ppmt = fv(r,p-1,pmt(r,n,P,F,t),P,t) - fv(r,p,pmt(r,n,P,F,t),P,t)
    // after substitution:
    // simplify (-P*(1+r)^(p-1)-(-(F+P*(1+r)^n)*r/((1+r)^n-1)/(1+r*t))*(1+r*t)/r*((1+r)^(p-1)-1)) - (-P*(1+r)^p-(-(F+P*(1+r)^n)*r/((1+r)^n-1)/(1+r*t))*(1+r*t)/r*((1+r)^p-1))
    // shorter formula: -r*(F+P)*(r+1)^(per-1)/((r+1)^n - 1)
    // type correction: result /= r + 1 if type = 1
    // denominator => 1/((r+1)^p-1) => 1/(((r+1)^p-1)*(r+1)) =>
    // => 1/((r+1)^(p+1) - (r+1)) => 1/((r+1)^(p+t) -r*t + 1)
    //
    // if rate == 0:
    // simplify (-P-(-(F+P)/n) *(p-1) - (-P-(-(F+P)/n)*p))
    // shorter: -(F + P) / n;

    if per < 1.0 || per > nper {
        return f64::NAN;
    }

    if rate == 0.0 {
        return -(fv + pv) / nper;
    }

    let when = convert_pmt_at_beginning(pmt_at_beginning);
    -rate * (fv + pv) * (rate + 1.).powf(per - 1.)
        / ((rate + 1.).powf(nper + when) - rate * when - 1.)
}

pub fn nper(rate: f64, pmt: f64, pv: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    if rate == 0.0 {
        return -(fv + pv) / pmt;
    }

    let pmt_at_beginning = convert_pmt_at_beginning(pmt_at_beginning);

    let z = pmt * (1. + rate * pmt_at_beginning) / rate;
    f64::log10((-fv + z) / (pv + z)) / f64::log10(1. + rate)
}

pub fn rate(
    nper: f64,
    pmt: f64,
    pv: f64,
    fv: f64,
    pmt_at_beginning: bool,
    guess: Option<f64>,
) -> f64 {
    newton_raphson_with_default_deriv(guess.unwrap_or(0.1), |rate| {
        fv - self::fv(rate, nper, pmt, pv, pmt_at_beginning)
    })
}

// http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-NFV-function
pub fn nfv(rate: f64, nper: f64, amounts: &[f64]) -> f64 {
    let pv = self::npv(rate, amounts, Some(false));
    self::fv(rate, nper, 0.0, -pv, false)
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
    values
        .iter()
        .enumerate()
        .map(|(i, v)| -(i as f64) * v * utils::fast_pow(rate + 1.0, -(i as f64 + 1.0)))
        .sum()
}

pub fn irr(values: &[f64], guess: Option<f64>) -> Result<f64, InvalidPaymentsError> {
    // must contain at least one positive and one negative value
    validate(values, None)?;

    if values.len() == 2 {
        return Ok(irr_analytical_2(values));
    }

    if values.len() == 3 {
        return Ok(irr_analytical_3(values));
    }

    let f = |rate| {
        if rate <= -1.0 {
            // bound newton_raphson
            return f64::INFINITY;
        }
        self::npv(rate, values, Some(true))
    };
    let df = |rate| self::npv_deriv(rate, values);

    let guess = match guess {
        Some(g) => g,
        None => {
            let (outflows, inflows) = utils::sum_negatives_positives(values);
            inflows / -outflows - 1.0
        }
    };

    let rate = newton_raphson(guess, &f, &df);

    if utils::is_a_good_rate(rate, f) {
        return Ok(rate);
    }

    let rate = brentq(&f, -0.999999999999999, 100., 100);

    if utils::is_a_good_rate(rate, f) {
        return Ok(rate);
    }

    // strategy: closest to zero
    // let breakpoints: &[f64] = &[0.0, 0.25, -0.25, 0.5, -0.5, 1.0, -0.9, -0.99999999999999, 1e9];
    // strategy: pessimistic
    let breakpoints: &[f64] = &[-0.99999999999999, -0.75, -0.5, -0.25, 0., 0.25, 0.5, 1.0, 1e6];
    let rate = brentq_grid_search(&[breakpoints], &f).next();

    Ok(rate.unwrap_or(f64::NAN))
}

fn irr_analytical_2(values: &[f64]) -> f64 {
    // cf[0]/(1+r)^0 + cf[1]/(1+r)^1 = 0  => multiply by (1 + r)
    // cf[0]*(1+r) + cf[1] = 0  => divide by cf[0] and move tho the right
    // lets x = 1+r, a = cf[0], b = cf[1]
    // solve a*x + b = 0
    // x = -b/a, r = x - 1
    -values[1] / values[0] - 1.0
}

fn irr_analytical_3(values: &[f64]) -> f64 {
    // cf[0]/(1+r)^0 + cf[1]/(1+r)^1 + cf[2]/(1+r)^2 = 0  => multiply by (1+r)^2
    // cf[0]*(1+r)^2 + cf[1]*(1+r) + cf[2] = 0  => quadratic equation
    // lets x = 1+r, a = cf[0], b = cf[1], c = cf[2]
    // solve a*x^2 + b*x + c = 0
    // x = (-b ± sqrt(b^2-4ac))/2a, a != 0

    let (a, b, c) = (values[0], values[1], values[2]);

    let x = if a == 0. {
        // 0*x^2 + bx + c = 0 =>
        // x = -c/b
        -c / b
    } else {
        let d = b.powf(2.) - 4. * a * c; // discriminant
        if d < 0.0 {
            // no real solutions
            f64::NAN
        } else if d == 0.0 {
            // exactly one solution
            -b / (2. * a)
        } else {
            let mut x1 = (-b + d.sqrt()) / (2. * a);
            let mut x2 = (-b - d.sqrt()) / (2. * a);
            // since x = 1 + r => r = x - 1,
            // negative x doesn't make sense (rate will be < -1)
            // use the first non negative value to be conservative
            if x1 > x2 {
                // make x2 always > x1
                std::mem::swap(&mut x1, &mut x2);
            }
            if x1 > 0.0 {
                x1
            } else if x2 > 0.0 {
                x2
            } else if x1 == 0.0 || x2 == 0.0 {
                0.0
            } else {
                f64::NAN
            }
        }
    };
    // x = 1 + r => r = x - 1
    x - 1.
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
