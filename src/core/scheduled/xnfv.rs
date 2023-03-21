use super::{year_fraction, DayCount};
use crate::core::{
    models::{validate, DateLike, InvalidPaymentsError},
    periodic::fv,
};

// http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-XFV-function
pub fn xfv(
    start_date: &DateLike,
    cash_flow_date: &DateLike,
    end_date: &DateLike,
    cash_flow_rate: f64,
    end_rate: f64,
    cash_flow: f64,
    day_count: Option<DayCount>,
) -> f64 {
    let dc = day_count.unwrap_or_default();
    let yf1 = year_fraction(start_date, end_date, dc);
    let yf2 = year_fraction(start_date, cash_flow_date, dc);
    let fv1 = self::fv(end_rate, yf1, 0., -1., false);
    let fv2 = self::fv(cash_flow_rate, yf2, 0., -1., false);
    fv1 / fv2 * cash_flow
}

// http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-XNFV-function
pub fn xnfv(
    rate: f64,
    dates: &[DateLike],
    amounts: &[f64],
    day_count: Option<DayCount>,
) -> Result<f64, InvalidPaymentsError> {
    validate(amounts, Some(dates))?;
    let d1 = dates.iter().min().unwrap();
    let d2 = dates.iter().max().unwrap();
    let periods = year_fraction(d1, d2, day_count.unwrap_or_default());
    let pv = super::xnpv(rate, dates, amounts, None)?;
    Ok(self::fv(rate, periods, 0., -pv, false))
}
