import sys
from datetime import date, datetime
from decimal import Decimal
from typing import (
    Any,
    Dict,
    Iterable,
    Optional,
    Tuple,
    Union,
    overload,
    Hashable,
    Literal,
)

if sys.version_info >= (3, 8):
    from typing import Protocol
else:
    from typing_extensions import Protocol

# We are using protocols because mypy does not support dynamic type hints for
# optional libraries, e.g in the ideal world:
#    _DateLike = Union[date, datetime] if pandas is not installed
#    _DateLike = Union[date, datetime, pandas.Timestamp] if pandas is installed
# but it is impossible to archive.

# fmt: off

_Shape = Tuple[int, ...]
_OrderKACF = Optional[Literal["K", "A", "C", "F"]]

class _ndarray(Protocol):
    @property
    def ndim(self) -> int: ...
    @property
    def size(self) -> int: ...
    @property
    def shape(self) -> _Shape: ...
    @property
    def strides(self) -> _Shape: ...
    @property
    def dtype(self) -> Any: ...
    def flatten( self, order: _OrderKACF = ...) -> "_ndarray": ...
    def ravel( self, order: _OrderKACF = ...,) -> "_ndarray": ...
    def fill(self, value: Any) -> None: ...

class _DatetimeScalar(Protocol):
    @property
    def day(self) -> int: ...
    @property
    def month(self) -> int: ...
    @property
    def year(self) -> int: ...

class _datetime64(Protocol):
    def __init__(
        self,
        value: Union[None, int , str, "_datetime64", _DatetimeScalar] = ...,
        format: Union[str, Tuple[str, int]] = ...,
    ) -> None: ...

# define some specific methods just to recognize Series/DataFrame
# https://github.com/VirtusLab/pandas-stubs

_Label = Optional[Hashable]

class _Series(Protocol):
    @property
    def hasnans(self) -> bool: ...
    def items(self) -> Iterable[Any]: ...
    def iteritems(self) -> Iterable[Any]: ...
    def to_frame(self, name: Optional[Any] = ...) -> "_DataFrame": ...

class _DataFrame(Protocol):
    @property
    def shape(self) -> Tuple[int, int]: ...
    def items(self) -> Iterable[Tuple[_Label, _Series]]: ...
    def iteritems(self) -> Iterable[Tuple[_Label, _Series]]: ...
    def iterrows(self) -> Iterable[Tuple[_Label, _Series]]: ...
    def itertuples( self, index: bool = ..., name: Optional[str] = ...) -> Iterable[Tuple[Any, ...]]: ...
    def assign(self, **kwargs: Any) -> "_DataFrame": ...

class _Timestamp(Protocol):
    def isoformat(self, sep: str = ...) -> str: ...
    def day_name(self, locale: Optional[str]) -> str: ...
    def month_name(self, locale: Optional[str]) -> str: ...
    def normalize(self) -> "_Timestamp": ...
    @property
    def freqstr(self) -> str: ...
    @property
    def is_month_end(self) -> bool: ...
    @property
    def is_month_start(self) -> bool: ...
    @property
    def is_quarter_start(self) -> bool: ...
    @property
    def is_quarter_end(self) -> bool: ...
    @property
    def is_year_start(self) -> bool: ...
    @property
    def is_year_end(self) -> bool: ...
    @property
    def is_leap_year(self) -> bool: ...

# fmt: on

# rate as decimal, not percentage, normally between [-1, 1]
_Rate = Union[float, Decimal]
_Period = Union[int, float, Decimal]
_Guess = Optional[_Rate]
_Amount = Union[int, float, Decimal]

_DateLike = Union[str, date, datetime, _datetime64, _Timestamp]
_Payment = Tuple[_DateLike, _Amount]
_CashFlowTable = Union[Iterable[_Payment], _DataFrame, _ndarray]
_CashFlowDict = Dict[_DateLike, _Amount]
_CashFlow = Union[_CashFlowTable, _CashFlowDict, _Series]

_DateLikeArray = Iterable[_DateLike]
_AmountArray = Iterable[_Amount]


def fv(
    rate: _Rate,
    nper: _Period,
    pmt: _Amount,
    pv: _Amount,
    *,
    pmt_at_begining: bool = False,
) -> Optional[float]:
    ...


def nfv(
    rate: _Rate,  # Rate of interest per period
    nper: _Period,  # Number of compounding periods
    amounts: _AmountArray,
) -> Optional[float]:
    ...


def xfv(
    start_date: _DateLike,
    cash_flow_date: _DateLike,
    end_date: _DateLike,
    cash_flow_rate: _Rate,  # annual rate
    end_rate: _Rate,  # annual rate
    cash_flow: _Amount,
) -> Optional[float]:
    ...


@overload
def xnfv(
    rate: _Rate,  # annual rate
    dates: _CashFlow,
) -> Optional[float]:
    ...


@overload
def xnfv(
    rate: _Rate,  # annual rate
    dates: _DateLikeArray,
    amounts: _AmountArray,
) -> Optional[float]:
    ...


def pv(
    rate: _Rate,
    nper: _Period,
    pmt: _Amount,
    fv: _Amount,
    *,
    pmt_at_begining: bool = False,
) -> Optional[float]:
    ...


def npv(
    rate: _Rate,
    amounts: _AmountArray,
    *,
    start_from_zero: bool = True,
) -> Optional[float]:
    ...


@overload
def xnpv(
    rate: _Rate,
    dates: _CashFlow,
    *,
    silent: bool = False,
) -> Optional[float]:
    ...


@overload
def xnpv(
    rate: _Rate,
    dates: _DateLikeArray,
    amounts: _AmountArray,
    *,
    silent: bool = False,
) -> Optional[float]:
    ...


def rate(
    nper: _Period,
    pmt: _Amount,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_begining: bool = False,
    guess: _Guess = 0.1,
):
    ...


def nper(
    rate: _Rate,
    pmt: _Amount,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_begining: bool = False,
) -> Optional[float]:
    ...


def pmt(
    rate: _Rate,
    nper: _Period,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_begining: bool = False,
) -> Optional[float]:
    ...


def ipmt(
    rate: _Rate,
    per: _Period,
    nper: _Period,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_begining: bool = False,
) -> Optional[float]:
    ...


def ppmt(
    rate: _Rate,
    per: _Period,
    nper: _Period,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_begining: bool = False,
) -> Optional[float]:
    ...


def irr(
    amounts: _AmountArray,
    *,
    guess: _Guess = 0.1,
    silent: bool = False,
) -> Optional[float]:
    ...


def mirr(
    amounts: _AmountArray,
    finance_rate: _Rate,
    reinvest_rate: _Rate,
    *,
    silent: bool = False,
) -> Optional[float]:
    ...


@overload
def xirr(
    dates: _DateLikeArray,
    amounts: _AmountArray,
    *,
    guess: _Guess = 0.1,
    silent: bool = False,
) -> Optional[float]:
    ...


@overload
def xirr(
    dates: _CashFlow,
    *,
    guess: _Guess = 0.1,
    silent: bool = False,
) -> Optional[float]:
    ...
