/// Turing-Polymarket — Binary Conditional Token CPMM
///
/// Each DAG node gets an isolated binary prediction market.
/// 1 Coin = 1 YES + 1 NO (physical conservation, zero fiat printing).
/// Price = Bayesian probability. Oracle resolves at OMEGA.
/// Pure math — zero domain knowledge.

#[derive(Debug, Clone)]
pub struct BinaryMarket {
    pub node_id: String,
    /// YES share reserves in the pool
    pub yes_reserve: f64,
    /// NO share reserves in the pool
    pub no_reserve: f64,
    /// Constant product K = yes_reserve * no_reserve
    pub k: f64,
    /// None = open market, Some(true) = YES wins, Some(false) = NO wins
    pub resolved: Option<bool>,
    /// Total LP shares outstanding (for proportional withdrawal)
    pub lp_total: f64,
}

impl BinaryMarket {
    /// Genesis ignition: inject LP coins as neutral seed liquidity.
    /// Mints lp_coins YES + lp_coins NO into pool reserves.
    pub fn create(node_id: String, lp_coins: f64) -> Result<Self, String> {
        if lp_coins <= 0.0 || !lp_coins.is_finite() {
            return Err(format!("Invalid LP amount: {}", lp_coins));
        }
        Ok(Self {
            node_id,
            yes_reserve: lp_coins,
            no_reserve: lp_coins,
            k: lp_coins * lp_coins,
            lp_total: 1.0, // Creator gets 1.0 LP share (100%)
            resolved: None,
        })
    }

    /// Bayesian probability that this node is on the Golden Path.
    /// P_yes = no_reserve / (yes_reserve + no_reserve)
    /// Intuition: scarce YES shares = high YES price = high confidence.
    pub fn yes_price(&self) -> f64 {
        match self.resolved {
            Some(true) => 1.0,   // Certainty: YES wins
            Some(false) => 0.0,  // Certainty: NO wins
            None => {
                let total = self.yes_reserve + self.no_reserve;
                if total == 0.0 { return 0.5; }
                self.no_reserve / total
            }
        }
    }

    /// P_no = 1 - P_yes
    pub fn no_price(&self) -> f64 {
        1.0 - self.yes_price()
    }

    /// Buy YES shares: Mint [coins_in YES + coins_in NO], sell NO to pool.
    /// Returns total YES shares received by the buyer.
    ///
    /// Mechanism:
    ///   1. Protocol mints coins_in complete sets (coins_in YES + coins_in NO)
    ///   2. Sell coins_in NO to pool → receive extra YES from pool
    ///   3. Buyer gets: coins_in (minted) + yes_from_pool (swapped)
    pub fn buy_yes(&mut self, coins_in: f64) -> Result<f64, String> {
        if coins_in <= 0.0 || !coins_in.is_finite() {
            return Err(format!("Invalid buy amount: {}", coins_in));
        }
        if self.resolved.is_some() {
            return Err("Market resolved. No more trading.".into());
        }
        // Sell coins_in NO to pool
        let new_no = self.no_reserve + coins_in;
        let new_yes = self.k / new_no;
        let yes_from_pool = self.yes_reserve - new_yes;

        self.yes_reserve = new_yes;
        self.no_reserve = new_no;

        // Total YES = minted + from pool
        Ok(coins_in + yes_from_pool)
    }

    /// Buy NO shares: Mint [coins_in YES + coins_in NO], sell YES to pool.
    /// Returns total NO shares received by the buyer.
    pub fn buy_no(&mut self, coins_in: f64) -> Result<f64, String> {
        if coins_in <= 0.0 || !coins_in.is_finite() {
            return Err(format!("Invalid buy amount: {}", coins_in));
        }
        if self.resolved.is_some() {
            return Err("Market resolved. No more trading.".into());
        }
        // Sell coins_in YES to pool
        let new_yes = self.yes_reserve + coins_in;
        let new_no = self.k / new_yes;
        let no_from_pool = self.no_reserve - new_no;

        self.yes_reserve = new_yes;
        self.no_reserve = new_no;

        // Total NO = minted + from pool
        Ok(coins_in + no_from_pool)
    }

    /// Oracle resolution: external verifier decides truth. Irreversible.
    pub fn resolve(&mut self, yes_wins: bool) {
        if self.resolved.is_none() {
            self.resolved = Some(yes_wins);
        }
    }

