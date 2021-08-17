#![allow(dead_code)]

pub fn xirr_expected_result(input: &str) -> f64 {
    match input {
        "tests/samples/unordered.csv" => 0.16353715844,
        "tests/samples/single_redemption.csv" => 0.13616957937417506,
        "tests/samples/random.csv" => 0.6924974337277426,
        "tests/samples/random_100.csv" => 29.829404437653,
        "tests/samples/random_1000.csv" => 5.508930558032,
        "tests/samples/30-0.csv" => 0.1660454339589889,
        "tests/samples/30-1.csv" => 0.18180763138335373,
        "tests/samples/30-2.csv" => -0.0027489547855574564,
        "tests/samples/30-3.csv" => 5.852451769434373,
        "tests/samples/30-4.csv" => 0.16098047379438984,
        "tests/samples/30-5.csv" => 0.008979287890185613,
        "tests/samples/30-6.csv" => 0.3255467341810659,
        "tests/samples/30-7.csv" => 0.3501464865493174,
        "tests/samples/30-8.csv" => 3.353029509425298,
        "tests/samples/30-9.csv" => 2.878013825163697,
        "tests/samples/30-10.csv" => 0.11143674454788119,
        "tests/samples/30-11.csv" => 1.5121634950488045,
        "tests/samples/30-12.csv" => -0.02578630164755525,
        "tests/samples/30-13.csv" => -0.6590570693637554,
        "tests/samples/30-14.csv" => 0.6996860198137344,
        "tests/samples/30-15.csv" => 0.02976853488940409,
        "tests/samples/30-16.csv" => 0.44203743561153225,
        "tests/samples/30-17.csv" => 2.7956075643765765,
        "tests/samples/30-18.csv" => 0.2593570136721243,
        "tests/samples/30-19.csv" => -0.0016474932646633118,
        "tests/samples/30-20.csv" => f64::NAN,
        "tests/samples/30-21.csv" => 0.05900900202336096,
        "tests/samples/30-22.csv" => -0.3154082674273421,
        "tests/samples/30-23.csv" => 1.1276768367328942,
        "tests/samples/30-24.csv" => 32.90894421344702,
        "tests/samples/30-25.csv" => -0.001245880387491199,
        "tests/samples/30-26.csv" => -0.33228389267806224,
        "tests/samples/30-27.csv" => 0.00017475536849502265,
        "tests/samples/30-28.csv" => 1.1258287638216773,
        "tests/samples/30-29.csv" => f64::NAN,
        "tests/samples/30-30.csv" => 0.08115488395163964,
        "tests/samples/30-31.csv" => f64::NAN,
        "tests/samples/30-32.csv" => -0.7080586361225928,
        "tests/samples/30-33.csv" => f64::NAN,
        "tests/samples/30-34.csv" => f64::NAN,
        "tests/samples/30-35.csv" => -0.23061428300107065,
        "tests/samples/30-36.csv" => -0.09610929159865819,
        "tests/samples/30-37.csv" => -0.6519313380797903,
        "tests/samples/30-38.csv" => f64::NAN,
        "tests/samples/30-39.csv" => -0.202699788567102,
        "tests/samples/30-40.csv" => f64::NAN,
        "tests/samples/30-41.csv" => -0.11644766662933352,
        "tests/samples/30-42.csv" => f64::NAN,
        "tests/samples/30-43.csv" => -0.12837518345271245,
        "tests/samples/30-44.csv" => f64::NAN,
        "tests/samples/30-45.csv" => f64::NAN,
        "tests/samples/30-46.csv" => -0.047401670775621726,
        "tests/samples/30-47.csv" => -0.6103425929117927,
        "tests/samples/30-48.csv" => -0.07525261340272364,
        "tests/samples/minus_99.csv" => -0.9989769231734277,
        _ => panic!(),
    }
}

pub fn xnpv_expected_result(rate: f64, input: &str) -> f64 {
    // rate is in range [-100, 100] because
    // floating-point types cannot be used in patterns
    let rate = (rate * 100.) as i8;
    match (rate, input) {
        (10, "tests/samples/unordered.csv") => 2218.42566365675,
        (10, "tests/samples/random_100.csv") => 6488.0382272781,
        (10, "tests/samples/random_1000.csv") => 41169.6659983284,
        _ => panic!(),
    }
}

pub fn irr_expected_result(input: &str) -> f64 {
    match input {
        "tests/samples/unordered.csv" => 0.7039842300,
        "tests/samples/random_100.csv" => 2.3320600601,
        "tests/samples/random_1000.csv" => 0.8607558299,
        _ => panic!(),
    }
}
