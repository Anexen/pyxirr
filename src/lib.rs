use broadcasting::Arg;
use conversions::{fallible_float_or_none, float_or_none, PyDayCount};
use pyo3::{create_exception, exceptions, prelude::*, wrap_pyfunction};

mod broadcasting;
mod conversions;
mod core;

create_exception!(pyxirr, InvalidPaymentsError, exceptions::PyException);
create_exception!(pyxirr, BroadcastingError, exceptions::PyException);

impl From<core::InvalidPaymentsError> for PyErr {
    fn from(value: core::InvalidPaymentsError) -> Self {
        InvalidPaymentsError::new_err(value.to_string())
    }
}

impl From<broadcasting::BroadcastingError> for PyErr {
    fn from(value: broadcasting::BroadcastingError) -> Self {
        BroadcastingError::new_err(value.to_string())
    }
}

macro_rules! dispatch_vectorized {
    ($py:ident, ($($vars:ident),*), $non_vec:expr, $vec:expr ) => {
        {
            match ($($vars,)*) {
                ($(Arg::Scalar($vars),)*) => {
                    let result = $py.allow_threads(move || $non_vec);
                    Ok(Arg::Scalar(result))
                },
                ($($vars,)*) => {
                    let has_numpy_array = $(matches!($vars, Arg::NumpyArray(_)) || )* false;
                    let ($($vars,)*) = ($($vars.into_arrayd(),)*);
                    let ($($vars,)*) = ($($vars.view(),)*);
                    let result = $py.allow_threads(move || $vec);
                    let result = if has_numpy_array {
                        result.map(|r| Arg::from(numpy::ToPyArray::to_pyarray(&r, $py)))
                    } else {
                        result.map(|r| Arg::from(r))
                    };
                    result.map_err(|e| e.into())
                }
            }
        }
    };
}

