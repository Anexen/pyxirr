use rstest::rstest;

use pyo3::{types::PyList, Python};

use pyxirr;

mod common;
use common::{irr_expected_result, PaymentsLoader};

const INTEREST_RATE: f64 = 0.05;
const PERIODS: f64 = 10.0;
const PAYMENT: f64 = -50_000.0;
const PV: f64 = 100_000.0;
const FV: f64 = 110_000.0;

#[cfg(not(feature = "nonumpy"))]
#[rstest]
fn test_fv_macro_working() {
    assert_future_value!(INTEREST_RATE, PERIODS, -12950.4574965456, PV, None, None);
    assert_future_value!(INTEREST_RATE, PERIODS, -12333.7690443292, PV, None, Some(true));
    assert_future_value!(INTEREST_RATE, PERIODS, -21695.9607427458, PV, Some(FV), None);
    assert_future_value!(INTEREST_RATE, PERIODS, -20662.8197549960, PV, Some(FV), Some(true));
}

// ------------ FV ----------------

#[rstest]
fn test_fv_pmt_at_end() {
    let result = pyxirr::fv(0.05 / 12.0, 10.0 * 12.0, -100.0, -100.0, None).unwrap();
    assert_almost_eq!(result, 15692.9288943357);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_fv = py.import("numpy_financial").unwrap().getattr("fv").unwrap();
            let npf_result = npf_fv.call1((0.05 / 12.0, 10.0 * 12.0, -100.0, -100.0));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_fv_pmt_at_begining() {
    let result = pyxirr::fv(0.05 / 12.0, 10.0 * 12.0, -100.0, -100.0, Some(true)).unwrap();
    assert_almost_eq!(result, 15757.6298441047);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_fv = py.import("numpy_financial").unwrap().getattr("fv").unwrap();
            let npf_result = npf_fv.call1((0.05 / 12.0, 10.0 * 12.0, -100.0, -100.0, "start"));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_fv_zero_rate() {
    let result = pyxirr::fv(0.0, 10.0 * 12.0, -100.0, -100.0, None).unwrap();
    assert_almost_eq!(result, 12100.0);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_fv = py.import("numpy_financial").unwrap().getattr("fv").unwrap();
            let npf_result = npf_fv.call1((0.0, 10.0 * 12.0, -100.0, -100.0));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

// ------------ PV ----------------

#[rstest]
fn test_pv_pmt_at_end() {
    let result = pyxirr::pv(0.05 / 12.0, 10.0 * 12.0, -100.0, Some(15692.93), None).unwrap();
    assert_almost_eq!(result, -100.0006713162);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_pv = py.import("numpy_financial").unwrap().getattr("pv").unwrap();
            let npf_result = npf_pv.call1((0.05 / 12.0, 10.0 * 12.0, -100.0, 15692.93));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_pv_pmt_at_begining() {
    let result = pyxirr::pv(0.05 / 12.0, 10.0 * 12.0, -100.0, Some(15692.93), Some(true)).unwrap();
    assert_almost_eq!(result, -60.71677534615);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_pv = py.import("numpy_financial").unwrap().getattr("pv").unwrap();
            let npf_result = npf_pv.call1((0.05 / 12.0, 10.0 * 12.0, -100.0, 15692.93, "start"));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_pv_zero_rate() {
    let result = pyxirr::pv(0.0, 10.0 * 12.0, -100.0, Some(15692.93), None).unwrap();
    assert_almost_eq!(result, -3692.93);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_pv = py.import("numpy_financial").unwrap().getattr("pv").unwrap();
            let npf_result = npf_pv.call1((0.0, 10.0 * 12.0, -100.0, 15692.93));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_pv_default_pv() {
    let result = pyxirr::pv(0.05 / 12.0, 10.0 * 12.0, -100.0, None, None).unwrap();
    assert_almost_eq!(result, 9428.1350328234);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_pv = py.import("numpy_financial").unwrap().getattr("pv").unwrap();
            let npf_result = npf_pv.call1((0.05 / 12.0, 10.0 * 12.0, -100.0));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

// ------------ NPV ----------------

#[rstest]
fn test_npv_works() {
    Python::with_gil(|py| {
        let values = PyList::new(py, &[-40_000., 5_000., 8_000., 12_000., 30_000.]);
        let result = pyxirr::npv(0.08, values, None).unwrap().unwrap();
        assert_almost_eq!(result, 3065.222668179);

        if cfg!(not(feature = "nonumpy")) {
            let npf_npv = py.import("numpy_financial").unwrap().getattr("npv").unwrap();
            let npf_result = npf_npv.call1((0.08, values));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        }
    });
}

#[rstest]
fn test_npv_start_from_zero() {
    Python::with_gil(|py| {
        let values = PyList::new(py, &[-40_000., 5_000., 8_000., 12_000., 30_000.]);
        let result = pyxirr::npv(0.08, values, Some(false)).unwrap().unwrap();
        assert_almost_eq!(result, 2838.169137203);
    });
}

#[rstest]
fn test_npv_zero_rate() {
    Python::with_gil(|py| {
        let values = PyList::new(py, &[-40_000., 5_000., 8_000., 12_000., 30_000.]);
        let result = pyxirr::npv(0., values, Some(false)).unwrap().unwrap();
        assert_almost_eq!(result, 15_000.0);

        if cfg!(not(feature = "nonumpy")) {
            let npf_npv = py.import("numpy_financial").unwrap().getattr("npv").unwrap();
            let npf_result = npf_npv.call1((0.0, values));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        }
    });
}

// ------------ PMT ----------------

#[rstest]
fn test_pmt_pmt_at_end() {
    let pmt = pyxirr::pmt(INTEREST_RATE, PERIODS, PV, None, None).unwrap();
    assert_future_value!(INTEREST_RATE, PERIODS, pmt, PV, None, None);
    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_pmt = py.import("numpy_financial").unwrap().getattr("pmt").unwrap();
            let npf_result = npf_pmt.call1((INTEREST_RATE, PERIODS, PV));
            assert_almost_eq!(pmt, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_pmt_pmt_at_begining() {
    let pmt = pyxirr::pmt(INTEREST_RATE, PERIODS, PV, None, Some(true)).unwrap();
    assert_future_value!(INTEREST_RATE, PERIODS, pmt, PV, None, Some(true));
    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_pmt = py.import("numpy_financial").unwrap().getattr("pmt").unwrap();
            let npf_result = npf_pmt.call1((INTEREST_RATE, PERIODS, PV, 0, "start"));
            assert_almost_eq!(pmt, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_pmt_non_zero_fv() {
    let pmt = pyxirr::pmt(INTEREST_RATE, PERIODS, PV, Some(FV), None).unwrap();
    assert_future_value!(INTEREST_RATE, PERIODS, pmt, PV, Some(FV), None);
    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_pmt = py.import("numpy_financial").unwrap().getattr("pmt").unwrap();
            let npf_result = npf_pmt.call1((INTEREST_RATE, PERIODS, PV, FV));
            assert_almost_eq!(pmt, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_pmt_zero_rate() {
    let pmt = pyxirr::pmt(0.0, PERIODS, PV, Some(FV), None).unwrap();
    assert_future_value!(0.0, PERIODS, pmt, PV, Some(FV), None);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_pmt = py.import("numpy_financial").unwrap().getattr("pmt").unwrap();
            let npf_result = npf_pmt.call1((0, PERIODS, PV, FV));
            assert_almost_eq!(pmt, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

// ------------ IPMT ----------------

#[rstest]
fn test_ipmt_works() {
    let result = pyxirr::ipmt(INTEREST_RATE, 2.0, PERIODS, PAYMENT, None, None).unwrap();
    assert_almost_eq!(result, 2301.238562586);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_ipmt = py.import("numpy_financial").unwrap().getattr("ipmt").unwrap();
            let npf_result = npf_ipmt.call1((INTEREST_RATE, 2, PERIODS, PAYMENT));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_ipmt_pmt_at_begining() {
    let result = pyxirr::ipmt(INTEREST_RATE, 2.0, PERIODS, PAYMENT, None, Some(true)).unwrap();
    assert_almost_eq!(result, 2191.6557738917);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_ipmt = py.import("numpy_financial").unwrap().getattr("ipmt").unwrap();
            let npf_result = npf_ipmt.call1((INTEREST_RATE, 2, PERIODS, PAYMENT, 0, "start"));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_ipmt_non_zero_fv() {
    let result = pyxirr::ipmt(INTEREST_RATE, 2.0, PERIODS, PAYMENT, Some(FV), Some(true)).unwrap();
    assert_almost_eq!(result, 2608.108309425);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_ipmt = py.import("numpy_financial").unwrap().getattr("ipmt").unwrap();
            let npf_result = npf_ipmt.call1((INTEREST_RATE, 2, PERIODS, PAYMENT, FV, "start"));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_ipmt_first_period() {
    let result = pyxirr::ipmt(INTEREST_RATE, 1.0, PERIODS, PAYMENT, None, None).unwrap();
    assert_almost_eq!(result, -PAYMENT * INTEREST_RATE);
}

#[rstest]
fn test_ipmt_zero_period() {
    assert!(pyxirr::ipmt(INTEREST_RATE, 0.0, PERIODS, PAYMENT, None, None).is_none());
}

#[rstest]
fn test_ipmt_per_greater_than_nper() {
    let result = pyxirr::ipmt(INTEREST_RATE, PERIODS + 2.0, PERIODS, PAYMENT, None, None).unwrap();
    assert_almost_eq!(result, -323.7614374136);
}

// ------------ PPMT ----------------

#[rstest]
fn test_ppmt_works() {
    let result = pyxirr::ppmt(INTEREST_RATE, 2.0, PERIODS, PAYMENT, None, None).unwrap();
    assert_almost_eq!(result, 4173.9901856864);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_ppmt = py.import("numpy_financial").unwrap().getattr("ppmt").unwrap();
            let npf_result = npf_ppmt.call1((INTEREST_RATE, 2, PERIODS, PAYMENT));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

// ------------ NPER ----------------

#[rstest]
fn test_nper_pmt_at_end() {
    let nper = pyxirr::nper(INTEREST_RATE, PAYMENT, PV, None, None).unwrap();
    assert_future_value!(INTEREST_RATE, nper, PAYMENT, PV, None, None);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_nper = py.import("numpy_financial").unwrap().getattr("nper").unwrap();
            let npf_result = npf_nper.call1((INTEREST_RATE, PAYMENT, PV));
            assert_almost_eq!(nper, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_nper_pmt_at_begining() {
    let nper = pyxirr::nper(INTEREST_RATE, PAYMENT, PV, None, Some(true)).unwrap();
    assert_future_value!(INTEREST_RATE, nper, PAYMENT, PV, None, Some(true));

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_nper = py.import("numpy_financial").unwrap().getattr("nper").unwrap();
            let npf_result = npf_nper.call1((INTEREST_RATE, PAYMENT, PV, 0, "start"));
            assert_almost_eq!(nper, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_nper_non_zero_fv() {
    let nper = pyxirr::nper(INTEREST_RATE, PAYMENT, PV, Some(FV), None).unwrap();
    assert_future_value!(INTEREST_RATE, nper, PAYMENT, PV, Some(FV), None);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_nper = py.import("numpy_financial").unwrap().getattr("nper").unwrap();
            let npf_result = npf_nper.call1((INTEREST_RATE, PAYMENT, PV, FV));
            assert_almost_eq!(nper, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_nper_zero_rate() {
    let nper = pyxirr::nper(0.0, PAYMENT, PV, Some(FV), None).unwrap();
    assert_future_value!(0.0, nper, PAYMENT, PV, Some(FV), None);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_nper = py.import("numpy_financial").unwrap().getattr("nper").unwrap();
            let npf_result = npf_nper.call1((0, PERIODS, PAYMENT, FV));
            assert_almost_eq!(nper, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

// ------------ RATE ----------------

#[rstest]
fn test_rate_works() {
    let rate = pyxirr::rate(PERIODS, PAYMENT, PV, None, None, None).unwrap();
    assert_future_value!(rate, PERIODS, PAYMENT, PV, None, None);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_rate = py.import("numpy_financial").unwrap().getattr("rate").unwrap();
            let npf_result = npf_rate.call1((PERIODS, PAYMENT, PV, 0));
            assert_almost_eq!(rate, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_rate_non_zero_fv() {
    let rate = pyxirr::rate(PERIODS, PAYMENT, PV, Some(FV), None, None).unwrap();
    assert_future_value!(rate, PERIODS, PAYMENT, PV, Some(FV), None);

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_rate = py.import("numpy_financial").unwrap().getattr("rate").unwrap();
            let npf_result = npf_rate.call1((PERIODS, PAYMENT, PV, FV));
            assert_almost_eq!(rate, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

#[rstest]
fn test_rate_pmt_at_begining() {
    let rate = pyxirr::rate(PERIODS, PAYMENT, PV, Some(FV), Some(true), None).unwrap();
    assert_future_value!(rate, PERIODS, PAYMENT, PV, Some(FV), Some(true));

    if cfg!(not(feature = "nonumpy")) {
        Python::with_gil(|py| {
            let npf_rate = py.import("numpy_financial").unwrap().getattr("rate").unwrap();
            let npf_result = npf_rate.call1((PERIODS, PAYMENT, PV, FV, "start"));
            assert_almost_eq!(rate, npf_result.unwrap().extract::<f64>().unwrap());
        })
    }
}

// ------------ NFV ----------------

#[rstest]
fn test_nfv() {
    // example from https://www.youtube.com/watch?v=775ljhriB8U
    Python::with_gil(|py| {
        let amounts = PyList::new(py, &[1050.0, 1350.0, 1350.0, 1450.0]);
        let result = pyxirr::nfv(0.03, 6.0, amounts).unwrap().unwrap();
        assert_almost_eq!(result, 5750.16, 0.01);
    });
}

// ------------ IRR ----------------

// test cases from numpy_finance.irr
#[rstest]
#[case(&[-100, 39, 59, 55, 20], 0.28094842116)]
#[case(&[-100, 0, 0, 74], -0.09549583034)]
#[case(&[-100, 100, 0, -7], -0.08329966618)]
fn test_irr_works(#[case] input: &[i64], #[case] expected: f64) {
    Python::with_gil(|py| {
        let values = PyList::new(py, input);
        let result = pyxirr::irr(values, None).unwrap().unwrap();
        assert_almost_eq!(result, expected);

        if cfg!(not(feature = "nonumpy")) {
            let npf_irr = py.import("numpy_financial").unwrap().getattr("irr").unwrap();
            let npf_result = npf_irr.call1((values,));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        }
    })
}

#[rstest]
#[case::unordered("tests/samples/unordered.csv")]
#[case::random_100("tests/samples/random_100.csv")]
#[case::random_1000("tests/samples/random_1000.csv")]
fn test_irr_samples(#[case] input: &str) {
    Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_columns();
        let result = pyxirr::irr(payments.1, None).unwrap().unwrap();

        assert_almost_eq!(result, irr_expected_result(input));
        // test net present value of all cash flows equal to zero
        assert_almost_eq!(pyxirr::npv(result, payments.1, None).unwrap().unwrap(), 0.0);

        // npf returns wrong results (npv is not equal to zero):
        // if cfg!(not(feature = "nonumpy")) {
        //     let npf_irr = py.import("numpy_financial").unwrap().getattr("irr").unwrap();
        //     let npf_result = npf_irr.call1((payments.1,));
        //     assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        // }
    });
}

// ------------ MIRR ----------------

#[rstest]
fn test_mirr_works() {
    Python::with_gil(|py| {
        let values = PyList::new(py, &[-1000, 100, 250, 500, 500]);
        let result = pyxirr::mirr(values, 0.1, 0.1).unwrap().unwrap();
        assert_almost_eq!(result, 0.10401626745);

        if cfg!(not(feature = "nonumpy")) {
            let npf_mirr = py.import("numpy_financial").unwrap().getattr("mirr").unwrap();
            let npf_result = npf_mirr.call1((values, 0.1, 0.1));
            assert_almost_eq!(result, npf_result.unwrap().extract::<f64>().unwrap());
        }
    });
}

#[rstest]
fn test_mirr_same_sign() {
    Python::with_gil(|py| {
        let values = PyList::new(py, &[100_000.0, 50_000.0, 25_000.0]);
        assert!(pyxirr::mirr(values, 0.1, 0.1).unwrap().is_none());

        let values = PyList::new(py, &[-100_000.0, -50_000.0, -25_000.0]);
        assert!(pyxirr::mirr(values, 0.1, 0.1).unwrap().is_none());
    });
}
