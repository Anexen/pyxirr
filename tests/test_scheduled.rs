use pyo3::{
    types::{PyDate, PyList},
    IntoPy, PyResult, Python,
};
use rstest::rstest;

mod common;
use common::PaymentsLoader;

#[rstest]
#[case::unordered("tests/samples/unordered.csv", 2218.42566365675)]
#[case::random_100("tests/samples/random_100.csv", 6488.0382272781)]
#[case::random_1000("tests/samples/random_1000.csv", 41169.6659983284)]
fn test_xnpv_samples(#[case] input: &str, #[case] expected: f64) {
    let rate = 0.1;
    let result: f64 = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_records();
        pyxirr_call!(py, "xnpv", (rate, payments))
    });
    assert_almost_eq!(result, expected);
}

#[rstest]
#[case::unordered("tests/samples/unordered.csv", 0.16353715844)]
#[case::single_redemption("tests/samples/single_redemption.csv", 0.13616957937417506)]
#[case::random("tests/samples/random.csv", 0.6924974337277426)]
#[case::random_100("tests/samples/random_100.csv", 29.829404437653)]
#[case::random_1000("tests/samples/random_1000.csv", 5.508930558032)]
#[case::case_30_0("tests/samples/30-0.csv", 0.1660454339589889)]
#[case::case_30_1("tests/samples/30-1.csv", 0.18180763138335373)]
#[case::case_30_2("tests/samples/30-2.csv", -0.0027489547855574564)]
#[case::case_30_3("tests/samples/30-3.csv", 5.852451769434373)]
#[case::case_30_4("tests/samples/30-4.csv", 0.16098047379438984)]
#[case::case_30_5("tests/samples/30-5.csv", 0.008979287890185613)]
#[case::case_30_6("tests/samples/30-6.csv", 0.3255467341810659)]
#[case::case_30_7("tests/samples/30-7.csv", 0.3501464865493174)]
#[case::case_30_8("tests/samples/30-8.csv", 3.353029509425298)]
#[case::case_30_9("tests/samples/30-9.csv", 2.878013825163697)]
#[case::case_30_10("tests/samples/30-10.csv", 0.11143674454788119)]
#[case::case_30_11("tests/samples/30-11.csv", -0.12606921657689435)]
#[case::case_30_12("tests/samples/30-12.csv", -0.02578630164755525)]
#[case::case_30_13("tests/samples/30-13.csv", -0.6590570693637554)]  // -0.02910731236366771
#[case::case_30_14("tests/samples/30-14.csv", 0.6996860198137344)]
#[case::case_30_15("tests/samples/30-15.csv", 0.02976853488940409)]
#[case::case_30_16("tests/samples/30-16.csv", 0.44203743561153225)]
#[case::case_30_17("tests/samples/30-17.csv", 2.7956075643765765)]
#[case::case_30_18("tests/samples/30-18.csv", -0.2692266976014054)]
#[case::case_30_19("tests/samples/30-19.csv", -0.0016474932646633118)]
#[case::case_30_20("tests/samples/30-20.csv", f64::NAN)]
#[case::case_30_21("tests/samples/30-21.csv", 0.05900900202336096)]
#[case::case_30_22("tests/samples/30-22.csv", -0.028668460065440993)] // -0.3154082674273421
#[case::case_30_23("tests/samples/30-23.csv", 1.1276768367328942)]
#[case::case_30_24("tests/samples/30-24.csv", 32.90894421344702)]
#[case::case_30_25("tests/samples/30-25.csv", -0.001245880387491199)]
#[case::case_30_26("tests/samples/30-26.csv", -0.33228389267806224)]
#[case::case_30_27("tests/samples/30-27.csv", 0.00017475536849502265)]
#[case::case_30_28("tests/samples/30-28.csv", -0.10396735360664396)] // 1.1258287638216773
#[case::case_30_29("tests/samples/30-29.csv", f64::NAN)]
#[case::case_30_30("tests/samples/30-30.csv", 0.08115488395163964)]
#[case::case_30_31("tests/samples/30-31.csv", f64::NAN)]
#[case::case_30_32("tests/samples/30-32.csv", -0.1305850720162)]
#[case::case_30_33("tests/samples/30-33.csv", f64::NAN)]
#[case::case_30_34("tests/samples/30-34.csv", f64::NAN)]
#[case::case_30_35("tests/samples/30-35.csv", -0.23061428300107065)]
#[case::case_30_36("tests/samples/30-36.csv", -0.09610929159865819)]
#[case::case_30_37("tests/samples/30-37.csv", -0.17219174455291367)] // -0.6519313380797903
#[case::case_30_38("tests/samples/30-38.csv", f64::NAN)]
#[case::case_30_39("tests/samples/30-39.csv", -0.202699788567102)]
#[case::case_30_40("tests/samples/30-40.csv", f64::NAN)]
#[case::case_30_41("tests/samples/30-41.csv", -0.11644766662933352)]
#[case::case_30_42("tests/samples/30-42.csv", f64::NAN)]
#[case::case_30_43("tests/samples/30-43.csv", -0.12837518345271245)]
#[case::case_30_44("tests/samples/30-44.csv", f64::NAN)]
#[case::case_30_45("tests/samples/30-45.csv", f64::NAN)]
#[case::case_30_46("tests/samples/30-46.csv", -0.047401670775621726)]
#[case::case_30_47("tests/samples/30-47.csv", -0.6103425929117927)]
#[case::case_30_48("tests/samples/30-48.csv", -0.07525261340272364)]
#[case::close_to_minus_0_13("tests/samples/minus_0_13.csv", -0.13423264098831872)]
#[case::close_to_minus_0_99("tests/samples/minus_0_99.csv", -0.9989769231734277)]
#[case::close_to_minus_0_99999("tests/samples/minus_0_99999.csv", -0.9999884228170087)]
#[case::close_to_minus_0_993("tests/samples/minus_0_993.csv", -0.993785049929284)]
#[case::zeros("tests/samples/zeros.csv", 0.175680730580782)]
#[case::neg_1938("tests/samples/1938.csv", -0.5945650822679239)]
fn test_xirr_samples(#[case] input: &str, #[case] expected: f64) {
    let result = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_records();
        let rate: Option<f64> = pyxirr_call!(py, "xirr", (payments,));

        if let Some(rate) = rate {
            let xnpv: f64 = pyxirr_call!(py, "xnpv", (rate, payments));
            assert_almost_eq!(xnpv, 0.0, 1e-3);
        }

        rate.unwrap_or(f64::NAN)
    });

    if result.is_nan() {
        assert!(expected.is_nan(), "assertion failed: expected {expected}, found NaN");
    } else {
        assert_almost_eq!(result, expected);
    }
}

