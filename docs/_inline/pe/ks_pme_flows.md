Use the Kaplan-Schoar method to re-scale the private equity flows to match the
public market equivalents (PME) for comparison.

This method works as follows, for each period, re-scale the amount as:
`amount * (index[final_period] / index[current_period])`.
Basically you are future-valuing the amount to the final period based on the
returns of the PME.

See also:
- <https://en.wikipedia.org/wiki/Public_Market_Equivalent#Kaplan_Schoar_PME>
- <https://blog.edda.co/advanced-fund-performance-methods-pme-direct-alpha/>
