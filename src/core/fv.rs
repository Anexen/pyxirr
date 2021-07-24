use super::{models::validate, npv, xnpv, DateLike, InvalidPaymentsError};

pub fn fv(rate: f64, nper: f64, pmt: f64, pv: f64, pmt_at_begining: Option<bool>) -> f64 {
    if rate == 0.0 {
        return -(pv + pmt * nper);
    }

    let factor = f64::powf(1.0 + rate, nper);
    let pmt_at_begining = if pmt_at_begining.unwrap_or(false) { 1.0 } else { 0.0 };

    -pv * factor - pmt * (1.0 + rate * pmt_at_begining) / rate * (factor - 1.0)
}

pub fn xfv(
    rate: f64,
    nper: f64,
    amounts: &[f64],
    dates: Option<&[DateLike]>,
) -> Result<f64, InvalidPaymentsError> {
    let preset_value = if let Some(dates) = dates {
        validate(amounts, Some(dates))?;
        xnpv(rate, dates, amounts)?
    } else {
        npv(rate, amounts, Some(false))
    };

    Ok(fv(rate, nper, 0.0, -preset_value, None))
}
