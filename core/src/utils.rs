pub(crate) fn fast_pow(a: f64, b: f64) -> f64 {
    // works only if a is positive
    (a.log2() * b).exp2()
}

pub(crate) fn scale(values: &[f64], factor: f64) -> Vec<f64> {
    values.iter().map(|v| v * factor).collect()
}

pub(crate) fn sum_pairwise_mul(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

pub(crate) fn pairwise_mul(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter().zip(b).map(|(x, y)| x * y).collect()
}

pub(crate) fn series_signum(a: &[f64]) -> f64 {
    // returns -1. if any item is negative, otherwise +1.
    if a.iter().any(|x| x.is_sign_negative()) {
        -1.
    } else {
        1.
    }
}

pub(crate) fn sum_negatives_positives(values: &[f64]) -> (f64, f64) {
    values.iter().fold((0., 0.), |acc, x| {
        if x.is_sign_negative() {
            (acc.0 + x, acc.1)
        } else {
            (acc.0, acc.1 + x)
        }
    })
}

pub(crate) fn is_a_good_rate<F>(rate: f64, f: F) -> bool
where
    F: Fn(f64) -> f64,
{
    rate.is_finite() && f(rate).abs() < 1e-3
}
