use broadcasting::Arg;
use conversions::{fallible_float_or_none, float_or_none, AmountArray, PyDayCount};
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
    (infallible $py:ident, ($($vars:ident),*), $non_vec:expr, $vec:expr ) => {
        {
            match ($($vars,)*) {
                ($(Arg::Scalar($vars),)*) => {
                    let result = $py.allow_threads(move || $non_vec);
                    Arg::Scalar(result)
                },
                ($($vars,)*) => {
                    let has_numpy_array = $(matches!($vars, Arg::NumpyArray(_)) || )* false;
                    let ($($vars,)*) = ($($vars.into_arrayd(),)*);
                    let ($($vars,)*) = ($($vars.view(),)*);
                    let result = $py.allow_threads(move || $vec);
                    if has_numpy_array {
                        Arg::from(numpy::ToPyArray::to_pyarray(&result, $py))
                    } else {
                        Arg::from(result)
                    }
                }
            }
        }
    };
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
#[pyo3(signature = (dates, amounts=None, *, guess=None, silent=false, day_count=None))]
#[pyo3(text_signature = "(dates, amounts=None, *, guess=None, silent=False, day_count=None)")]
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
fn xnpv<'a>(
    py: Python<'a>,
    rate: Arg<f64, 'a>,
    dates: &PyAny,
    amounts: Option<&PyAny>,
    silent: Option<bool>,
    day_count: Option<PyDayCount>,
) -> PyResult<Option<Arg<f64, 'a>>> {
    let (dates, amounts) = conversions::extract_payments(dates, amounts)?;
    let day_count = day_count.map(|x| x.try_into()).transpose()?;
    let silent = silent.unwrap_or(false);

    match rate {
        Arg::Scalar(rate) => {
            let result = py.allow_threads(move || core::xnpv(rate, &dates, &amounts, day_count));
            match result {
                Ok(rate) if rate.is_finite() => Ok(Some(Arg::Scalar(rate))),
                Ok(_) => Ok(None),
                Err(e) => {
                    if silent {
                        Ok(None)
                    } else {
                        Err(e.into())
                    }
                }
            }
        }
        rate => {
            let has_numpy_array = matches!(rate, Arg::NumpyArray(_)) || false;
            let rate = rate.into_arrayd();
            let result = py.allow_threads(move || {
                let r = rate.mapv(|r| core::xnpv(r, &dates, &amounts, day_count));

                if silent {
                    Ok(r.mapv(|e| e.unwrap_or(f64::NAN)))
                } else {
                    let err = r.iter().filter(|e| e.is_err()).next();
                    if let Some(err) = err {
                        Err(err.clone().unwrap_err())
                    } else {
                        Ok(r.mapv(|v| v.unwrap()))
                    }
                }
            });

            let result = if has_numpy_array {
                result.map(|r| Arg::from(numpy::ToPyArray::to_pyarray(&r, py)))
            } else {
                result.map(|r| Arg::from(r))
            };
            result.map(|v| Some(v)).map_err(|e| e.into())
        }
    }
}

