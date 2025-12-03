use alphafield_strategy::indicators::{BollingerBands, Ema, Indicator, Macd, Rsi, Sma};

#[test]
fn test_sma_integration() {
    let mut sma = Sma::new(3);

    // Test sequence
    assert_eq!(sma.update(10.0), None);
    assert_eq!(sma.update(20.0), None);
    assert_eq!(sma.update(30.0), Some(20.0)); // (10+20+30)/3 = 20
    assert_eq!(sma.update(40.0), Some(30.0)); // (20+30+40)/3 = 30

    // Test reset
    sma.reset();
    assert_eq!(sma.update(10.0), None);
}

#[test]
fn test_ema_integration() {
    let mut ema = Ema::new(3); // k = 2/(3+1) = 0.5

    assert_eq!(ema.update(10.0), Some(10.0));
    assert_eq!(ema.update(20.0), Some(15.0)); // (20-10)*0.5 + 10 = 15

    ema.reset();
    assert_eq!(ema.update(10.0), Some(10.0));
}

#[test]
fn test_rsi_integration() {
    let mut rsi = Rsi::new(3);

    // Need at least period+1 values to get first RSI
    // Initial values to build history
    assert_eq!(rsi.update(100.0), None);
    assert_eq!(rsi.update(102.0), None); // Gain 2
    assert_eq!(rsi.update(104.0), None); // Gain 2

    // Fourth value triggers calculation
    // Gains: 2, 2, 2. Avg Gain = 2. Avg Loss = 0. RS = inf -> RSI = 100
    assert_eq!(rsi.update(106.0), Some(100.0));

    rsi.reset();
    assert_eq!(rsi.update(100.0), None);
}

#[test]
fn test_bollinger_bands_integration() {
    let mut bb = BollingerBands::new(5, 2.0);

    // Fill window
    assert_eq!(bb.update(10.0), None);
    assert_eq!(bb.update(10.0), None);
    assert_eq!(bb.update(10.0), None);
    assert_eq!(bb.update(10.0), None);

    // 5th value, all same -> std dev 0
    let (upper, middle, lower) = bb.update(10.0).unwrap();
    assert_eq!(middle, 10.0);
    assert_eq!(upper, 10.0);
    assert_eq!(lower, 10.0);

    // Add variance
    let (upper, middle, lower) = bb.update(20.0).unwrap();
    assert!(upper > middle);
    assert!(lower < middle);
    assert_eq!(middle, 12.0); // (10+10+10+10+20)/5 = 12
}

#[test]
fn test_macd_integration() {
    // Use small periods for testing
    let mut macd = Macd::new(3, 6, 3);

    // Feed data
    for i in 0..10 {
        let val = 10.0 + i as f64;
        let result = macd.update(val);

        if i < 5 { // Needs slow_period-1 to start producing meaningful values?
             // Actually EMA starts immediately, but signal line needs history
             // First few might be None depending on implementation details of signal line
        } else {
            assert!(result.is_some());
            let (macd_line, signal, hist) = result.unwrap();
            // In an uptrend, MACD should generally be positive
            assert!(macd_line > -1.0); // Loose check
            assert_eq!(hist, macd_line - signal);
        }
    }
}