/// Internal Rate of Return for a non-periodic cash flows.
#[pyfunction]
#[pyo3(signature = (dates, amounts=None, *, guess=0.1, silent=false, day_count=None))]
#[pyo3(text_signature = "(dates, amounts=None, *, guess=0.1, silent=False, day_count=None)")]
fn xirr(
    py: Python,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    guess: Option<f64>,
    silent: Option<bool>,
    day_count: Option<PyDayCount>,
) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let day_count = day_count.map(|x| x.try_into()).transpose()?;

    py.allow_threads(move || {
        let result = core::xirr(&dates, &amounts, guess, day_count);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Net Present Value for a non-periodic cash flows.
#[pyfunction]
#[pyo3(signature = (rate, dates, amounts=None, *, silent=false, day_count=None))]
#[pyo3(text_signature = "(rate, dates, amounts=None, *, silent=False, day_count=None)")]
fn xnpv(
    py: Python,
    rate: f64,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    silent: Option<bool>,
    day_count: Option<PyDayCount>,
) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let day_count = day_count.map(|x| x.try_into()).transpose()?;

    py.allow_threads(move || {
        let result = core::xnpv(rate, &dates, &amounts, day_count);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Internal Rate of Return
#[pyfunction]
#[pyo3(signature = (amounts, *, guess=0.1, silent=false))]
#[pyo3(text_signature = "(amounts, *, guess=0.1, silent=False)")]
fn irr(
    py: Python,
    amounts: &PyAny,
    guess: Option<f64>,
    silent: Option<bool>,
) -> PyResult<Option<f64>> {
    let amounts = conversions::extract_amount_series(amounts)?;
    py.allow_threads(move || {
        let result = core::irr(&amounts, guess);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Net Present Value.
/// NPV is calculated using the following formula:
/// sum([values[i]/(1 + rate)**i for i in range(len(values))])
/// There is a difference between numpy NPV and excel NPV.
/// By default, npv function starts from zero (numpy compatible),
/// but you can call it with `start_from_zero=False` parameter to make it Excel compatible.
#[pyfunction]
#[pyo3(signature = (rate, amounts, *, start_from_zero=true))]
#[pyo3(text_signature = "(rate, amounts, *, start_from_zero = True)")]
fn npv(
    py: Python,
    rate: f64,
    amounts: &PyAny,
    start_from_zero: Option<bool>,
) -> PyResult<Option<f64>> {
    let payments = conversions::extract_amount_series(amounts)?;
    py.allow_threads(move || {
        let result = core::npv(rate, &payments, start_from_zero);
        Ok(float_or_none(result))
    })
}

/// Future Value.
#[pyfunction]
#[pyo3(signature = (rate, nper, pmt, pv, *, pmt_at_beginning=Arg::Scalar(false)))]
#[pyo3(text_signature = "(rate, nper, pmt, pv, *, pmt_at_beginning=False)")]
fn fv<'a>(
    py: Python<'a>,
    rate: Arg<'a, f64>,
    nper: Arg<'a, f64>,
    pmt: Arg<'a, f64>,
    pv: Arg<'a, f64>,
    pmt_at_beginning: Arg<'a, bool>,
) -> PyResult<Arg<'a, f64>> {
    dispatch_vectorized!(
        py,
        (rate, nper, pmt, pv, pmt_at_beginning),
        core::fv(rate, nper, pmt, pv, pmt_at_beginning),
        core::fv_vec(&rate, &nper, &pmt, &pv, &pmt_at_beginning)
    )
}

/// Net Future Value.
#[pyfunction]
#[pyo3(text_signature = "(rate, nper, amounts)")]
fn nfv(py: Python, rate: f64, nper: f64, amounts: &PyAny) -> PyResult<Option<f64>> {
    let amounts = conversions::extract_amount_series(amounts)?;
    py.allow_threads(move || Ok(float_or_none(core::nfv(rate, nper, &amounts))))
}

/// Extended Future Value.
/// Future value of a cash flow between two dates.
#[pyfunction]
#[pyo3(
    signature = (start_date, cash_flow_date, end_date, cash_flow_rate, end_rate, cash_flow, *, day_count=None),
    text_signature = "(start_date, cash_flow_date, end_date, cash_flow_rate, end_rate, cash_flow, * day_count=None)"
)]
#[allow(clippy::too_many_arguments)]
fn xfv(
    py: Python,
    start_date: core::DateLike,
    cash_flow_date: core::DateLike,
    end_date: core::DateLike,
    cash_flow_rate: f64,
    end_rate: f64,
    cash_flow: f64,
    day_count: Option<PyDayCount>,
) -> PyResult<Option<f64>> {
    let day_count = day_count.map(|x| x.try_into()).transpose()?;

    py.allow_threads(move || {
        Ok(float_or_none(core::xfv(
            &start_date,
            &cash_flow_date,
            &end_date,
            cash_flow_rate,
            end_rate,
            cash_flow,
            day_count,
        )))
    })
}

/// Net future value of a series of irregular cash flows
#[pyfunction]
#[pyo3(signature = (rate, dates, amounts=None, *, silent=false, day_count=None))]
#[pyo3(text_signature = "(rate, dates, amounts=None, *, silent=False, day_count=None)")]
fn xnfv(
    py: Python,
    rate: f64,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    silent: Option<bool>,
    day_count: Option<PyDayCount>,
) -> PyResult<Option<f64>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let day_count = day_count.map(|x| x.try_into()).transpose()?;
    py.allow_threads(move || {
        let result = core::xnfv(rate, &dates, &amounts, day_count);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Present Value
#[pyfunction]
#[pyo3(signature = (rate, nper, pmt, fv=Arg::Scalar(0.0), *, pmt_at_beginning=Arg::Scalar(false)))]
#[pyo3(text_signature = "(rate, nper, pmt, fv=0, *, pmt_at_beginning=False)")]
fn pv<'a>(
    py: Python<'a>,
    rate: Arg<'a, f64>,
    nper: Arg<'a, f64>,
    pmt: Arg<'a, f64>,
    fv: Arg<'a, f64>,
    pmt_at_beginning: Arg<'a, bool>,
) -> PyResult<Arg<'a, f64>> {
    dispatch_vectorized!(
        py,
        (rate, nper, pmt, fv, pmt_at_beginning),
        core::pv(rate, nper, pmt, fv, pmt_at_beginning),
        core::pv_vec(&rate, &nper, &pmt, &fv, &pmt_at_beginning)
    )
}

/// Modified Internal Rate of Return.
#[pyfunction]
#[pyo3(signature = (amounts, finance_rate, reinvest_rate, *, silent=false))]
#[pyo3(text_signature = "(amounts, finance_rate, reinvest_rate, *, silent=False)")]
fn mirr(
    py: Python,
    amounts: &PyAny,
    finance_rate: f64,
    reinvest_rate: f64,
    silent: Option<bool>,
) -> PyResult<Option<f64>> {
    let values = conversions::extract_amount_series(amounts)?;
    py.allow_threads(move || {
        let result = core::mirr(&values, finance_rate, reinvest_rate);
        fallible_float_or_none(result, silent.unwrap_or(false))
    })
}

/// Compute the payment against loan principal plus interest.
#[pyfunction]
#[pyo3(signature = (rate, nper, pv, fv=Arg::Scalar(0.0), *, pmt_at_beginning=Arg::Scalar(false)))]
#[pyo3(text_signature = "(rate, nper, pv, fv=0, *, pmt_at_beginning=False)")]
fn pmt<'a>(
    py: Python<'a>,
    rate: Arg<'a, f64>,
    nper: Arg<'a, f64>,
    pv: Arg<'a, f64>,
    fv: Arg<'a, f64>,
    pmt_at_beginning: Arg<'a, bool>,
) -> PyResult<Arg<'a, f64>> {
    dispatch_vectorized!(
        py,
        (rate, nper, pv, fv, pmt_at_beginning),
        core::pmt(rate, nper, pv, fv, pmt_at_beginning),
        core::pmt_vec(&rate, &nper, &pv, &fv, &pmt_at_beginning)
    )
}

/// Compute the interest portion of a payment.
#[pyfunction]
#[pyo3(signature = (rate, per, nper, pv, fv=Arg::Scalar(0.0), *, pmt_at_beginning=Arg::Scalar(false)))]
#[pyo3(text_signature = "(rate, per, nper, pv, fv=0, *, pmt_at_beginning=False)")]
fn ipmt<'a>(
    py: Python<'a>,
    rate: Arg<'a, f64>,
    per: Arg<'a, f64>,
    nper: Arg<'a, f64>,
    pv: Arg<'a, f64>,
    fv: Arg<'a, f64>,
    pmt_at_beginning: Arg<'a, bool>,
) -> PyResult<Arg<'a, f64>> {
    dispatch_vectorized!(
        py,
        (rate, per, nper, pv, fv, pmt_at_beginning),
        core::ipmt(rate, per, nper, pv, fv, pmt_at_beginning),
        core::ipmt_vec(&rate, &per, &nper, &pv, &fv, &pmt_at_beginning)
    )
}

