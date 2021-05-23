use super::optimize::powers;
use super::models::validate;

pub fn mirr(values: &[f64], finance_rate: f64, reinvest_rate: f64) -> f64 {
    // must contain at least one positive and one negative value or nan is returned
    // make it consistent with numpy_financial
    if validate(values, None).is_err() {
        return f64::NAN;
    }

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

    (positive / -negative).powf(1.0 / (values.len() - 1) as f64) - 1.0
}
