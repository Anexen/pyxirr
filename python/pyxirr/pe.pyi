from collections.abc import Iterable
from decimal import Decimal
from typing import List, Optional, Tuple, Union

_Amount = Union[int, float, Decimal]
_AmountArray = Iterable[_Amount]


def dpi(amounts: _AmountArray) -> float:
    ...


def dpi_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
) -> float:
    ...


def rvpi(
    contributions: _AmountArray,
    nav: _Amount,
) -> float:
    ...


def tvpi(
    amounts: _AmountArray,
    nav: _Amount = 0,
) -> float:
    ...


def tvpi_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    nav: _Amount = 0,
) -> float:
    ...


def moic(
    amounts: _AmountArray,
    nav: _Amount = 0,
) -> float:
    ...


def moic_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    nav: _Amount = 0,
) -> float:
    ...


def ks_pme(
    amounts: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> Optional[float]:
    ...


def ks_pme_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> Optional[float]:
    ...


def ks_pme_flows(
    amounts: _AmountArray,
    index: _AmountArray,
) -> List[float]:
    ...


def ks_pme_flows_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    index: _AmountArray,
) -> Tuple[List[float], List[float]]:
    ...


def m_pme(
    amounts: _AmountArray,
    index: _AmountArray,
    nav: _AmountArray,
) -> float:
    ...


def m_pme_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    index: _AmountArray,
    nav: _AmountArray,
) -> float:
    ...


def pme_plus(
    amounts: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> Optional[float]:
    ...


def pme_plus_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> Optional[float]:
    ...


def pme_plus_flows(
    amounts: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> List[float]:
    ...


def pme_plus_flows_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> Tuple[List[float], List[float]]:
    ...


def pme_plus_lambda(
    amounts: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> float:
    ...


def pme_plus_lambda_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> float:
    ...


def ln_pme_nav(
    amounts: _AmountArray,
    index: _AmountArray,
) -> float:
    ...


def ln_pme_nav_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    index: _AmountArray,
) -> float:
    ...


def ln_pme(
    amounts: _AmountArray,
    index: _AmountArray,
) -> Optional[float]:
    ...


def ln_pme_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    index: _AmountArray,
) -> Optional[float]:
    ...


def direct_alpha(
    amounts: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> Optional[float]:
    ...


def direct_alpha_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
    index: _AmountArray,
    nav: _Amount = 0,
) -> Optional[float]:
    ...