/// Compute the payment against loan principal.
#[pyfunction]
#[pyo3(signature = (rate, per, nper, pv, fv=Arg::Scalar(0.0), *, pmt_at_beginning=Arg::Scalar(false)))]
#[pyo3(text_signature = "(rate, per, nper, pv, fv=0, *, pmt_at_beginning=False)")]
fn ppmt<'a>(
    py: Python<'a>,
    rate: Arg<'a, f64>,
    per: Arg<'a, f64>,
    nper: Arg<'a, f64>,
    pv: Arg<'a, f64>,
    fv: Arg<'a, f64>,
    pmt_at_beginning: Arg<'a, bool>,
) -> PyResult<Arg<'a, f64>> {
    dispatch_vectorized!(
        py,
        (rate, per, nper, pv, fv, pmt_at_beginning),
        core::ppmt(rate, per, nper, pv, fv, pmt_at_beginning),
        core::ppmt_vec(&rate, &per, &nper, &pv, &fv, &pmt_at_beginning)
    )
}

/// Compute the number of periodic payments.
#[pyfunction]
#[pyo3(signature = (rate, pmt, pv, fv=Arg::Scalar(0.0), *, pmt_at_beginning=Arg::Scalar(false)))]
#[pyo3(text_signature = "(rate, pmt, pv, fv=0, *, pmt_at_beginning=False)")]
fn nper<'a>(
    py: Python<'a>,
    rate: Arg<'a, f64>,
    pmt: Arg<'a, f64>,
    pv: Arg<'a, f64>,
    fv: Arg<'a, f64>,
    pmt_at_beginning: Arg<'a, bool>,
) -> PyResult<Arg<'a, f64>> {
    dispatch_vectorized!(
        py,
        (rate, pmt, pv, fv, pmt_at_beginning),
        core::nper(rate, pmt, pv, fv, pmt_at_beginning),
        core::nper_vec(&rate, &pmt, &pv, &fv, &pmt_at_beginning)
    )
}

