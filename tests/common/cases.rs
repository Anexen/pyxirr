#![allow(dead_code)]

pub fn irr_expected_result(input: &str) -> f64 {
    match input {
        "tests/samples/unordered.csv" => 0.7039842300,
        "tests/samples/random_100.csv" => 2.3320600601,
        "tests/samples/random_1000.csv" => 0.8607558299,
        "tests/samples/minus_0_993.csv" => -0.995697224362268,
        _ => panic!(),
    }
}
