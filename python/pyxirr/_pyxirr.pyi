import sys
from collections.abc import Iterable, Sequence
from datetime import date, datetime
from decimal import Decimal
from typing import (
    Any,
    Dict,
    Hashable,
    List,
    Optional,
    Tuple,
    TypeVar,
    Union,
    overload,
)


if sys.version_info >= (3, 8):
    from typing import Literal, Protocol
else:
    from typing_extensions import Literal, Protocol

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
_DayCount = Union["DayCount" | str]

_DateLike = Union[str, date, datetime, _datetime64, _Timestamp]
_Payment = Tuple[_DateLike, _Amount]
_CashFlowTable = Union[Iterable[_Payment], _DataFrame, _ndarray]
_CashFlowDict = Dict[_DateLike, _Amount]
_CashFlow = Union[_CashFlowTable, _CashFlowDict, _Series]

_DateLikeArray = Iterable[_DateLike]
_AmountArray = Iterable[_Amount]

_T = TypeVar("_T")
_ArrayLike = Union[
    _ndarray,
    Sequence[_T],
    Sequence[Sequence[_T]],
    Sequence[Sequence[Sequence[_T]]],
    Sequence[Sequence[Sequence[Sequence[_T]]]],
    Sequence[Sequence[Sequence[Sequence[Sequence[_T]]]]],
    Sequence[Sequence[Sequence[Sequence[Sequence[Sequence[_T]]]]]],
    Sequence[Sequence[Sequence[Sequence[Sequence[Sequence[Sequence[_T]]]]]]],
    Sequence[
        Sequence[
            Sequence[Sequence[Sequence[Sequence[Sequence[Sequence[_T]]]]]]
        ]
    ],
]
_ScalarOrArrayLike = Union[_T, _ArrayLike[_T]]


class InvalidPaymentsError(Exception):
    pass


class BroadcastingError(Exception):
    pass


class DayCount:
    ACT_ACT_ISDA: "DayCount"
    ACT_365F: "DayCount"
    ACT_365_25: "DayCount"
    ACT_364: "DayCount"
    ACT_360: "DayCount"
    THIRTY_360_ISDA: "DayCount"
    THIRTY_E_360: "DayCount"
    THIRTY_E_PLUS_360: "DayCount"
    THIRTY_E_360_ISDA: "DayCount"
    THIRTY_U_360: "DayCount"
    NL_365: "DayCount"
    NL_360: "DayCount"

    @staticmethod
    def of(day_count: str) -> "DayCount":
        ...


def year_fraction(d1: _DateLike, d2: _DateLike, day_count: _DayCount) -> float:
    ...


def days_between(d1: _DateLike, d2: _DateLike, day_count: _DayCount) -> int:
    ...


@overload
def fv(  # type: ignore[misc]
    rate: _Rate,
    nper: _Period,
    pmt: _Amount,
    pv: _Amount,
    *,
    pmt_at_beginning: bool = False,
) -> Optional[float]:
    ...


@overload
def fv(
    rate: _ScalarOrArrayLike[_Rate],
    nper: _ScalarOrArrayLike[_Period],
    pmt: _ScalarOrArrayLike[_Amount],
    pv: _ScalarOrArrayLike[_Amount],
    *,
    pmt_at_beginning: _ScalarOrArrayLike[bool] = False,
) -> List[Optional[float]]:
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
    *,
    silent: bool = False,
    day_count: _DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...