/// Compute the number of periodic payments.
#[pyfunction]
#[pyo3(signature = (nper, pmt, pv, fv=Arg::Scalar(0.0), *, pmt_at_beginning=Arg::Scalar(false), guess=0.1))]
#[pyo3(text_signature = "(nper, pmt, pv, fv=0, *, pmt_at_beginning=False, guess=0.1)")]
fn rate<'a>(
    py: Python<'a>,
    nper: Arg<'a, f64>,
    pmt: Arg<'a, f64>,
    pv: Arg<'a, f64>,
    fv: Arg<'a, f64>,
    pmt_at_beginning: Arg<'a, bool>,
    guess: Option<f64>,
) -> PyResult<Arg<'a, f64>> {
    dispatch_vectorized!(
        py,
        (nper, pmt, pv, fv, pmt_at_beginning),
        core::rate(nper, pmt, pv, fv, pmt_at_beginning, guess),
        core::rate_vec(&nper, &pmt, &pv, &fv, &pmt_at_beginning, guess)
    )
}

#[pyfunction]
#[pyo3(signature = (rate, nper, pv, start_period, end_period, *, pmt_at_beginning=false))]
fn cumprinc(
    py: Python,
    rate: f64,
    nper: f64,
    pv: f64,
    start_period: f64,
    end_period: f64,
    pmt_at_beginning: bool,
) -> Option<f64> {
    // https://wiki.documentfoundation.org/Documentation/Calc_Functions/CUMPRINC
    let result = py.allow_threads(move || {
        (start_period.trunc() as u64..=end_period.trunc() as u64)
            .map(|per| core::ppmt(rate, per as f64, nper, pv, 0.0, pmt_at_beginning))
            .sum()
    });

    float_or_none(result)
}

#[pyfunction]
#[pyo3(signature = (rate, nper, pv, start_period, end_period, *, pmt_at_beginning=false))]
fn cumipmt(
    py: Python,
    rate: f64,
    nper: f64,
    pv: f64,
    start_period: f64,
    end_period: f64,
    pmt_at_beginning: bool,
) -> Option<f64> {
    // https://wiki.documentfoundation.org/Documentation/Calc_Functions/CUMIPMT
    let result = py.allow_threads(move || {
        (start_period.trunc() as u64..=end_period.trunc() as u64)
            .map(|per| core::ipmt(rate, per as f64, nper, pv, 0.0, pmt_at_beginning))
            .sum()
    });

    float_or_none(result)
}

#[pyfunction]
fn year_fraction(d1: core::DateLike, d2: core::DateLike, day_count: PyDayCount) -> PyResult<f64> {
    Ok(core::year_fraction(&d1, &d2, day_count.try_into()?))
}

#[pyfunction]
fn days_between(d1: core::DateLike, d2: core::DateLike, day_count: PyDayCount) -> PyResult<i32> {
    Ok(core::days_between(&d1, &d2, day_count.try_into()?))
}

#[pymodule]
pub fn pyxirr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    m.add_class::<core::DayCount>()?;
    m.add_function(wrap_pyfunction!(year_fraction, m)?)?;
    m.add_function(wrap_pyfunction!(days_between, m)?)?;

    m.add_function(wrap_pyfunction!(pmt, m)?)?;
    m.add_function(wrap_pyfunction!(ipmt, m)?)?;
    m.add_function(wrap_pyfunction!(cumipmt, m)?)?;
    m.add_function(wrap_pyfunction!(ppmt, m)?)?;
    m.add_function(wrap_pyfunction!(cumprinc, m)?)?;
    m.add_function(wrap_pyfunction!(nper, m)?)?;
    m.add_function(wrap_pyfunction!(rate, m)?)?;
    m.add_function(wrap_pyfunction!(fv, m)?)?;
    m.add_function(wrap_pyfunction!(nfv, m)?)?;
    m.add_function(wrap_pyfunction!(xfv, m)?)?;
    m.add_function(wrap_pyfunction!(xnfv, m)?)?;
    m.add_function(wrap_pyfunction!(pv, m)?)?;
    m.add_function(wrap_pyfunction!(npv, m)?)?;
    m.add_function(wrap_pyfunction!(xnpv, m)?)?;
    m.add_function(wrap_pyfunction!(irr, m)?)?;
    m.add_function(wrap_pyfunction!(mirr, m)?)?;
    m.add_function(wrap_pyfunction!(xirr, m)?)?;

    m.add("InvalidPaymentsError", py.get_type::<InvalidPaymentsError>())?;
    m.add("BroadcastingError", py.get_type::<BroadcastingError>())?;

    Ok(())
}
