// https://www.insead.edu/sites/default/files/assets/dept/centres/gpei/docs/Measuring_PE_Fund-Performance-2019.pdf

use super::InvalidPaymentsError;

type Result<T> = std::result::Result<T, InvalidPaymentsError>;

#[doc = include_str!("../../docs/_inline/pe/dpi.md")]
pub fn dpi(amounts: &[f64]) -> Result<f64> {
    let (cs, ds) = sum_negatives_positives(amounts);
    check_zero_contributions(cs)?;
    Ok(ds / -cs)
}

#[doc = include_str!("../../docs/_inline/pe/dpi.md")]
pub fn dpi_2(contributions: &[f64], distributions: &[f64]) -> Result<f64> {
    let cs: f64 = contributions.iter().sum();
    check_zero_contributions(cs)?;
    let ds: f64 = distributions.iter().sum();
    Ok(ds / cs)
}

#[doc = include_str!("../../docs/_inline/pe/rvpi.md")]
pub fn rvpi(contributions: &[f64], nav: f64) -> Result<f64> {
    let cs: f64 = contributions.iter().sum();
    check_zero_contributions(cs)?;
    let sign = series_signum(contributions);
    Ok(nav / (sign * cs))
}

#[doc = include_str!("../../docs/_inline/pe/tvpi.md")]
pub fn tvpi(amounts: &[f64], nav: f64) -> Result<f64> {
    let (cs, ds) = sum_negatives_positives(amounts);
    check_zero_contributions(cs)?;
    Ok((ds + nav) / -cs)
}

#[doc = include_str!("../../docs/_inline/pe/tvpi.md")]
pub fn tvpi_2(contributions: &[f64], distributions: &[f64], nav: f64) -> Result<f64> {
    // this is basically dpi_2(contributions, distributions) + rvpi(&contributions, nav)
    let cs: f64 = contributions.iter().sum();
    check_zero_contributions(cs)?;
    let ds: f64 = distributions.iter().sum();
    Ok((ds + nav) / cs)
}

pub fn moic(amounts: &[f64], nav: f64) -> Result<f64> {
    // MOIC divides the total value of the investment or fund by the total invested capital,
    // whereas TVPI divides it by the paid-in capital (meaning, the capital that investors have
    // actually transferred to the fund).
    // https://financestu.com/tvpi-vs-moic/
    // The math behind is the same. Simply create a semantic alias for the user.
    tvpi(amounts, nav)
}

pub fn moic_2(contributions: &[f64], distributions: &[f64], nav: f64) -> Result<f64> {
    tvpi_2(contributions, distributions, nav)
}

#[doc = include_str!("../../docs/_inline/pe/ks_pme_flows.md")]
pub fn ks_pme_flows(amounts: &[f64], index: &[f64]) -> Result<Vec<f64>> {
    check_input_len(amounts, index)?;

    Ok(pairwise_mul(amounts, &index_performance(index)))
}

#[doc = include_str!("../../docs/_inline/pe/ks_pme_flows.md")]
pub fn ks_pme_flows_2(
    contributions: &[f64],
    distributions: &[f64],
    index: &[f64],
) -> Result<(Vec<f64>, Vec<f64>)> {
    check_input_len(contributions, index)?;
    check_input_len(distributions, index)?;

    let px = index_performance(index);
    let c = pairwise_mul(contributions, &px);
    let d = pairwise_mul(distributions, &px);

    Ok((c, d))
}

pub fn ks_pme(amounts: &[f64], nav: f64, index: &[f64]) -> Result<f64> {
    ks_pme_flows(amounts, index).and_then(|a| tvpi(&a, nav))
}

pub fn ks_pme_2(
    contributions: &[f64],
    distributions: &[f64],
    nav: f64,
    index: &[f64],
) -> Result<f64> {
    ks_pme_flows_2(contributions, distributions, index).and_then(|(c, d)| tvpi_2(&c, &d, nav))
}

