Find λ used in PME+ method.

Formula: λ = (Scaled Distributions - NAV) / Scaled Contributions
Where:
- Scaled Distributions = sum(distributions * index[last] / index[current])
- Scaled Contributions = sum(contributions * index[last] / index[current])

See also:
- `pme_plus_flows` function
- <https://en.wikipedia.org/wiki/Public_Market_Equivalent#PME+>
- <https://blog.edda.co/advanced-fund-performance-methods-pme-direct-alpha/>