#[rstest]
fn test_xirr_silent() {
    Python::with_gil(|py| {
        let args = (PyList::empty(py), PyList::empty(py));
        let err = pyxirr_call_impl!(py, "xirr", args).unwrap_err();
        assert!(err.is_instance(py, py.get_type::<pyxirr::InvalidPaymentsError>()));

        let result: Option<f64> = pyxirr_call!(py, "xirr", args, py_dict!(py, "silent" => true));
        assert!(result.is_none());
    })
}

#[rstest]
fn test_xfv() {
    // http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-XFV-function
    Python::with_gil(|py| {
        let args = (
            PyDate::new(py, 2011, 2, 1).unwrap(),
            PyDate::new(py, 2011, 3, 1).unwrap(),
            PyDate::new(py, 2012, 2, 1).unwrap(),
            0.00142,
            0.00246,
            100000.,
        );
        let result: f64 = pyxirr_call!(py, "xfv", args);
        assert_almost_eq!(result, 100235.088391894);
    });
}

#[rstest]
fn test_xnfv() {
    Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, "tests/samples/xnfv.csv").to_records();
        let result: f64 = pyxirr_call!(py, "xnfv", (0.0250, payments));
        assert_almost_eq!(result, 57238.1249299303);
    });
}

#[rstest]
fn test_xnfv_silent() {
    Python::with_gil(|py| {
        let dates = vec!["2021-01-01", "2022-01-01"].into_py(py);
        let amounts = vec![1000, 100].into_py(py);
        let args = (0.0250, dates, amounts);
        let kwargs = py_dict!(py);
        let result: PyResult<_> = pyxirr_call_impl!(py, "xnfv", args.clone(), kwargs);

        assert!(result.is_err());
        let kwargs = py_dict!(py, "silent" => true);
        let result: Option<f64> = pyxirr_call!(py, "xnfv", args, kwargs);
        assert!(result.is_none());
    });
}

#[rstest]
fn test_sum_xfv_eq_xnfv() {
    Python::with_gil(|py| {
        let rate = 0.0250;
        let (dates, amounts) = PaymentsLoader::from_csv(py, "tests/samples/xnfv.csv").to_columns();

        let xnfv_result: f64 = pyxirr_call!(py, "xnfv", (rate, dates, amounts));

        let builtins = py.import("builtins").unwrap();
        let locals = py_dict!(py, "dates" => dates);
        let min_date = py.eval("min(dates)", Some(locals), Some(builtins.dict())).unwrap();
        let max_date = py.eval("max(dates)", Some(locals), Some(builtins.dict())).unwrap();

        let sum_xfv_result: f64 = dates
            .iter()
            .unwrap()
            .map(Result::unwrap)
            .zip(amounts.iter().unwrap().map(Result::unwrap))
            .map(|(date, amount)| -> f64 {
                pyxirr_call!(py, "xfv", (min_date, date, max_date, rate, rate, amount))
            })
            .sum();

        assert_almost_eq!(xnfv_result, sum_xfv_result);
    });
}

// https://www.mathworks.com/help/finance/xirr.html
#[rstest]
#[case("30/360 SIA", 0.100675477282743)] // 1
#[case("act/360", 0.0991988898057063)] // 2
#[case("act/365F", 0.10064378342638)] // 3
#[case("30/360 ISDA", 0.100675477282743)] // 5
#[case("30E/360", 0.100675477282743)] // 6
#[case("act/act ISDA", 0.100739648987346)] // 12
fn test_xirr_day_count(#[case] day_count: &str, #[case] expected: f64) {
    Python::with_gil(|py| {
        let dates = ["01/12/2007", "02/14/2008", "03/03/2008", "06/14/2008", "12/01/2008"];
        let amounts = [-10000, 2500, 2000, 3000, 4000];

        let kwargs = py_dict!(py, "day_count" => day_count);
        let value: f64 = pyxirr_call!(py, "xirr", (dates, amounts), kwargs);

        assert_almost_eq!(value, expected);
    })
}