    /// Redeem winning shares after resolution.
    /// YES wins: each YES share = 1 Coin, NO = 0.
    /// NO wins: each NO share = 1 Coin, YES = 0.
    pub fn redeem(&self, yes_shares: f64, no_shares: f64) -> f64 {
        match self.resolved {
            Some(true) => yes_shares,   // YES wins: 1 YES = 1 Coin
            Some(false) => no_shares,   // NO wins: 1 NO = 1 Coin
            None => 0.0,                // Not resolved yet
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_neutral_50_50() {
        let m = BinaryMarket::create("n".into(), 100.0).unwrap();
        assert!((m.yes_price() - 0.5).abs() < 1e-10);
        assert!((m.no_price() - 0.5).abs() < 1e-10);
        assert_eq!(m.k, 10_000.0);
    }

    #[test]
    fn test_buy_yes_increases_price() {
        let mut m = BinaryMarket::create("n".into(), 100.0).unwrap();
        let p_before = m.yes_price();
        let shares = m.buy_yes(50.0).unwrap();
        assert!(shares > 50.0, "Should get more than coins_in due to pool swap");
        assert!(m.yes_price() > p_before, "YES price should increase after buying");
    }

    #[test]
    fn test_buy_no_increases_no_price() {
        let mut m = BinaryMarket::create("n".into(), 100.0).unwrap();
        let p_before = m.no_price();
        let shares = m.buy_no(50.0).unwrap();
        assert!(shares > 50.0);
        assert!(m.no_price() > p_before);
    }

    #[test]
    fn test_prices_sum_to_one() {
        let mut m = BinaryMarket::create("n".into(), 100.0).unwrap();
        m.buy_yes(30.0).unwrap();
        m.buy_no(10.0).unwrap();
        assert!((m.yes_price() + m.no_price() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_split_ignition() {
        // Simulate: 1 Coin LP + 49 Coins buy_yes
        let mut m = BinaryMarket::create("n".into(), 1.0).unwrap();
        assert!((m.yes_price() - 0.5).abs() < 1e-10);
        let yes_shares = m.buy_yes(49.0).unwrap();
        assert!(yes_shares > 49.0);
        assert!(m.yes_price() > 0.95, "After 49x buy_yes on 1-LP pool, P_yes should be >95%");
    }

    #[test]
    fn test_resolve_and_redeem() {
        let mut m = BinaryMarket::create("n".into(), 100.0).unwrap();
        // YES wins
        m.resolve(true);
        assert_eq!(m.redeem(50.0, 30.0), 50.0); // 50 YES shares = 50 Coins
        // NO wins
        let mut m2 = BinaryMarket::create("n2".into(), 100.0).unwrap();
        m2.resolve(false);
        assert_eq!(m2.redeem(50.0, 30.0), 30.0); // 30 NO shares = 30 Coins
    }

    #[test]
    fn test_no_trading_after_resolution() {
        let mut m = BinaryMarket::create("n".into(), 100.0).unwrap();
        m.resolve(true);
        assert!(m.buy_yes(10.0).is_err());
        assert!(m.buy_no(10.0).is_err());
    }

    #[test]
    fn test_invalid_inputs() {
        assert!(BinaryMarket::create("n".into(), 0.0).is_err());
        assert!(BinaryMarket::create("n".into(), -1.0).is_err());
        assert!(BinaryMarket::create("n".into(), f64::NAN).is_err());

        let mut m = BinaryMarket::create("n".into(), 10.0).unwrap();
        assert!(m.buy_yes(0.0).is_err());
        assert!(m.buy_no(-5.0).is_err());
    }

    #[test]
    fn test_early_pioneer_profit() {
        // Pioneer buys YES at 50% → OMEGA resolves YES → profit
        let mut m = BinaryMarket::create("n".into(), 1.0).unwrap();
        let shares = m.buy_yes(49.0).unwrap();
        m.resolve(true);
        let payout = m.redeem(shares, 0.0);
        assert!(payout > 49.0, "Pioneer should profit: paid 49, got {:.2}", payout);
    }

    #[test]
    fn test_short_assassin_profit() {
        // Creator buys YES, then short assassin buys NO, creator's node fails
        let mut m = BinaryMarket::create("n".into(), 1.0).unwrap();
        let _creator_yes = m.buy_yes(49.0).unwrap();
        let assassin_no = m.buy_no(10.0).unwrap();
        m.resolve(false); // Node is NOT on GP
        let assassin_payout = m.redeem(0.0, assassin_no);
        assert!(assassin_payout > 10.0, "Assassin should profit: paid 10, got {:.2}", assassin_payout);
    }
}
