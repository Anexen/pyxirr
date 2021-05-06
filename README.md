# PyXIRR

Rust-powered collection of financial functions.

# Installation

```
pip install pyxirr
```

# Usage

## xirr

Function signature:

```python
# You have two options:
# 1. Two iterables for dates and amounts
# 2. Single iterable of tuples (date, amount)

DateLike = Union[datetime.date, datetime.datetime]
Amount = Union[int, foat, Decimal]

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
