#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use rstest::rstest;

    // Test examples from the book:
    // https://www.insead.edu/sites/default/files/assets/dept/centres/gpei/docs/Measuring_PE_Fund-Performance-2019.pdf

    #[rstest]
    #[case(&[-12., 0., 0., 40.], 0.494)]
    #[case(&[-12., -10., -4., 40., 0., 15., 5.], 0.324)]
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
}
