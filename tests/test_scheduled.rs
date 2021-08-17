use rstest::rstest;

use pyo3::types::{IntoPyDict, PyDate};
use pyo3::Python;

use pyxirr;

mod common;
use common::{xirr_expected_result, xnpv_expected_result, PaymentsLoader};

#[rstest]
#[case::unordered("tests/samples/unordered.csv")]
#[case::random_100("tests/samples/random_100.csv")]
fn test_xnpv_samples(#[case] input: &str) {
    let rate = 0.1;
    let result = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_records();
        pyxirr::xnpv(rate, payments, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, xnpv_expected_result(rate, input));
}

#[rstest]
#[case::unordered("tests/samples/unordered.csv")]
#[case::single_redemption("tests/samples/single_redemption.csv")]
#[case::random("tests/samples/random.csv")]
#[case::random_100("tests/samples/random_100.csv")]
#[case::random_1000("tests/samples/random_1000.csv")]
#[case::case_30_0("tests/samples/30-0.csv")]
#[case::case_30_1("tests/samples/30-1.csv")]
#[case::case_30_2("tests/samples/30-2.csv")]
#[case::case_30_3("tests/samples/30-3.csv")]
#[case::case_30_4("tests/samples/30-4.csv")]
#[case::case_30_5("tests/samples/30-5.csv")]
#[case::case_30_6("tests/samples/30-6.csv")]
#[case::case_30_7("tests/samples/30-7.csv")]
#[case::case_30_8("tests/samples/30-8.csv")]
#[case::case_30_9("tests/samples/30-9.csv")]
#[case::case_30_10("tests/samples/30-10.csv")]
#[case::case_30_11("tests/samples/30-11.csv")]
#[case::case_30_12("tests/samples/30-12.csv")]
#[case::case_30_13("tests/samples/30-13.csv")]
#[case::case_30_14("tests/samples/30-14.csv")]
#[case::case_30_15("tests/samples/30-15.csv")]
#[case::case_30_16("tests/samples/30-16.csv")]
#[case::case_30_17("tests/samples/30-17.csv")]
#[case::case_30_18("tests/samples/30-18.csv")]
#[case::case_30_19("tests/samples/30-19.csv")]
#[case::case_30_20("tests/samples/30-20.csv")]
#[case::case_30_21("tests/samples/30-21.csv")]
#[case::case_30_22("tests/samples/30-22.csv")]
#[case::case_30_23("tests/samples/30-23.csv")]
#[case::case_30_24("tests/samples/30-24.csv")]
#[case::case_30_25("tests/samples/30-25.csv")]
#[case::case_30_26("tests/samples/30-26.csv")]
#[case::case_30_27("tests/samples/30-27.csv")]
#[case::case_30_28("tests/samples/30-28.csv")]
#[case::case_30_29("tests/samples/30-29.csv")]
#[case::case_30_30("tests/samples/30-30.csv")]
#[case::case_30_31("tests/samples/30-31.csv")]
#[case::case_30_32("tests/samples/30-32.csv")]
#[case::case_30_33("tests/samples/30-33.csv")]
#[case::case_30_34("tests/samples/30-34.csv")]
#[case::case_30_35("tests/samples/30-35.csv")]
#[case::case_30_36("tests/samples/30-36.csv")]
#[case::case_30_37("tests/samples/30-37.csv")]
#[case::case_30_38("tests/samples/30-38.csv")]
#[case::case_30_39("tests/samples/30-39.csv")]
#[case::case_30_40("tests/samples/30-40.csv")]
#[case::case_30_41("tests/samples/30-41.csv")]
#[case::case_30_42("tests/samples/30-42.csv")]
#[case::case_30_43("tests/samples/30-43.csv")]
#[case::case_30_44("tests/samples/30-44.csv")]
#[case::case_30_45("tests/samples/30-45.csv")]
#[case::case_30_46("tests/samples/30-46.csv")]
#[case::case_30_47("tests/samples/30-47.csv")]
#[case::case_30_48("tests/samples/30-48.csv")]
#[case::close_to_minus_99("tests/samples/minus_99.csv")]
fn test_xirr_samples(#[case] input: &str) {
    let result = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_records();
        let rate = pyxirr::xirr(payments, None, None).unwrap();

        if let Some(rate) = rate {
            assert_almost_eq!(pyxirr::xnpv(rate, payments, None).unwrap().unwrap(), 0.0, 1e-3);
        }

        rate.unwrap_or(f64::NAN)
    });
    let expected = xirr_expected_result(input);

    if result.is_nan() {
        assert!(expected.is_nan());
    } else {
        assert_almost_eq!(result, expected);
    }
}

#[rstest]
fn test_xfv() {
    // http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-XFV-function
    Python::with_gil(|py| {
        let result = pyxirr::xfv(
            PyDate::new(py, 2011, 2, 1).unwrap().into(),
            PyDate::new(py, 2011, 3, 1).unwrap().into(),
            PyDate::new(py, 2012, 2, 1).unwrap().into(),
            0.00142,
            0.00246,
            100000.,
        )
        .unwrap()
        .unwrap();
        assert_almost_eq!(result, 100235.088391894);
    });
}

#[rstest]
fn test_xnfv() {
    Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, "tests/samples/xnfv.csv").to_records();
        let result = pyxirr::xnfv(0.0250, payments, None).unwrap().unwrap();
        assert_almost_eq!(result, 57238.1249299303);
    });
}

#[rstest]
fn test_sum_xfv_eq_xnfv() {
    Python::with_gil(|py| {
        let rate = 0.0250;
        let (dates, amounts) = PaymentsLoader::from_csv(py, "tests/samples/xnfv.csv").to_columns();

        let xnfv_result = pyxirr::xnfv(rate, dates, Some(amounts)).unwrap().unwrap();

        let builtins = py.import("builtins").unwrap();
        let locals = vec![
            ("dates", dates),
            ("min", builtins.getattr("min").unwrap()),
            ("max", builtins.getattr("max").unwrap()),
        ]
        .into_py_dict(py);

        let min_date = py.eval("min(dates)", Some(locals), None).unwrap();
        let max_date = py.eval("max(dates)", Some(locals), None).unwrap();
        let sum_xfv_result: f64 = dates
            .iter()
            .unwrap()
            .zip(amounts.iter().unwrap())
            .map(|(d, a)| {
                pyxirr::xfv(
                    min_date.extract().unwrap(),
                    d.unwrap().extract().unwrap(),
                    max_date.extract().unwrap(),
                    rate,
                    rate,
                    a.unwrap().extract().unwrap(),
                )
                .unwrap()
                .unwrap()
            })
            .sum();

        assert_almost_eq!(xnfv_result, sum_xfv_result);
    });
}
