use super::xnpv;
use crate::core::models::{validate, DateLike, InvalidPaymentsError};
use crate::core::periodic::{fv, npv};

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
