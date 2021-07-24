fn convert_pmt_at_begining(pmt_at_begining: Option<bool>) -> f64 {
    if pmt_at_begining.unwrap_or(false) {
        1.
    } else {
        0.
    }
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

pub fn nper(rate: f64, pmt: f64, pv: f64, fv: Option<f64>, pmt_at_begining: Option<bool>) -> f64 {
    let fv = fv.unwrap_or(0.0);

    if rate == 0.0 {
        return -(fv + pv) / pmt;
    }

    let pmt_at_begining = convert_pmt_at_begining(pmt_at_begining);

    let z = pmt * (1. + rate * pmt_at_begining) / rate;
    f64::log10((-fv + z) / (pv + z)) / f64::log10(1. + rate)
}
