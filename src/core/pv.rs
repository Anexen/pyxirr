pub fn pv(rate: f64, nper: f64, pmt: f64, fv: Option<f64>, pmt_at_begining: Option<bool>) -> f64 {
    let fv = fv.unwrap_or(0.);

    if rate == 0.0 {
        return -(fv + pmt * nper);
    }

    let pmt_at_begining = if pmt_at_begining.unwrap_or(false) { 1. } else { 0. };
    let exp = f64::powf(1. + rate, nper);
    let factor = (1. + rate * pmt_at_begining) * (exp - 1.) / rate;
    -(fv + pmt * factor) / exp
}
