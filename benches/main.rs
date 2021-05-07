use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDict, PyList};

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
            residual += ta[1] / pow(guess, years[i])
        if abs(residual) > epsilon:
            if residual > 0:
                guess += step
            else:
                guess -= step
                step /= 2.0
    return guess - 1
"#;

fn generate_payments<'a>(py: Python<'a>, sample_size: i64) -> &'a PyList {
    let dates =
        (0..sample_size).map(move |i| PyDate::from_timestamp(py, i * 24 * 60 * 60).unwrap());

    let data = dates.zip(-sample_size / 4..sample_size / 4 * 3).collect::<Vec<(&PyDate, i64)>>();
    return PyList::new(py, data);
}

pub fn benchmark(c: &mut Criterion) {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let xirr = py.import("xirr").expect("xirr is not installed").getattr("xirr").unwrap();
    let so_xirr = PyModule::from_code(py, TOP_STACK_OVERFLOW_ANSWER, "so_xirr.py", "so_xirr")
        .unwrap()
        .getattr("xirr")
        .unwrap();

    let mut group = c.benchmark_group("XIRR");

    for sample_size in &[100, 1000, 10000] {
        let data = generate_payments(py, *sample_size);
        let xdata = PyDict::from_sequence(py, data.into()).unwrap();

        group.bench_function(BenchmarkId::new("rust", sample_size), |b| {
            b.iter(|| pyxirr::xirr(py, black_box(data), black_box(None), black_box(None)))
        });

        group.bench_function(BenchmarkId::new("scipy", sample_size), |b| {
            b.iter(|| xirr.call1(black_box((xdata,))).unwrap())
        });

        group.bench_function(BenchmarkId::new("pure python", sample_size), |b| {
            b.iter(|| so_xirr.call1(black_box((data,))).unwrap())
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
