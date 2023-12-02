{% include head.html %}

## Type annotations

```python
Amount = Union[int, float, Decimal]  # also supports numpy types
AmountArray = Iterable[Amount]
```

## DPI

```python
def dpi(amounts: AmountArray) -> float:
    ...


def dpi_2(
    contributions: AmountArray,
    distributions: AmountArray,
) -> float:
    ...
```

{% include_relative _inline/pe/dpi.md %}

## RVPI

```python
def rvpi(
    contributions: AmountArray,
    nav: Amount,
) -> float:
    ...
```

{% include_relative _inline/pe/rvpi.md %}

## TVPI

```python
def tvpi(
    amounts: AmountArray,
    nav: Amount = 0,
) -> float:
    ...


def tvpi_2(
    contributions: AmountArray,
    distributions: AmountArray,
    nav: Amount = 0,
) -> float:
    ...
```

{% include_relative _inline/pe/tvpi.md %}

## MOIC

```python
def moic(
    amounts: AmountArray,
    nav: Amount = 0,
) -> float:
    ...


def moic_2(
    contributions: AmountArray,
    distributions: AmountArray,
    nav: Amount = 0,
) -> float:
    ...
```

{% include_relative _inline/pe/moic.md %}

## LN-PME

```python
def ln_pme(
    amounts: AmountArray,
    index: AmountArray,
) -> Optional[float]:
    ...


def ln_pme_2(
    contributions: AmountArray,
    distributions: AmountArray,
    index: AmountArray,
) -> Optional[float]:
    ...
```

{% include_relative _inline/pe/ln_pme.md %}

## LN-PME NAV

```python
def ln_pme_nav(
    amounts: AmountArray,
    index: AmountArray,
) -> float:
    ...


def ln_pme_nav_2(
    contributions: AmountArray,
    distributions: AmountArray,
    index: AmountArray,
) -> float:
    ...
```

{% include_relative _inline/pe/ln_pme_nav.md %}

## KS-PME Flows

```python
def ks_pme_flows(
    amounts: AmountArray,
    index: AmountArray,
) -> List[float]:
    ...


def ks_pme_flows_2(
    contributions: AmountArray,
    distributions: AmountArray,
    index: AmountArray,
) -> Tuple[List[float], List[float]]:
    ...
```

{% include_relative _inline/pe/ks_pme_flows.md %}

## KS-PME

```python
def ks_pme(
    amounts: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> Optional[float]:
    ...


def ks_pme_2(
    contributions: AmountArray,
    distributions: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> Optional[float]:
    ...
```

{% include_relative _inline/pe/ks_pme.md %}

## mPME

```python
def m_pme(
    amounts: AmountArray,
    index: AmountArray,
    nav: AmountArray,
) -> float:
    ...


def m_pme_2(
    contributions: AmountArray,
    distributions: AmountArray,
    index: AmountArray,
    nav: AmountArray,
) -> float:
    ...
```

{% include_relative _inline/pe/m_pme.md %}

## PME+ Flows

```python
def pme_plus_flows(
    amounts: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> List[float]:
    ...


def pme_plus_flows_2(
    contributions: AmountArray,
    distributions: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> Tuple[List[float], List[float]]:
    ...
```

{% include_relative _inline/pe/pme_plus_flows.md %}

## PME+ Lambda

```python
def pme_plus_lambda(
    amounts: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> float:
    ...


def pme_plus_lambda_2(
    contributions: AmountArray,
    distributions: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> float:
    ...
```

{% include_relative _inline/pe/pme_plus_lambda.md %}

## PME+

```python
def pme_plus(
    amounts: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> Optional[float]:
    ...


def pme_plus_2(
    contributions: AmountArray,
    distributions: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> Optional[float]:
    ...
```

{% include_relative _inline/pe/pme_plus.md %}

## Direct Alpha

```python
def direct_alpha(
    amounts: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> Optional[float]:
    ...


def direct_alpha_2(
    contributions: AmountArray,
    distributions: AmountArray,
    index: AmountArray,
    nav: Amount = 0,
) -> Optional[float]:
    ...
```

{% include_relative _inline/pe/direct_alpha.md %}
