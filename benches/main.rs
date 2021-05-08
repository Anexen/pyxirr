use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use pyxirr;

// https://stackoverflow.com/questions/8919718/financial-python-library-that-has-xirr-and-xnpv-function
const TOP_STACK_OVERFLOW_ANSWER: &str = r#"
def xirr(transactions):
    years = [(ta[0] - transactions[0][0]).days / 365.0 for ta in transactions]
    residual = 1
    step = 0.05
    guess = 0.05
    epsilon = 0.0001
    limit = 10000
    while abs(residual) > epsilon and limit > 0:
        limit -= 1
        residual = 0.0
        for i, ta in enumerate(transactions):
            residual += ta[1] / pow(guess + 1, years[i])
        if abs(residual) > epsilon:
            if residual > 0:
                guess += step
            else:
                guess -= step
                step /= 2.0
    return guess
"#;

const READ_CSV: &str = r#"
import csv
from datetime import datetime

def read_csv(name):
    with open(name, newline='') as csvfile:
        reader = csv.reader(csvfile, delimiter=',')
        return [
            (datetime.strptime(date, "%Y-%m-%d"), float(amount))
            for date, amount in reader
        ]
"#;

pub fn benchmark(c: &mut Criterion) {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let xirr = py.import("xirr").expect("xirr is not installed").getattr("xirr").unwrap();
    let so_xirr = PyModule::from_code(py, TOP_STACK_OVERFLOW_ANSWER, "so_xirr.py", "so_xirr")
        .unwrap()
        .getattr("xirr")
        .unwrap();

    let read_csv = PyModule::from_code(py, READ_CSV, "read_csv.py", "read_csv")
        .unwrap()
        .getattr("read_csv")
        .unwrap();

    let mut group = c.benchmark_group("XIRR");

    for sample_size in &[100, 1000, 10000] {
        let data = read_csv.call1((format!("tests/samples/random_{}.csv", sample_size),)).unwrap();
        let xdata = PyDict::from_sequence(py, data.into()).unwrap();

        group.sample_size(128).bench_function(BenchmarkId::new("rust", sample_size), |b| {
            b.iter(|| pyxirr::xirr(black_box(data), black_box(None), black_box(None)))
        });

        group.sample_size(64).bench_function(BenchmarkId::new("scipy", sample_size), |b| {
            b.iter(|| xirr.call1(black_box((xdata,))))
        });

        if *sample_size != 10000 {
            group
                .sample_size(16)
                .bench_function(BenchmarkId::new("pure python", sample_size), |b| {
                    b.iter(|| so_xirr.call1(black_box((data,))))
                });
        }
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