#[doc = include_str!("../../docs/_inline/pe/pme_plus_flows.md")]
pub fn pme_plus_flows(amounts: &[f64], nav: f64, index: &[f64]) -> Result<Vec<f64>> {
    check_input_len(amounts, index)?;

    let (contributions, distributions) = split_amounts(amounts);
    let scaled_distributions = pme_plus_flows_2(&contributions, &distributions, nav, index)?;
    let scaled_amounts = combine_amounts(&contributions, &scaled_distributions);

    Ok(scaled_amounts)
}

#[doc = include_str!("../../docs/_inline/pe/pme_plus_flows.md")]
pub fn pme_plus_flows_2(
    contributions: &[f64],
    distributions: &[f64],
    nav: f64,
    index: &[f64],
) -> Result<Vec<f64>> {
    let lambda = pme_plus_lambda_2(contributions, distributions, nav, index)?;
    Ok(scale(distributions, lambda))
}

pub fn pme_plus_lambda(amounts: &[f64], nav: f64, index: &[f64]) -> Result<f64> {
    check_input_len(amounts, index)?;

    let (contributions, distributions) = split_amounts(amounts);
    pme_plus_lambda_2(&contributions, &distributions, nav, index)
}

pub fn pme_plus_lambda_2(
    contributions: &[f64],
    distributions: &[f64],
    nav: f64,
    index: &[f64],
) -> Result<f64> {
    check_input_len(contributions, index)?;
    check_input_len(distributions, index)?;

    let px = index_performance(index);
    let ds = sum_pairwise_mul(distributions, &px);
    let cs = sum_pairwise_mul(contributions, &px);

    Ok((cs - nav) / ds)
}

pub fn pme_plus(amounts: &[f64], nav: f64, index: &[f64]) -> Result<f64> {
    let mut cf = pme_plus_flows(amounts, nav, index)?;

    if let Some(last) = cf.last_mut() {
        *last = nav
    };

    super::irr(&cf, None)
}

pub fn pme_plus_2(
    contributions: &[f64],
    distributions: &[f64],
    nav: f64,
    index: &[f64],
) -> Result<f64> {
    let scaled_distributions = pme_plus_flows_2(contributions, distributions, nav, index)?;
    let mut cf = combine_amounts(contributions, &scaled_distributions);

    if let Some(last) = cf.last_mut() {
        *last = nav
    };

    super::irr(&cf, None)
}
#[doc = include_str!("../../docs/_inline/pe/ln_pme_nav.md")]
pub fn ln_pme_nav(amounts: &[f64], index: &[f64]) -> Result<f64> {
    check_input_len(amounts, index)?;
    Ok(-sum_pairwise_mul(amounts, &index_performance(index)))
}

#[doc = include_str!("../../docs/_inline/pe/ln_pme_nav.md")]
pub fn ln_pme_nav_2(contributions: &[f64], distributions: &[f64], index: &[f64]) -> Result<f64> {
    check_input_len(contributions, index)?;
    check_input_len(distributions, index)?;

    let amounts = combine_amounts(contributions, distributions);
    ln_pme_nav(&amounts, index)
}

pub fn ln_pme(amounts: &[f64], index: &[f64]) -> Result<f64> {
    let pme_nav = ln_pme_nav(amounts, index)?;
    let mut cf = amounts.to_owned();
    if let Some(last) = cf.last_mut() {
        *last = pme_nav
    };
    super::irr(&cf, None)
}

pub fn ln_pme_2(contributions: &[f64], distributions: &[f64], index: &[f64]) -> Result<f64> {
    let mut amounts = combine_amounts(contributions, distributions);
    let pme_nav = ln_pme_nav(&amounts, index)?;
    if let Some(last) = amounts.last_mut() {
        *last = pme_nav
    };
    super::irr(&amounts, None)
}

fn check_zero_contributions(contributions: f64) -> Result<()> {
    if contributions == 0.0 {
        Err(InvalidPaymentsError::new("Contributions are zero"))
    } else {
        Ok(())
    }
}

fn check_input_len(amounts: &[f64], index: &[f64]) -> Result<()> {
    if amounts.len() != index.len() {
        Err(InvalidPaymentsError::new("Amounts must be the same length as index."))
    } else if index.len() == 0 {
        Err(InvalidPaymentsError::new("Input array must contain at least one value"))
    } else {
        Ok(())
    }
}

