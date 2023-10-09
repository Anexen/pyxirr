use numpy::{pyarray, PyArrayDyn};
use pyo3::{types::PyList, Python};
use rstest::rstest;

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
    Python::with_gil(|py| {
        let args = (0.05 / 12.0, 10.0 * 12.0, -100.0, -100.0);
        let result: f64 = pyxirr_call!(py, "fv", args);
        assert_almost_eq!(result, 15692.9288943357);
    })
}

#[rstest]
fn test_fv_pmt_at_beginning() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(
            py,
            "fv",
            (0.05 / 12.0, 10 * 12, -100, -100),
            py_dict!(py, "pmt_at_beginning" => true)
        );
        assert_almost_eq!(result, 15757.6298441047);
    })
}

#[rstest]
fn test_fv_zero_rate() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "fv", (0, 10 * 12, -100, -100));
        assert_almost_eq!(result, 12100.0);
    })
}

#[rstest]
fn test_fv_vectorized() {
    Python::with_gil(|py| {
        let rates = [[0.05 / 12.0, 0.06 / 12.0], [0.07 / 12.0, 0.0]];
        let result: Vec<Vec<f64>> = pyxirr_call!(py, "fv", (rates, 10 * 12, -100, -100));

        assert_almost_eq!(result[0][0], 15692.928894335748);
        assert_almost_eq!(result[0][1], 16569.874354049032);
        assert_almost_eq!(result[1][0], 17509.446881023265);
        assert_almost_eq!(result[1][1], 12100.0);
    })
}

#[rstest]
fn test_fv_vectorized_multi() {
    Python::with_gil(|py| {
        let rates = [0.05 / 12.0, 0.06 / 12.0, 0.07 / 12.0];
        let nper = [5 * 12, 10 * 12, 12 * 12];
        let pv = [-100, -150, -200];
        let kwargs = py_dict!(py, "pmt_at_beginning" => [false, false, true]);

        let result: Vec<f64> = pyxirr_call!(py, "fv", (rates, nper, -100, pv), kwargs);

        assert_almost_eq!(result[0], 6928.944151934635);
        assert_almost_eq!(result[1], 16660.844190750646);
        assert_almost_eq!(result[2], 23062.71469294612);
    })
}

#[rstest]
fn test_fv_vectorized_iterable() {
    Python::with_gil(|py| {
        let pmt = py.eval("range(-100, -400, -100)", None, None).unwrap();
        let actual: Vec<f64> = pyxirr_call!(py, "fv", (0.05 / 12.0, 10 * 12, pmt, -100));
        let expected = vec![15692.92889433575, 31221.15683890247, 46749.38478346919];
        for i in 0..actual.len() {
            assert_almost_eq!(actual[i], expected[i]);
        }
    });
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_fv_vectorized_ndarray() {
    // pyarray input -> pyarray output
    // pylist input -> pylist output
    Python::with_gil(|py| {
        let rates = pyarray!(py, [[0.05 / 12.0, 0.06 / 12.0], [0.07 / 12.0, 0.0]]);

        let actual: &PyArrayDyn<f64> =
            pyxirr_call!(py, "fv", (rates.as_ref(), 10 * 12, -100, -100));

        let expected =
            pyarray![py, [15692.928894335748, 16569.874354049032], [17509.446881023265, 12100.0]];

        actual.readonly().as_array().iter().zip(expected.readonly().as_array().iter()).for_each(
            |(a, e)| {
                assert_almost_eq!(a, e);
            },
        );

        let actual: &PyList =
            pyxirr_call!(py, "fv", (rates.to_vec().unwrap(), 10 * 12, -100, -100));

        actual
            .iter()
            .map(|a| a.extract::<f64>().unwrap())
            .zip(expected.readonly().as_array().iter())
            .for_each(|(a, e)| {
                assert_almost_eq!(a, e);
            });
    })
}

// ------------ PV ----------------

#[rstest]
fn test_pv_pmt_at_end() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "pv", (0.05 / 12.0, 10 * 12, -100, 15692.93));
        assert_almost_eq!(result, -100.0006713162);
    })
}

