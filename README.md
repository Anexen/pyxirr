[![rust-lang.org](https://img.shields.io/badge/Made%20with-Rust-red)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/Anexen/pyxirr.svg)](https://github.com/Anexen/pyxirr/blob/master/LICENSE)
[![pypi](https://img.shields.io/pypi/v/pyxirr.svg)](https://pypi.org/project/pyxirr/)
[![versions](https://img.shields.io/pypi/pyversions/pyxirr.svg)](https://pypi.org/project/pyxirr/)

# PyXIRR

Rust-powered collection of financial functions.

Features:

- correct
- blazingly fast
- works with iterators
- works with unordered input
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

# Examples

```python
from datetime import date
from pyxirr import xirr

dates = [date(2020, 1, 1), date(2021, 1, 1), date(2022, 1, 1)]
amounts = [-1000, 1000, 1000]

# feed columnar data
xirr(dates, amounts)
# feed iterators
xirr(iter(dates), (x for x in amounts))
# feed an iterable of tuples
xirr(zip(dates, amounts))
# feed a dictionary
xirr(dict(zip(dates, amounts)))
```

Numpy and Pandas support

```python
import numpy as np
import pandas as pd

# feed numpy array
xirr(np.array([dates, amounts]))
xirr(np.array(dates), np.array(amounts))
# feed DataFrame (columns names doesn't matter; ordering matters)
xirr(pd.DataFrame({"a": dates, "b": amounts}))
```

# API reference

Let's define type annotations:

```python
DateLike = Union[datetime.date, datetime.datetime, numpy.datetime64, pandas.Timestamp]
Amount = Union[int, float, Decimal]
Payment = Tuple[DateLike, Amount]

DateLikeArray = Iterable[DateLike]
AmountArray = Iterable[Amount]
CashFlowTable = Iterable[Payment]
CashFlowDict = Dict[DateLike, Amount]

```

## XIRR

```python
def xirr(
    dates: Union[CashFlowTable, CashFlowDict, DateLikeArray]
    amounts: Optional[AmountArray] = None
    guess: Optional[float] = None,
)

```

## XNPV

```python

def xnpv(
    rate: float,
    dates: Union[CashFlowTable, CashFlowDict, DateLikeArray]
    amounts: Optional[AmountArray] = None
)

```

# Roadmap

- [x] NumPy support
- [x] XIRR
- [x] XNPV
- [ ] FV
- [ ] PV
- [ ] NPV
- [ ] IRR
- [ ] MIRR

# Development

Running tests with pyo3 is a bit tricky. In short, you need to compile your tests without `extension-module` feature to avoid linking errors.
See the following issues for the details: [#341](https://github.com/PyO3/pyo3/issues/341), [#771](https://github.com/PyO3/pyo3/issues/771).

If you are using `pyenv`, make sure you have the shared library installed (check for `${PYENV_ROOT}/versions/<version>/lib/libpython3.so` file).

```bash
$ export PYO3_PYTHON_VERSION=3.8.6
$ PYTHON_CONFIGURE_OPTS="--enable-shared" pyenv install ${PYO3_PYTHON_VERSION}
```

Install dev-requirements

```bash
$ pip install -r dev-requirements.txt
```

### Building

```bash
$ matirun develop
```

### Testing

```bash
$ LD_LIBRARY_PATH=${PYENV_ROOT}/versions/${PYO3_PYTHON_VERSION}/lib cargo tests --no-default-features --features tests
```

# Building and distribution

This library uses [matirun](https://github.com/PyO3/maturin) to build and distribute python wheels.

```bash
$ docker run --rm -v $(pwd):/io konstin2/maturin build --release --manylinux 2010 --strip
$ docker run --rm -v $(pwd):/io konstin2/maturin upload target/wheels/pyxirr-<version>*
```
