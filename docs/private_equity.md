{% include head.html %}

## DPI

{% include_relative _inline/pe/dpi.md %}

```python
def dpi(amounts: _AmountArray) -> float:
    ...


def dpi_2(
    contributions: _AmountArray,
    distributions: _AmountArray,
) -> float:
    ...
```

## RVPI

{% include_relative _inline/pe/rvpi.md %}

```python
def rvpi(
    contributions: _AmountArray,
    nav: _Amount,
) -> float:
    ...
```

## TVPI

{% include_relative _inline/pe/tvpi.md %}

```python
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
```
## MOIC

{% include_relative _inline/pe/moic.md %}

```python
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
```

## LN-PME

{% include_relative _inline/pe/ln_pme.md %}

```python
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
```

## LN-PME NAV

{% include_relative _inline/pe/ln_pme_nav.md %}

```python
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
```

## KS-PME Flows

{% include_relative _inline/pe/ks_pme_flows.md %}

```python
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
```

## KS-PME

{% include_relative _inline/pe/ks_pme.md %}

```python
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
```


## mPME

{% include_relative _inline/pe/m_pme.md %}

```python
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
```

## PME+ Flows

{% include_relative _inline/pe/pme_plus_flows.md %}

```python
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
```

## PME+ Lambda

{% include_relative _inline/pe/pme_plus_lambda.md %}

```python
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
```

## PME+

{% include_relative _inline/pe/pme_plus.md %}

```python
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
```

## Direct Alpha

{% include_relative _inline/pe/direct_alpha.md %}

```python
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
```