#[rstest]
fn test_pv_pmt_at_beginning() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(
            py,
            "pv",
            (0.05 / 12.0, 10 * 12, -100, 15692.93),
            py_dict!(py, "pmt_at_beginning" => true)
        );
        assert_almost_eq!(result, -60.71677534615);
    })
}

#[rstest]
fn test_pv_zero_rate() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "pv", (0, 10 * 12, -100, 15692.93));
        assert_almost_eq!(result, -3692.93);
    })
}

#[rstest]
fn test_pv_default_pv() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "pv", (0.05 / 12.0, 10 * 12, -100));
        assert_almost_eq!(result, 9428.1350328234);
    })
}

#[rstest]
fn test_pv_vectorized() {
    Python::with_gil(|py| {
        let rates = [[0.05 / 12.0, 0.06 / 12.0], [0.07 / 12.0, 0.0]];
        let result: Vec<Vec<f64>> = pyxirr_call!(py, "pv", (rates, 10 * 12, -100));

        assert_almost_eq!(result[0][0], 9428.135032823473);
        assert_almost_eq!(result[0][1], 9007.345332716726);
        assert_almost_eq!(result[1][0], 8612.635414137785);
        assert_almost_eq!(result[1][1], 12000.0);
    })
}

// ------------ NPV ----------------

#[rstest]
fn test_npv_works() {
    Python::with_gil(|py| {
        let values = PyList::new(py, [-40_000., 5_000., 8_000., 12_000., 30_000.]);
        let result: f64 = pyxirr_call!(py, "npv", (0.08, values));
        assert_almost_eq!(result, 3065.222668179);
    });
}

#[rstest]
fn test_npv_start_from_zero() {
    Python::with_gil(|py| {
        let values = PyList::new(py, [-40_000., 5_000., 8_000., 12_000., 30_000.]);
        let result: f64 =
            pyxirr_call!(py, "npv", (0.08, values), py_dict!(py, "start_from_zero" => false));
        assert_almost_eq!(result, 2838.169137203);
    });
}

#[rstest]
fn test_npv_zero_rate() {
    Python::with_gil(|py| {
        let values = PyList::new(py, [-40_000., 5_000., 8_000., 12_000., 30_000.]);
        let result: f64 =
            pyxirr_call!(py, "npv", (0, values), py_dict!(py, "start_from_zero" => false));
        assert_almost_eq!(result, 15_000.0);
    });
}

// ------------ PMT ----------------

#[rstest]
fn test_pmt_pmt_at_end() {
    Python::with_gil(|py| {
        let pmt: f64 = pyxirr_call!(py, "pmt", (INTEREST_RATE, PERIODS, PV));
        assert_future_value!(INTEREST_RATE, PERIODS, pmt, PV, None, None);
    })
}

#[rstest]
fn test_pmt_pmt_at_beginning() {
    Python::with_gil(|py| {
        let pmt: f64 = pyxirr_call!(
            py,
            "pmt",
            (INTEREST_RATE, PERIODS, PV),
            py_dict!(py, "pmt_at_beginning" => true)
        );
        assert_future_value!(INTEREST_RATE, PERIODS, pmt, PV, None, Some(true));
    })
}

#[rstest]
fn test_pmt_non_zero_fv() {
    Python::with_gil(|py| {
        let pmt: f64 = pyxirr_call!(py, "pmt", (INTEREST_RATE, PERIODS, PV, FV));
        assert_future_value!(INTEREST_RATE, PERIODS, pmt, PV, Some(FV), None);
    })
}

#[rstest]
fn test_pmt_zero_rate() {
    Python::with_gil(|py| {
        let pmt: f64 = pyxirr_call!(py, "pmt", (0, PERIODS, PV, FV));
        assert_future_value!(0.0, PERIODS, pmt, PV, Some(FV), None);
    })
}

