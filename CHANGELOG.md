# Changelog

## [0.10.2] - 2024-01-27

- (X)IRR Performance improvements

## [0.10.1] - 2023-12-08

- XIRR improvements ([#49](https://github.com/Anexen/pyxirr/pull/49))
- Handle NAN in utility functions for multi-root analysis

## [0.10.0] - 2023-12-03

- Private Equity functions ([#42](https://github.com/Anexen/pyxirr/issues/42))
- XNPV/NPV functions now accept rate as multidimensional array
- Explain Multiple IRR problem + provide utility functions for analysis
- XIRR improvements (prevent from values < -1, grid search + brentq as fallback)

## [0.9.3] - 2023-10-11

- IRR improvements ([#47](https://github.com/Anexen/pyxirr/pull/47))
- Python 3.12 support

## [0.9.2] - 2023-06-14

- XNFV: suppress `InvalidPaymentsError` by passing `silent=True` flag ([#40](https://github.com/Anexen/pyxirr/issues/40))

## [0.9.1] - 2023-06-03

- CUMPRINC, CUMIPMT functions
- Upgrade maturin to v1

## [0.9.0] - 2023-03-12

- Vectorized versions of numpy-financial functions
- Breaking change: fixed misspelling in keyword-only argument (pmt_at_beginning)

## [0.8.1] - 2023-02-21

- Fix type annotations ([#35](https://github.com/Anexen/pyxirr/issues/35))

## [0.8.0] - 2023-02-20

- Add support for different day count conventions ([#34](https://github.com/Anexen/pyxirr/pull/34))
- Upgrade Rust libraries
- Upgrade maturin to v0.14.13

## [0.7.3] - 2022-11-05

- Upgrade Rust libraries
- Update package metadata

## [0.7.2] - 2022-01-28

- Tweaked IRR to prefer rate > 0 ([#24](https://github.com/Anexen/pyxirr/issues/24))
- All functions now accept date strings in the format yyyy-mm-dd or mm/dd/yyyy

## [0.7.1] - 2021-12-03

- handle XIRR close to -1 (use brentq algorithm as fallback)

## [0.7.0] - 2021-12-02

- Add an ability to suppress `InvalidPaymentsError` by passing `silent=True` flag ([#22](https://github.com/Anexen/pyxirr/issues/22))
- Release the GIL for rust-only code
- Type hints
- Refactor tests (use `PyCFunction` interface instead of calling functions directly)
- Upgrade dependencies

## [0.6.5] - 2021-11-16

- Support aarch64, armv7, s390x, ppc64le, ppc64 architectures
- Improve IRR calculation

## [0.6.4] - 2021-10-15

- Wheels for python 3.10
- Add Rate, IPMT, PPMT ([#18](https://github.com/Anexen/pyxirr/pull/18))
- Test against `numpy-financial` when possible

## [0.6.3] - 2021-08-17

- XIRR improvements ([#15](https://github.com/Anexen/pyxirr/pull/15))
- add more xirr tests
- handle XIRR close to -1
- fix nfv signature; always return None instead of nan

## [0.6.2] - 2021-08-06

- Support Series with DatetimeIndex ([#13](https://github.com/Anexen/pyxirr/pull/13))

## [0.6.1] - 2021-07-28

- Add NFV, XFV, XNFV ([#11](https://github.com/Anexen/pyxirr/pull/11))

## [0.6.0] - 2021-07-24

- Add XFV, PMT, NPER ([#8](https://github.com/Anexen/pyxirr/pull/8), [#9](https://github.com/Anexen/pyxirr/pull/9))

## [0.5.2] - 2021-06-04

- NPV compatibility mode with Excel
- XIRR optimizations
- Improve the docs

## [0.5.1] - 2021-05-25

- Remove pyo3 wrappers from core
- Benchmark: compare with `numpy-financial`

## [0.5.0] - 2021-05-24

- MIRR, FV
- Performance improvements ([#6](https://github.com/Anexen/pyxirr/pull/6))
- Test without numpy
- Setup Github Action for benchmark ([#5](https://github.com/Anexen/pyxirr/pull/5))

## [0.4.1] - 2021-05-20

- Add FV

## [0.4.0] - 2021-05-20

- Add IRR & NPV ([#4](https://github.com/Anexen/pyxirr/pull/4))
- Optimize cargo build profile for speed
- Setup Github actions for testing and publishing

## [0.3.1] - 2021-05-16

- Faster conversion from `numpy` arrays ([#3](https://github.com/Anexen/pyxirr/pull/3))

## [0.3.0] - 2021-05-11

- Simplify python conversions
- Refactor tests
- Numpy & Pandas support ([#2](https://github.com/Anexen/pyxirr/pull/2))

## [0.2.1] - 2021-05-07

- Support row-oriented input for xirr
- Add XNPV
- Faster XIRR implementation
