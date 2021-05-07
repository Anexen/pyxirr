[![rust-lang.org](https://img.shields.io/badge/Made%20with-Rust-red)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/Anexen/pyxirr.svg)](https://github.com/Anexen/pyxirr/blob/master/LICENSE.txt)
[![pypi](https://img.shields.io/pypi/v/pyxirr.svg)](https://pypi.org/project/pyxirr/)

# PyXIRR

Rust-powered collection of financial functions.

Features:
 - correct
 - blazingly fast
 - works with iterators
 - no external dependencies

# Installation

```
pip install pyxirr
```

# Benchmarks

Rust implementation has been tested against existing [xirr](https://pypi.org/project/xirr/) package
(uses [scipy.optimize](https://docs.scipy.org/doc/scipy/reference/generated/scipy.optimize.newton.html) under the hood)
and the [implementation from the Stack Overflow](https://stackoverflow.com/a/11503492) (pure python).

| Implementation | Sample size |  Execution time |
| -------------- | :---------: | --------------: |
| pyxirr (Rust)  |     100     |  **_45.89 us_** |
| xirr (scipy)   |     100     |       790.76 us |
| pure Python    |     100     |        14.37 ms |
| pyxirr (Rust)  |    1000     | **_404.03 us_** |
| xirr (scipy)   |    1000     |         3.47 ms |
| pure Python    |    1000     |        35.97 ms |
| pyxirr (Rust)  |    10000    |   **_3.58 ms_** |
| xirr (scipy)   |    10000    |        28.04 ms |
| pure Python    |    10000    |         24.23 s |

PyXIRR is ~10-20x faster than other solutions!

# Usage

## xirr

Function signature:

```python
# You have two options:
# 1. Two iterables for dates and amounts
# 2. Single iterable of tuples (date, amount)

DateLike = Union[datetime.date, datetime.datetime]
Amount = Union[int, float, Decimal]

def xirr(
    dates: Union[Iterable[DateLike], Iterable[Tuple[DateLike, Amount]]]
    amounts Optional[Iterable[Amount]] = None
    guess: Optional[float] = None
)
```

Example:

```python
from datetime import date
from pyxirr import xirr

dates = [date(2020, 1, 1), date(2020, 2, 1)]
amounts = [-100, 125]

xirr(dates, amounts)

# list of tuples is also possible:
xirr(zip(dates, amounts))
```

## xnpv

Function signature:

```python
# similar to xirr: iterable of tuples or two iterables

def xnpv(
    rate: float,
    dates: Union[Iterable[DateLike], Iterable[Tuple[DateLike, Amount]]]
    amounts Optional[Iterable[Amount]] = None
)

```

Example:

```python
from pyxirr import xnpv

xnpv(0.1, dates, amounts)
xnpv(0.1, zip(dates, amounts))
```

# Development

Running tests with pyo3 is a bit tricky. In short, you need to compile your tests without `extension-module` feature to avoid linking errors.
See the following issues for the details: [#341](https://github.com/PyO3/pyo3/issues/341), [#771](https://github.com/PyO3/pyo3/issues/771).

If you are using `pyenv`, make sure you have the shared library installed (check for `${PYENV_ROOT}/versions/<version>/lib/libpython3.so` file).

```bash
$ export PYO3_PYTHON_VERSION=3.8.6
$ PYTHON_CONFIGURE_OPTS="--enable-shared" pyenv install ${PYO3_PYTHON_VERSION}
```

```bash
# running tests
$ LD_LIBRARY_PATH=${PYENV_ROOT}/versions/${PYO3_PYTHON_VERSION}/lib cargo tests --no-default-features --features tests
# running benches
$ LD_LIBRARY_PATH=${PYENV_ROOT}/versions/${PYO3_PYTHON_VERSION}/lib cargo bench --no-default-features --features tests
```