#[rstest]
fn test_pmt_vec() {
    Python::with_gil(|py| {
        let rates = [[0.075 / 12., 0.01 / 12.], [0.0, 0.5 / 12.]];
        let result: Vec<Vec<f64>> = pyxirr_call!(py, "pmt", (rates, 12 * 15, 200_000));
        assert_almost_eq!(result[0][0], -1854.0247200054619);
        assert_almost_eq!(result[0][1], -1196.9890290366611);
        assert_almost_eq!(result[1][0], -1111.111111111111);
        assert_almost_eq!(result[1][1], -8338.702667524864);
    })
}

// ------------ IPMT ----------------

#[rstest]
fn test_ipmt_works() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "ipmt", (INTEREST_RATE, 2.0, PERIODS, PAYMENT));
        assert_almost_eq!(result, 2301.238562586);
    })
}

#[rstest]
fn test_ipmt_pmt_at_beginning() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(
            py,
            "ipmt",
            (INTEREST_RATE, 2.0, PERIODS, PAYMENT),
            py_dict!(py, "pmt_at_beginning" => true)
        );
        assert_almost_eq!(result, 2191.6557738917);
    })
}

#[rstest]
fn test_ipmt_non_zero_fv() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(
            py,
            "ipmt",
            (INTEREST_RATE, 2.0, PERIODS, PAYMENT, FV),
            py_dict!(py, "pmt_at_beginning" => true)
        );
        assert_almost_eq!(result, 2608.108309425);
    })
}

#[rstest]
fn test_ipmt_first_period() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "ipmt", (INTEREST_RATE, 1.0, PERIODS, PAYMENT));
        assert_almost_eq!(result, -PAYMENT * INTEREST_RATE);
    })
}

#[rstest]
fn test_ipmt_zero_period() {
    Python::with_gil(|py| {
        let result: Option<f64> = pyxirr_call!(py, "ipmt", (INTEREST_RATE, 0.0, PERIODS, PAYMENT));
        assert!(result.is_none());
    })
}

#[rstest]
fn test_ipmt_per_greater_than_nper() {
    Python::with_gil(|py| {
        let result: Option<f64> =
            pyxirr_call!(py, "ipmt", (INTEREST_RATE, PERIODS + 2.0, PERIODS, PAYMENT));
        assert_eq!(result, None);
    })
}

#[rstest]
fn test_ipmt_large_power() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "ipmt", (0.1479, 297, 300, -270.51));
        assert_almost_eq!(result, 16.9656277018672);

        let result: f64 = pyxirr_call!(py, "ipmt", (0.1479, 297, 300, -270.51, -100));
        assert_almost_eq!(result, 8.447346936597);

        let kwargs = py_dict!(py, "pmt_at_beginning" => true);

        let result: f64 = pyxirr_call!(py, "ipmt", (0.1479, 297, 300, -270.51), kwargs);
        assert_almost_eq!(result, 14.779708774167800);

        let result: f64 = pyxirr_call!(py, "ipmt", (0.1479, 297, 300, -270.51, -100), kwargs);
        assert_almost_eq!(result, 7.358957171005);
    })
}

#[rstest]
fn test_ipmt_vec() {
    Python::with_gil(|py| {
        let per = (0..=13).collect::<Vec<_>>();
        let n = per.len();
        let result: Vec<Option<f64>> = pyxirr_call!(py, "ipmt", (0.0824 / 12., per, 12, 25_000));
        let expected = vec![
            f64::NAN,
            -171.66666666666666,
            -157.89337457350777,
            -144.0255058746426,
            -130.06241114404526,
            -116.00343649629737,
            -101.84792355596869,
            -87.59520942678299,
            -73.2446266605768,
            -58.79550322604296,
            -44.24716247725825,
            -29.598923121998908,
            -14.850099189833006,
            f64::NAN,
        ];

        for i in 0..n {
            match result[i] {
                Some(v) => assert_almost_eq!(v, expected[i]),
                None => assert!(expected[i].is_nan()),
            }
        }
    })
}