@overload
def xnfv(
    rate: _Rate,  # annual rate
    dates: _DateLikeArray,
    amounts: _AmountArray,
    *,
    silent: bool = False,
    day_count: _DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...


@overload
def pv(  # type: ignore[misc]
    rate: _Rate,
    nper: _Period,
    pmt: _Amount,
    fv: _Amount,
    *,
    pmt_at_beginning: bool = False,
) -> Optional[float]:
    ...


@overload
def pv(
    rate: _ScalarOrArrayLike[_Rate],
    nper: _ScalarOrArrayLike[_Period],
    pmt: _ScalarOrArrayLike[_Amount],
    fv: _ScalarOrArrayLike[_Amount],
    *,
    pmt_at_beginning: _ScalarOrArrayLike[bool] = False,
) -> List[Optional[float]]:
    ...


@overload
def npv(
    rate: _Rate,
    amounts: _AmountArray,
    *,
    start_from_zero: bool = True,
) -> Optional[float]:
    ...


@overload
def npv(
    rate: Iterable[_Rate],
    amounts: _AmountArray,
    *,
    start_from_zero: bool = True,
) -> List[Optional[float]]:
    ...


@overload
def xnpv(
    rate: _Rate,
    dates: _CashFlow,
    *,
    silent: bool = False,
    day_count: _DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...


@overload
def xnpv(
    rate: _Rate,
    dates: _DateLikeArray,
    amounts: _AmountArray,
    *,
    silent: bool = False,
    day_count: _DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...


@overload
def xnpv(
    rate: Iterable[_Rate],
    dates: _CashFlow,
    *,
    silent: bool = False,
    day_count: _DayCount = DayCount.ACT_365F,
) -> List[Optional[float]]:
    ...


@overload
def rate(  # type: ignore[misc]
    nper: _Period,
    pmt: _Amount,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_beginning: bool = False,
    guess: _Guess = 0.1,
) -> Optional[float]:
    ...


@overload
def rate(
    nper: _ScalarOrArrayLike[_Period],
    pmt: _ScalarOrArrayLike[_Amount],
    pv: _ScalarOrArrayLike[_Amount],
    fv: _ScalarOrArrayLike[_Amount] = 0,
    *,
    pmt_at_beginning: _ScalarOrArrayLike[bool] = False,
    guess: _Guess = 0.1,
) -> List[Optional[float]]:
    ...


@overload
def nper(  # type: ignore[misc]
    rate: _Rate,
    pmt: _Amount,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_beginning: bool = False,
) -> Optional[float]:
    ...


@overload
def nper(
    rate: _ScalarOrArrayLike[_Rate],
    pmt: _ScalarOrArrayLike[_Amount],
    pv: _ScalarOrArrayLike[_Amount],
    fv: _ScalarOrArrayLike[_Amount] = 0,
    *,
    pmt_at_beginning: _ScalarOrArrayLike[bool] = False,
) -> List[Optional[float]]:
    ...


@overload
def pmt(  # type: ignore[misc]
    rate: _Rate,
    nper: _Period,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_beginning: bool = False,
) -> Optional[float]:
    ...


@overload
def pmt(
    rate: _ScalarOrArrayLike[_Rate],
    nper: _ScalarOrArrayLike[_Period],
    pv: _ScalarOrArrayLike[_Amount],
    fv: _ScalarOrArrayLike[_Amount] = 0,
    *,
    pmt_at_beginning: _ScalarOrArrayLike[bool] = False,
) -> List[Optional[float]]:
    ...


@overload
def ipmt(  # type: ignore[misc]
    rate: _Rate,
    per: _Period,
    nper: _Period,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_beginning: bool = False,
) -> Optional[float]:
    ...


@overload
def ipmt(
    rate: _ScalarOrArrayLike[_Rate],
    per: _ScalarOrArrayLike[_Period],
    nper: _ScalarOrArrayLike[_Period],
    pv: _ScalarOrArrayLike[_Amount],
    fv: _ScalarOrArrayLike[_Amount] = 0,
    *,
    pmt_at_beginning: _ScalarOrArrayLike[bool] = False,
) -> List[Optional[float]]:
    ...


def cumipmt(
    rate: _Rate,
    nper: _Period,
    pv: _Amount,
    start_period: _Period,
    end_period: _Period,
    *,
    pmt_at_beginning: bool = False,
) -> Optional[float]:
    ...


@overload
def ppmt(  # type: ignore[misc]
    rate: _Rate,
    per: _Period,
    nper: _Period,
    pv: _Amount,
    fv: _Amount = 0,
    *,
    pmt_at_beginning: bool = False,
) -> Optional[float]:
    ...


@overload
def ppmt(
    rate: _ScalarOrArrayLike[_Rate],
    per: _ScalarOrArrayLike[_Period],
    nper: _ScalarOrArrayLike[_Period],
    pv: _ScalarOrArrayLike[_Amount],
    fv: _ScalarOrArrayLike[_Amount] = 0,
    *,
    pmt_at_beginning: _ScalarOrArrayLike[bool] = False,
) -> List[Optional[float]]:
    ...


def cumprinc(
    rate: _Rate,
    nper: _Period,
    pv: _Amount,
    start_period: _Period,
    end_period: _Period,
    *,
    pmt_at_beginning: bool = False,
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
    day_count: _DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...


@overload
def xirr(
    dates: _CashFlow,
    *,
    guess: _Guess = 0.1,
    silent: bool = False,
    day_count: _DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...


def is_conventional_cash_flow(cf: _AmountArray) -> bool:
    ...


def zero_crossing_points(cf: _AmountArray) -> list[int]:
    ...