fn split_amounts(amounts: &[f64]) -> (Vec<f64>, Vec<f64>) {
    // split amounts into contributions and distributions.
    // make contributions positive
    let contributions: Vec<_> = amounts.iter().map(|x| x.clamp(f64::MIN, 0.0).abs()).collect();
    let distributions: Vec<_> = amounts.iter().map(|x| x.clamp(0.0, f64::MAX)).collect();

    (contributions, distributions)
}

fn combine_amounts(contributions: &[f64], distributions: &[f64]) -> Vec<f64> {
    // assume both contributions and distributions are positive
    // inverse operation of split_amounts
    contributions.iter().zip(distributions).map(|(c, d)| d - c).collect()
}

fn index_performance(index: &[f64]) -> Vec<f64> {
    let last = index.last().unwrap();
    index.iter().map(|p| last / p).collect()
}

fn scale(values: &[f64], factor: f64) -> Vec<f64> {
    values.iter().map(|v| v * factor).collect()
}

fn sum_pairwise_mul(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn pairwise_mul(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter().zip(b).map(|(x, y)| x * y).collect()
}

fn series_signum(a: &[f64]) -> f64 {
    // returns -1.0 if any item is negative, otherwise +1.0
    a.iter().any(|x| x.is_sign_negative()).then_some(-1.0).unwrap_or(1.0)
}

fn sum_negatives_positives(values: &[f64]) -> (f64, f64) {
    values.iter().fold((0.0, 0.0), |acc, x| {
        if x.is_sign_negative() {
            (acc.0 + x, acc.1)
        } else {
            (acc.0, acc.1 + x)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use rstest::rstest;

    // Test examples from the book:
    // https://www.insead.edu/sites/default/files/assets/dept/centres/gpei/docs/Measuring_PE_Fund-Performance-2019.pdf

    #[rstest]
    #[case(&[-12.0, 0.0, 0.0, 40.0], 0.494)]
    #[case(&[-12.0, -10.0, -4.0, 40.0, 0.0, 15.0, 5.0], 0.324)]
    fn test_irr(#[case] amounts: &[f64], #[case] expected: f64) {
        let result = crate::core::irr(amounts, None).unwrap();
        assert_approx_eq!(result, expected, 1e-3);
    }

    #[rstest]
    fn test_mirr() {
        let amounts = &[-12.0, -10.0, -4.0, 40.0, 0.0, 15.0, 5.0];
        let finance_rate = 0.07;
        let reinvest_rate = 0.12;
        let result = crate::core::mirr(amounts, finance_rate, reinvest_rate).unwrap();
        assert_approx_eq!(result, 0.21, 1e-3);
    }

    #[rstest]
    fn test_dpi() {
        let amounts = &[-10.0, -20.0, 15.0, 30.0];
        let (contributions, distributions) = split_amounts(amounts);

        assert_approx_eq!(dpi(amounts).unwrap(), 1.5);
        assert_approx_eq!(dpi_2(&contributions, &distributions).unwrap(), 1.5);
    }

    #[rstest]
    fn test_rvpi() {
        let amounts = &[10.0, 20.0, 15.0, 30.0];
        assert_approx_eq!(rvpi(amounts, 15.0).unwrap(), 0.2);
    }

    #[rstest]
    #[case(&[-10.0, -20.0, 15.0, 30.0], 15.0, 2.0)]
    #[case(&[-25.0, 15.0, 0.0], 20.0, 1.4)]
    fn test_tvpi(#[case] amounts: &[f64], #[case] nav: f64, #[case] expected: f64) {
        let result = tvpi(amounts, nav).unwrap();
        assert_approx_eq!(result, expected);

        let (contributions, distributions) = split_amounts(amounts);
        let result = tvpi_2(&contributions, &distributions, nav).unwrap();
        assert_approx_eq!(result, expected);
    }

    #[rstest]
    #[case(&[-25.0, 15.0, 0.0], 20.0, &[100.0, 115.0, 130.0], 1.14)]
    fn test_ks_pme(
        #[case] amounts: &[f64],
        #[case] nav: f64,
        #[case] index: &[f64],
        #[case] expected: f64,
    ) {
        let result = ks_pme(amounts, nav, index).unwrap();
        assert_approx_eq!(result, expected, 0.01);

        let (contributions, distributions) = split_amounts(amounts);
        let result = ks_pme_2(&contributions, &distributions, nav, index).unwrap();
        assert_approx_eq!(result, expected, 0.01);
    }

    #[rstest]
    #[case(&[-25.0, 15.0, 0.0], &[100.0, 115.0, 130.0], 15.5)]
    // example from https://en.wikipedia.org/wiki/Public_Market_Equivalent#Long-Nickels_PME
    #[case(&[-100.0, -50.0, 60.0, 10.0, 0.0], &[100.0, 105.0, 115.0, 117.0, 120.0], 104.28)]
    fn test_ln_pme_nav(#[case] amounts: &[f64], #[case] index: &[f64], #[case] expected: f64) {
        let result = ln_pme_nav(amounts, index).unwrap();
        assert_approx_eq!(result, expected, 0.1);

        let (contributions, distributions) = split_amounts(amounts);
        let result = ln_pme_nav_2(&contributions, &distributions, index).unwrap();
        assert_approx_eq!(result, expected, 0.1);
    }

    #[rstest]
    #[case(&[-25.0, 15.0, 0.0], &[100.0, 115.0, 130.0], 0.144)]
    // example from https://en.wikipedia.org/wiki/Public_Market_Equivalent#Long-Nickels_PME
    #[case(&[-100.0, -50.0, 60.0, 10.0, 0.0], &[100.0, 105.0, 115.0, 117.0, 120.0], 0.053)]
    fn test_ln_pme(#[case] amounts: &[f64], #[case] index: &[f64], #[case] expected: f64) {
        let result = ln_pme(amounts, index).unwrap();
        assert_approx_eq!(result, expected, 1e-3);

        let (contributions, distributions) = split_amounts(amounts);
        let result = ln_pme_2(&contributions, &distributions, index).unwrap();
        assert_approx_eq!(result, expected, 1e-3);
    }

    #[rstest]
    #[case(&[-25.0, 15.0, 0.0], 20.0, &[100.0, 115.0, 130.0], 0.7)]
    // example from https://en.wikipedia.org/wiki/Public_Market_Equivalent#PME+_Formula
    #[case(&[-100.0, -50.0, 60.0, 100.0, 0.0], 20.0, &[100.0, 105.0, 115.0, 110.0, 120.0], 0.86)]
    fn test_pme_plus_lambda(
        #[case] amounts: &[f64],
        #[case] nav: f64,
        #[case] index: &[f64],
        #[case] expected: f64,
    ) {
        let result = pme_plus_lambda(amounts, nav, index).unwrap();
        assert_approx_eq!(result, expected, 0.1);

        let (contributions, distributions) = split_amounts(amounts);
        let result = pme_plus_lambda_2(&contributions, &distributions, nav, index).unwrap();
        assert_approx_eq!(result, expected, 0.1);
    }

    #[rstest]
    #[case(&[-25.0, 15.0, 0.0], 20.0, &[100.0, 115.0, 130.0], 0.143)]
    // example from https://en.wikipedia.org/wiki/Public_Market_Equivalent#PME+_Formula
    #[case(&[-100.0, -50.0, 60.0, 100.0, 0.0], 20.0, &[100.0, 105.0, 115.0, 110.0, 120.0], 0.0205)]
    fn test_pme_plus(
        #[case] amounts: &[f64],
        #[case] nav: f64,
        #[case] index: &[f64],
        #[case] expected: f64,
    ) {
        let result = pme_plus(amounts, nav, index).unwrap();
        assert_approx_eq!(result, expected, 0.1);

        let (contributions, distributions) = split_amounts(amounts);
        let result = pme_plus_2(&contributions, &distributions, nav, index).unwrap();
        assert_approx_eq!(result, expected, 0.1);
    }
}