#[rstest]
fn test_ipmt_vec_large_power() {
    Python::with_gil(|py| {
        let result: Vec<f64> = pyxirr_call!(
            py,
            "ipmt",
            ([0.0, 0.1479, 0.1479, 0.1479], 297, 300, -270.51, [0.0, 0.0, -100.0, 0.0]),
            py_dict!(py, "pmt_at_beginning" =>
                [false, false, false, true]
            )
        );

        let expected = vec![0.0, 16.9656277018672, 8.447346936597, 14.779708774167800];

        for i in 0..expected.len() {
            assert_almost_eq!(result[i], expected[i]);
        }
    })
}

// ------------ PPMT ----------------

#[rstest]
fn test_ppmt_works() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "ppmt", (INTEREST_RATE, 2.0, PERIODS, PAYMENT));
        assert_almost_eq!(result, 4173.9901856864);

        let kwargs = py_dict!(py, "pmt_at_beginning" => true);
        let result: f64 = pyxirr_call!(py, "ppmt", (INTEREST_RATE, 2.0, PERIODS, PAYMENT), kwargs);
        assert_almost_eq!(result, 3975.2287482728307);

        let result: Option<f64> = pyxirr_call!(py, "ppmt", (INTEREST_RATE, 0, 10, PAYMENT));
        assert!(result.is_none());

        let result: Option<f64> = pyxirr_call!(py, "ppmt", (INTEREST_RATE, 11, 10, PAYMENT));
        assert!(result.is_none());
    })
}

#[rstest]
fn test_ppmt_zero_rate() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "ppmt", (0, 2.0, PERIODS, PAYMENT));
        assert_almost_eq!(result, 5000.);

        let kwargs = py_dict!(py, "pmt_at_beginning" => true);
        let result: f64 = pyxirr_call!(py, "ppmt", (0, 2.0, PERIODS, PAYMENT), kwargs);
        assert_almost_eq!(result, 5000.);
    })
}

#[rstest]
fn test_ppmt_large_power() {
    // https://github.com/numpy/numpy-financial/issues/35
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "ppmt", (0.1479, 297, 300, -270.51));
        assert_almost_eq!(result, 23.0428012981328);

        let kwargs = py_dict!(py, "pmt_at_beginning" => true);
        let result: f64 = pyxirr_call!(py, "ppmt", (0.1479, 297, 300, -270.51), kwargs);
        assert_almost_eq!(result, 20.0738751617151);

        let result: f64 = pyxirr_call!(py, "ppmt", (0, 297, 300, -270.51));
        assert_almost_eq!(result, 0.9017);
    })
}

#[rstest]
fn test_ppmt_vec() {
    Python::with_gil(|py| {
        let per = (1..6).collect::<Vec<_>>();
        let result: Vec<f64> = pyxirr_call!(py, "ppmt", (0.1 / 12., per, 24, 2000));
        let expected = vec![
            -75.62318600836664,
            -76.25337922510303,
            -76.88882405197889,
            -77.52956425241204,
            -78.17564395451548,
        ];

        for i in 0..expected.len() {
            assert_almost_eq!(result[i], expected[i])
        }

        let result: Vec<f64> = pyxirr_call!(
            py,
            "ppmt",
            ([0., 0., 0.05, 0.05], 2, 10, -50_000),
            py_dict!(py, "pmt_at_beginning" => [true, false, true, false])
        );

        let expected = vec![5000., 5000., 3975.2287482728307, 4173.9901856864];
        for i in 0..expected.len() {
            assert_almost_eq!(result[i], expected[i])
        }

        let result: Vec<Option<f64>> = pyxirr_call!(py, "ppmt", (0.05, [0, 11], 10, -100));
        result.into_iter().for_each(|x| assert!(x.is_none()))
    })
}

// ------------ NPER ----------------

#[rstest]
fn test_nper_pmt_at_end() {
    Python::with_gil(|py| {
        let nper: f64 = pyxirr_call!(py, "nper", (INTEREST_RATE, PAYMENT, PV));
        assert_future_value!(INTEREST_RATE, nper, PAYMENT, PV, None, None);
    })
}

