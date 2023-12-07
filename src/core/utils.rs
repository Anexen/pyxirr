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
    a.iter().any(|x| x.is_sign_negative()).then_some(-1.).unwrap_or(1.)
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
