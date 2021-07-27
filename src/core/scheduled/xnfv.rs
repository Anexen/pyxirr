use crate::core::models::{validate, DateLike, InvalidPaymentsError};
use crate::core::periodic::fv;

// http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-XFV-function
pub fn xfv(
    start_date: &DateLike,
    cash_flow_date: &DateLike,
    end_date: &DateLike,
    cash_flow_rate: f64,
    end_rate: f64,
    cash_flow: f64,
) -> f64 {
    self::fv(end_rate, (end_date - start_date) as f64 / 365., 0., -1., None)
        / self::fv(cash_flow_rate, (cash_flow_date - start_date) as f64 / 365., 0., -1., None)
        * cash_flow
}

// http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-XNFV-function
pub fn xnfv(rate: f64, dates: &[DateLike], amounts: &[f64]) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;
    let periods = (dates.iter().max().unwrap() - dates.iter().min().unwrap()) as f64 / 365.;
    let pv = super::xnpv(rate, dates, amounts)?;
    Ok(self::fv(rate, periods, 0., -pv, None))
}