#[rstest]
fn test_nper_pmt_at_beginning() {
    Python::with_gil(|py| {
        let nper: f64 = pyxirr_call!(
            py,
            "nper",
            (INTEREST_RATE, PAYMENT, PV),
            py_dict!(py, "pmt_at_beginning" => true)
        );
        assert_future_value!(INTEREST_RATE, nper, PAYMENT, PV, None, Some(true));
    })
}

#[rstest]
fn test_nper_non_zero_fv() {
    Python::with_gil(|py| {
        let nper: f64 = pyxirr_call!(py, "nper", (INTEREST_RATE, PAYMENT, PV, FV));
        assert_future_value!(INTEREST_RATE, nper, PAYMENT, PV, Some(FV), None);
    })
}

#[rstest]
fn test_nper_zero_rate() {
    Python::with_gil(|py| {
        let nper: f64 = pyxirr_call!(py, "nper", (0.0, PAYMENT, PV, FV));
        assert_future_value!(0.0, nper, PAYMENT, PV, Some(FV), None);
    })
}

#[rstest]
fn test_nper_vec() {
    Python::with_gil(|py| {
        let rates = [[[0.0]], [[0.075]]];
        let result: Vec<Vec<Vec<f64>>> = pyxirr_call!(py, "nper", (rates, -2000, 0, 100_000));

        assert_almost_eq!(result[0][0][0], 50.0);
        assert_almost_eq!(result[1][0][0], 21.544944197323336);
    })
}

// ------------ RATE ----------------

#[rstest]
fn test_rate_works() {
    Python::with_gil(|py| {
        let rate: f64 = pyxirr_call!(py, "rate", (PERIODS, PAYMENT, PV));
        assert_future_value!(rate, PERIODS, PAYMENT, PV, None, None);
    })
}

#[rstest]
fn test_rate_non_zero_fv() {
    Python::with_gil(|py| {
        let rate: f64 = pyxirr_call!(py, "rate", (PERIODS, PAYMENT, PV, FV));
        assert_future_value!(rate, PERIODS, PAYMENT, PV, Some(FV), None);
    })
}

#[rstest]
fn test_rate_pmt_at_beginning() {
    Python::with_gil(|py| {
        let rate: f64 = pyxirr_call!(
            py,
            "rate",
            (PERIODS, PAYMENT, PV, FV),
            py_dict!(py, "pmt_at_beginning" => true)
        );
        assert_future_value!(rate, PERIODS, PAYMENT, PV, Some(FV), Some(true));
    })
}

#[rstest]
fn test_rate_vec() {
    Python::with_gil(|py| {
        let pv = [-593.06, -4725.38, -662.05, -428.78, -13.65];
        let fv = [214.07, 4509.97, 224.11, 686.29, -329.67];

        let actual: Vec<Option<f64>> = pyxirr_call!(py, "rate", (2, 0, pv, fv));
        let expected = [-0.39920185, -0.02305873, -0.41818459, 0.26513414, f64::NAN];

        for i in 0..actual.len() {
            match actual[i] {
                Some(value) => assert_almost_eq!(value, expected[i], 1e-8),
                None => assert!(expected[i].is_nan()),
            }
        }
    })
}

// ------------ NFV ----------------

#[rstest]
fn test_nfv() {
    // example from https://www.youtube.com/watch?v=775ljhriB8U
    Python::with_gil(|py| {
        let amounts = PyList::new(py, [1050.0, 1350.0, 1350.0, 1450.0]);
        let result: f64 = pyxirr_call!(py, "nfv", (0.03, 6.0, amounts));
        assert_almost_eq!(result, 5750.16, 0.01);
    });
}

// ------------ IRR ----------------

