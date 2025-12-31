use alphafield_backtest::error::BacktestError;
use alphafield_backtest::portfolio::Portfolio;

#[test]
fn rejects_selling_more_than_owned() {
    let mut p = Portfolio::new(1000.0);

    // Buy 1.0 unit at price 100.0
    p.update_from_fill("BTC", 1.0, 100.0, 0.0, None)
        .expect("buy should succeed");

    // Attempt to sell 2.0 units -> should error with InsufficientPosition
    let res = p.update_from_fill("BTC", -2.0, 100.0, 0.0, None);
    match res {
        Err(BacktestError::InsufficientPosition {
            symbol,
            required,
            available,
        }) => {
            assert_eq!(symbol, "BTC");
            assert!((required - 2.0).abs() < 1e-9);
            assert!((available - 1.0).abs() < 1e-9);
        }
        other => panic!("expected InsufficientPosition, got: {:?}", other),
    }
}