/// Internal Rate of Return
#[pyfunction]
#[pyo3(signature = (amounts, *, guess=0.1, silent=false))]
#[pyo3(text_signature = "(amounts, *, guess=0.1, silent=False)")]
fn irr(
    py: Python,
    amounts: AmountArray,
    guess: Option<f64>,
    silent: Option<bool>,
) -> PyResult<Option<f64>> {
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
fn npv<'a>(
    py: Python<'a>,
    rate: Arg<f64, 'a>,
    amounts: AmountArray,
    start_from_zero: Option<bool>,
) -> Arg<f64, 'a> {
    match rate {
        Arg::Scalar(rate) => {
            let result = py.allow_threads(move || core::npv(rate, &amounts, start_from_zero));
            Arg::Scalar(result)
        }
        Arg::Array(rates) => {
            let result =
                py.allow_threads(move || rates.mapv(|r| core::npv(r, &amounts, start_from_zero)));
            Arg::from(result)
        }
        Arg::NumpyArray(rates) => {
            let view = rates.readonly();
            let rates = view.as_array();
            let result =
                py.allow_threads(move || rates.mapv(|r| core::npv(r, &amounts, start_from_zero)));
            Arg::from(numpy::ToPyArray::to_pyarray(&result, py))
        }
    }
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
fn nfv(py: Python, rate: f64, nper: f64, amounts: AmountArray) -> PyResult<Option<f64>> {
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
        let result = core::xfv(
            &start_date,
            &cash_flow_date,
            &end_date,
            cash_flow_rate,
            end_rate,
            cash_flow,
            day_count,
        );
        Ok(float_or_none(result))
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
    amounts: AmountArray,
    finance_rate: f64,
    reinvest_rate: f64,
    silent: Option<bool>,
) -> PyResult<Option<f64>> {
    py.allow_threads(move || {
        let result = core::mirr(&amounts, finance_rate, reinvest_rate);
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

#[pyfunction]
/// Conventional cash flow is a series of inward and outward cash flows over time in which there is
/// only one change in the cash flow direction. A conventional cash flow for a project or
/// investment is typically structured as an initial outlay or outflow, followed by a number of
/// inflows over a period of time.
fn is_conventional_cash_flow(cf: AmountArray) -> bool {
    core::sign_changes(&cf) == 1
}

#[pyfunction]
fn zero_crossing_points(cf: AmountArray) -> Vec<usize> {
    core::zero_crossing_points(&cf)
}

mod pe {
    use crate::{
        conversions::{fallible_float_or_none, AmountArray},
        core::private_equity,
    };
    use pyo3::prelude::*;

    pub fn module(_py: Python, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(dpi, m)?)?;
        m.add_function(wrap_pyfunction!(dpi_2, m)?)?;
        m.add_function(wrap_pyfunction!(rvpi, m)?)?;
        m.add_function(wrap_pyfunction!(tvpi, m)?)?;
        m.add_function(wrap_pyfunction!(tvpi_2, m)?)?;
        m.add_function(wrap_pyfunction!(moic, m)?)?;
        m.add_function(wrap_pyfunction!(moic_2, m)?)?;
        m.add_function(wrap_pyfunction!(ks_pme, m)?)?;
        m.add_function(wrap_pyfunction!(ks_pme_2, m)?)?;
        m.add_function(wrap_pyfunction!(ks_pme_flows, m)?)?;
        m.add_function(wrap_pyfunction!(ks_pme_flows_2, m)?)?;
        m.add_function(wrap_pyfunction!(m_pme, m)?)?;
        m.add_function(wrap_pyfunction!(m_pme_2, m)?)?;
        m.add_function(wrap_pyfunction!(pme_plus, m)?)?;
        m.add_function(wrap_pyfunction!(pme_plus_2, m)?)?;
        m.add_function(wrap_pyfunction!(pme_plus_flows, m)?)?;
        m.add_function(wrap_pyfunction!(pme_plus_flows_2, m)?)?;
        m.add_function(wrap_pyfunction!(pme_plus_lambda, m)?)?;
        m.add_function(wrap_pyfunction!(pme_plus_lambda_2, m)?)?;
        m.add_function(wrap_pyfunction!(ln_pme_nav, m)?)?;
        m.add_function(wrap_pyfunction!(ln_pme_nav_2, m)?)?;
        m.add_function(wrap_pyfunction!(ln_pme, m)?)?;
        m.add_function(wrap_pyfunction!(ln_pme_2, m)?)?;
        m.add_function(wrap_pyfunction!(direct_alpha, m)?)?;
        m.add_function(wrap_pyfunction!(direct_alpha_2, m)?)?;

        Ok(())
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/dpi.md")]
    fn dpi(py: Python, amounts: AmountArray) -> PyResult<f64> {
        py.allow_threads(move || Ok(private_equity::dpi(&amounts)?))
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/dpi.md")]
    fn dpi_2(py: Python, contributions: AmountArray, distributions: AmountArray) -> PyResult<f64> {
        py.allow_threads(move || Ok(private_equity::dpi_2(&contributions, &distributions)?))
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/rvpi.md")]
    fn rvpi(py: Python, contributions: AmountArray, nav: f64) -> PyResult<f64> {
        py.allow_threads(move || Ok(private_equity::rvpi(&contributions, nav)?))
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/tvpi.md")]
    pub fn tvpi(py: Python, amounts: AmountArray, nav: Option<f64>) -> PyResult<f64> {
        py.allow_threads(move || Ok(private_equity::tvpi(&amounts, nav.unwrap_or(0.0))?))
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/tvpi.md")]
    pub fn tvpi_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<f64> {
        py.allow_threads(move || {
            Ok(private_equity::tvpi_2(&contributions, &distributions, nav.unwrap_or(0.0))?)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/moic.md")]
    pub fn moic(py: Python, amounts: AmountArray, nav: Option<f64>) -> PyResult<f64> {
        py.allow_threads(move || Ok(private_equity::moic(&amounts, nav.unwrap_or(0.0))?))
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/moic.md")]
    pub fn moic_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<f64> {
        py.allow_threads(move || {
            Ok(private_equity::moic_2(&contributions, &distributions, nav.unwrap_or(0.0))?)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/ks_pme.md")]
    fn ks_pme(
        py: Python,
        amounts: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<f64> {
        py.allow_threads(move || Ok(private_equity::ks_pme(&amounts, &index, nav.unwrap_or(0.0))?))
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/ks_pme.md")]
    fn ks_pme_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<f64> {
        py.allow_threads(move || {
            Ok(private_equity::ks_pme_2(
                &contributions,
                &distributions,
                &index,
                nav.unwrap_or(0.0),
            )?)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/ks_pme_flows.md")]
    fn ks_pme_flows(py: Python, amounts: AmountArray, index: AmountArray) -> PyResult<Vec<f64>> {
        py.allow_threads(move || Ok(private_equity::ks_pme_flows(&amounts, &index)?))
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/ks_pme_flows.md")]
    fn ks_pme_flows_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        index: AmountArray,
    ) -> PyResult<(Vec<f64>, Vec<f64>)> {
        py.allow_threads(move || {
            Ok(private_equity::ks_pme_flows_2(&contributions, &distributions, &index)?)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/m_pme.md")]
    fn m_pme(
        py: Python,
        amounts: AmountArray,
        index: AmountArray,
        nav: AmountArray,
    ) -> PyResult<f64> {
        py.allow_threads(move || Ok(private_equity::m_pme(&amounts, &index, &nav)?))
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/m_pme.md")]
    fn m_pme_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        index: AmountArray,
        nav: AmountArray,
    ) -> PyResult<f64> {
        py.allow_threads(move || {
            Ok(private_equity::m_pme_2(&contributions, &distributions, &index, &nav)?)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/pme_plus.md")]
    fn pme_plus(
        py: Python,
        amounts: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<Option<f64>> {
        py.allow_threads(move || {
            fallible_float_or_none(
                private_equity::pme_plus(&amounts, &index, nav.unwrap_or(0.0)),
                false,
            )
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/pme_plus.md")]
    fn pme_plus_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<Option<f64>> {
        py.allow_threads(move || {
            fallible_float_or_none(
                private_equity::pme_plus_2(
                    &contributions,
                    &distributions,
                    &index,
                    nav.unwrap_or(0.0),
                ),
                false,
            )
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/pme_plus_flows.md")]
    fn pme_plus_flows(
        py: Python,
        amounts: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<Vec<f64>> {
        py.allow_threads(move || {
            Ok(private_equity::pme_plus_flows(&amounts, &index, nav.unwrap_or(0.0))?)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/pme_plus_flows.md")]
    fn pme_plus_flows_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<(Vec<f64>, Vec<f64>)> {
        py.allow_threads(move || {
            let adj_distributions = private_equity::pme_plus_flows_2(
                &contributions,
                &distributions,
                &index,
                nav.unwrap_or(0.0),
            )?;

            Ok((contributions.to_vec(), adj_distributions))
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/pme_plus_lambda.md")]
    fn pme_plus_lambda(
        py: Python,
        amounts: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<f64> {
        py.allow_threads(move || {
            Ok(private_equity::pme_plus_lambda(&amounts, &index, nav.unwrap_or(0.0))?)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/pme_plus_lambda.md")]
    fn pme_plus_lambda_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<f64> {
        py.allow_threads(move || {
            Ok(private_equity::pme_plus_lambda_2(
                &contributions,
                &distributions,
                &index,
                nav.unwrap_or(0.0),
            )?)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/ln_pme_nav.md")]
    fn ln_pme_nav(py: Python, amounts: AmountArray, index: AmountArray) -> PyResult<f64> {
        py.allow_threads(move || Ok(private_equity::ln_pme_nav(&amounts, &index)?))
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/ln_pme_nav.md")]
    fn ln_pme_nav_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        index: AmountArray,
    ) -> PyResult<f64> {
        py.allow_threads(move || {
            Ok(private_equity::ln_pme_nav_2(&contributions, &distributions, &index)?)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/ln_pme.md")]
    fn ln_pme(py: Python, amounts: AmountArray, index: AmountArray) -> PyResult<Option<f64>> {
        py.allow_threads(move || {
            fallible_float_or_none(private_equity::ln_pme(&amounts, &index), false)
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/ln_pme.md")]
    fn ln_pme_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        index: AmountArray,
    ) -> PyResult<Option<f64>> {
        py.allow_threads(move || {
            fallible_float_or_none(
                private_equity::ln_pme_2(&contributions, &distributions, &index),
                false,
            )
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/direct_alpha.md")]
    fn direct_alpha(
        py: Python,
        amounts: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<Option<f64>> {
        py.allow_threads(move || {
            fallible_float_or_none(
                private_equity::direct_alpha(&amounts, &index, nav.unwrap_or(0.0)),
                false,
            )
        })
    }

    #[pyfunction]
    #[doc = include_str!("../docs/_inline/pe/direct_alpha.md")]
    fn direct_alpha_2(
        py: Python,
        contributions: AmountArray,
        distributions: AmountArray,
        index: AmountArray,
        nav: Option<f64>,
    ) -> PyResult<Option<f64>> {
        py.allow_threads(move || {
            fallible_float_or_none(
                private_equity::direct_alpha_2(
                    &contributions,
                    &distributions,
                    &index,
                    nav.unwrap_or(0.0),
                ),
                false,
            )
        })
    }
}

fn add_submodule<F>(py: Python, parent: &PyModule, name: &str, mod_init: F) -> PyResult<()>
where
    F: Fn(Python, &PyModule) -> PyResult<()>,
{
    let child_module = PyModule::new(py, name)?;
    mod_init(py, child_module)?;
    parent.add(name.split(".").last().unwrap(), child_module)?;
    py.import("sys")?.getattr("modules")?.set_item(name, child_module)?;
    Ok(())
}

#[pymodule]
#[pyo3(name = "_pyxirr")]
pub fn pyxirr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    add_submodule(py, m, "pyxirr.pe", pe::module)?;

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
    m.add_function(wrap_pyfunction!(is_conventional_cash_flow, m)?)?;
    m.add_function(wrap_pyfunction!(zero_crossing_points, m)?)?;

    m.add("InvalidPaymentsError", py.get_type::<InvalidPaymentsError>())?;
    m.add("BroadcastingError", py.get_type::<BroadcastingError>())?;

    Ok(())
}