#[rstest]
#[case(&[-100.0, 39.0, 59.0, 55.0, 20.0], 0.28094842116)]
#[case(&[-100.0, 0.0, 0.0, 74.0], -0.09549583034)]
#[case(&[-100.0, 100.0, 0.0, -7.0], -0.08329966618)]
#[case(&[-161445.03, 2113.73, 7626.73, 8619.84, 8612.92], -0.43658134635)]
#[case(&[-150000.0, 15000.0, 25000.0, 35000.0, 45000.0, 60000.0], 0.05243288885)]
#[case(&[-100.0, 0.0, 0.0, 74.0], -0.09549583034)]
#[case(&[-100.0, 39.0, 59.0, 55.0, 20.0], 0.28094842115)]
#[case(&[-100.0, 100.0, 0.0, -7.0], -0.08329966618)]
#[case(&[-100.0, 100.0, 0.0, 7.0], 0.06205848562)]
#[case(&[-5.0, 10.5, 1.0, -8.0, 1.0], 0.08859833852)]
#[case(&[-5.0, 10.5, 1.0, -8.0, 1.0, 0.0, 0.0, 0.0], 0.08859833852)]
#[case(&[-40000.0, 5000.0, 8000.0, 12000.0, 30000.0], 0.10582259840)]
fn test_irr_works(#[case] input: &[f64], #[case] expected: f64) {
    Python::with_gil(|py| {
        let values = PyList::new(py, input);
        let result: f64 = pyxirr_call!(py, "irr", (values,));
        assert_almost_eq!(result, expected);
    })
}

#[rstest]
#[case(&[87.17; 5], &[-86.43], -0.49367042606)]
#[case(&[-87.17; 180], &[5809.3], -0.01352676905)]
#[case(&[-172545.848122807], &[787.735232517999; 480], 0.0038401048)]
fn test_irr_equal_payments(#[case] first: &[f64], #[case] other: &[f64], #[case] expected: f64) {
    let input: Vec<_> = first.iter().chain(other).collect();

    Python::with_gil(|py| {
        let values = PyList::new(py, input);
        let result: f64 = pyxirr_call!(py, "irr", (values,));
        assert_almost_eq!(result, expected);
    })
}

#[rstest]
// https://github.com/numpy/numpy-financial/issues/44
#[case(&[-1678.87, 771.96, 1814.05, 3520.30, 3552.95, 3584.99, -1.0], 0.9688775470209261)]
#[case(&[-1678.87, 771.96, 1814.05, 3520.30, 3552.95, 3584.99, 4789.91, -1.0], 1.0042698487205577)]
// https://github.com/numpy/numpy-financial/issues/39
#[case(&[
    -217500.0, -217500.0, 108466.80462450592, 101129.96439328062, 93793.12416205535,
    86456.28393083003, 79119.44369960476, 71782.60346837944, 64445.76323715414,
    57108.92300592884, 49772.08277470355, 42435.24254347826, 35098.40231225296,
    27761.56208102766, 20424.721849802358, 13087.88161857707, 5751.041387351768,
    -1585.7988438735192, -8922.639075098821, -16259.479306324123, -23596.31953754941,
    -30933.159768774713, -38270.0, -45606.8402312253, -52943.680462450604,
    -60280.520693675906, -67617.36092490121
], 0.12)]
// https://github.com/numpy/numpy-financial/issues/28
#[case(&[-50.0, -100.0, 600.0, 300.0, -100.0], 1.8544178284461061)]
fn test_irr_special_cases(#[case] input: &[f64], #[case] expected: f64) {
    Python::with_gil(|py| {
        let values = PyList::new(py, input);
        let rate: f64 = pyxirr_call!(py, "irr", (values,));
        assert_almost_eq!(rate, expected);

        // test net present value of all cash flows equal to zero
        let npv: f64 = pyxirr_call!(py, "npv", (rate, values));
        assert_almost_eq!(npv, 0.0);
    })
}

#[rstest]
// https://github.com/Anexen/pyxirr/issues/46
#[case(&[
    -1.44852555e+08,  1.28859998e+06,  1.27305118e+06,  1.25407349e+06,
    1.24199669e+06,  1.22647792e+06,  1.21095552e+06,  1.19206955e+06,
    1.17989821e+06,  1.16436524e+06,  1.14883185e+06,  1.12945217e+06,
    1.11780102e+06,  1.10228427e+06,  1.08671783e+06,  1.06755759e+06,
    1.05502327e+06,  1.03885451e+06,  1.02247003e+06,  1.00227444e+06,
    9.88024873e+05
], -0.138541274008)]
#[case(&[
    -1.44852555e+08,  1.41881733e+06,  1.40267049e+06,  1.38296290e+06,
    1.37042160e+06,  1.35430596e+06,  1.33818655e+06,  1.31857419e+06,
    1.30593472e+06,  1.28980433e+06,  1.27367350e+06,  1.25354845e+06,
    1.24144917e+06,  1.22533563e+06,  1.20917048e+06,  1.18927331e+06,
    1.17625690e+06,  1.15946627e+06,  1.14245161e+06,  1.12147927e+06,
    1.10668164e+06
] , -0.13209372260468)]
fn test_gh_46(#[case] input: &[f64], #[case] expected: f64) {
    Python::with_gil(|py| {
        let values = PyList::new(py, input);
        let rate: f64 = pyxirr_call!(py, "irr", (values,));
        assert_almost_eq!(rate, expected);

        // test net present value of all cash flows equal to zero
        let npv: f64 = pyxirr_call!(py, "npv", (rate, values));
        assert_almost_eq!(npv, 0.0, 1e-6);
    })
}

#[rstest]
#[case::unordered("tests/samples/unordered.csv")]
#[case::random_100("tests/samples/random_100.csv")]
#[case::random_1000("tests/samples/random_1000.csv")]
fn test_irr_samples(#[case] input: &str) {
    Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_columns();
        let rate: f64 = pyxirr_call!(py, "irr", (payments.1,));

        assert_almost_eq!(rate, irr_expected_result(input));
        // test net present value of all cash flows equal to zero
        let npv: f64 = pyxirr_call!(py, "npv", (rate, payments.1));
        assert_almost_eq!(npv, 0.0);
    });
}

// ------------ MIRR ----------------

#[rstest]
fn test_mirr_works() {
    Python::with_gil(|py| {
        let values = PyList::new(py, [-1000, 100, 250, 500, 500]);
        let result: f64 = pyxirr_call!(py, "mirr", (values, 0.1, 0.1));
        assert_almost_eq!(result, 0.10401626745);
    });
}

#[rstest]
fn test_mirr_same_sign() {
    Python::with_gil(|py| {
        let kwargs = py_dict!(py, "silent" => true);

        let values = PyList::new(py, [100_000, 50_000, 25_000]);
        let err = pyxirr_call_impl!(py, "mirr", (values, 0.1, 0.1)).unwrap_err();
        assert!(err.is_instance(py, py.get_type::<pyxirr::InvalidPaymentsError>()));

        let result: Option<f64> = pyxirr_call!(py, "mirr", (values, 0.1, 0.1), kwargs);
        assert!(result.is_none());

        let values = PyList::new(py, [-100_000.0, -50_000.0, -25_000.0]);
        let err = pyxirr_call_impl!(py, "mirr", (values, 0.1, 0.1)).unwrap_err();
        assert!(err.is_instance(py, py.get_type::<pyxirr::InvalidPaymentsError>()));

        let result: Option<f64> = pyxirr_call!(py, "mirr", (values, 0.1, 0.1), kwargs);
        assert!(result.is_none());
    });
}

// ------------ CUMPRINC ----------------

#[rstest]
fn test_cumprinc_works() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "cumprinc", (0.09 / 12.0, 30 * 12, 125_000, 13, 24));
        assert_almost_eq!(result, -934.1071234, 1e-7);

        let result: f64 = pyxirr_call!(py, "cumprinc", (0.09 / 12.0, 30 * 12, 125_000, 1, 1));
        assert_almost_eq!(result, -68.27827118, 1e-7);
    });
}

// ------------ CUMIPMT ----------------

#[rstest]
fn test_cumipmt_works() {
    Python::with_gil(|py| {
        let result: f64 = pyxirr_call!(py, "cumipmt", (0.09 / 12.0, 30 * 12, 125_000, 13, 24));
        assert_almost_eq!(result, -11135.23213075);

        let result: f64 = pyxirr_call!(py, "cumipmt", (0.09 / 12.0, 30 * 12, 125_000, 1, 1));
        assert_almost_eq!(result, -937.5, 1e-7);
    });
}
