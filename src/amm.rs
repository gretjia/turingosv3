/// TuringSwap — Constant-Product AMM for Knowledge DAG Nodes
///
/// Each node in the Tape gets its own Uniswap V2-style liquidity pool.
/// Citation = spot purchase. Price discovery is ex-ante, not ex-post.
/// Pure math — zero domain knowledge.

#[derive(Debug, Clone)]
pub struct UniswapPool {
    pub node_id: String,
    pub coin_reserve: f64,
    pub token_reserve: f64,
    pub k: f64,
}

impl UniswapPool {
    /// IDO: Pioneer injects initial coins, system creates token supply.
    /// Pioneer receives founder tokens separately (not stored here).
    pub fn launch(node_id: String, initial_coin: f64) -> Result<Self, String> {
        if initial_coin <= 0.0 || !initial_coin.is_finite() {
            return Err(format!("Invalid initial coin: {}", initial_coin));
        }
        let initial_token = 9_000.0;
        Ok(Self {
            node_id,
            coin_reserve: initial_coin,
            token_reserve: initial_token,
            k: initial_coin * initial_token,
        })
    }

    /// Quote: how many coins needed to buy `out_tokens` tokens?
    pub fn get_amount_in(&self, out_tokens: f64) -> Result<f64, String> {
        if out_tokens <= 0.0 || !out_tokens.is_finite() {
            return Err(format!("Invalid token amount: {}", out_tokens));
        }
        if out_tokens >= self.token_reserve {
            return Err(format!(
                "Insufficient liquidity: want {:.1} tokens, pool has {:.1}",
                out_tokens, self.token_reserve
            ));
        }
        let new_token_reserve = self.token_reserve - out_tokens;
        let new_coin_reserve = self.k / new_token_reserve;
        Ok(new_coin_reserve - self.coin_reserve)
    }

    /// Buy tokens with coins. Returns tokens received.
    pub fn swap_coin_for_token(&mut self, coins_in: f64) -> Result<f64, String> {
        if coins_in <= 0.0 || !coins_in.is_finite() {
            return Err(format!("Invalid coin input: {}", coins_in));
        }
        let new_coin_reserve = self.coin_reserve + coins_in;
        let new_token_reserve = self.k / new_coin_reserve;
        let tokens_out = self.token_reserve - new_token_reserve;
        self.coin_reserve = new_coin_reserve;
        self.token_reserve = new_token_reserve;
        Ok(tokens_out)
    }

    /// Sell tokens for coins. Returns coins received.
    pub fn swap_token_for_coin(&mut self, tokens_in: f64) -> Result<f64, String> {
        if tokens_in <= 0.0 || !tokens_in.is_finite() {
            return Err(format!("Invalid token input: {}", tokens_in));
        }
        let new_token_reserve = self.token_reserve + tokens_in;
        let new_coin_reserve = self.k / new_token_reserve;
        let coins_out = self.coin_reserve - new_coin_reserve;
        self.coin_reserve = new_coin_reserve;
        self.token_reserve = new_token_reserve;
        Ok(coins_out)
    }

    /// Current marginal price (coins per token).
    pub fn spot_price(&self) -> f64 {
        if self.token_reserve == 0.0 { return 0.0; }
        self.coin_reserve / self.token_reserve
    }

    /// Inject external liquidity (OMEGA bounty). Resets K.
    pub fn inject_liquidity(&mut self, coins: f64) -> Result<(), String> {
        if coins <= 0.0 || !coins.is_finite() {
            return Err(format!("Invalid liquidity amount: {}", coins));
        }
        self.coin_reserve += coins;
        self.k = self.coin_reserve * self.token_reserve;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_and_spot_price() {
        let pool = UniswapPool::launch("node_1".into(), 50.0).unwrap();
        assert_eq!(pool.coin_reserve, 50.0);
        assert_eq!(pool.token_reserve, 9_000.0);
        assert_eq!(pool.k, 450_000.0);
        assert!((pool.spot_price() - 50.0 / 9_000.0).abs() < 1e-10);
    }

    #[test]
    fn test_swap_round_trip_preserves_k() {
        let mut pool = UniswapPool::launch("n".into(), 100.0).unwrap();
        let k_before = pool.k;
        let tokens = pool.swap_coin_for_token(50.0).unwrap();
        assert!(tokens > 0.0);
        let coins_back = pool.swap_token_for_coin(tokens).unwrap();
        // K preserved within float tolerance
        assert!((pool.k - k_before).abs() < 1e-6);
        // Without fees, exact round-trip returns original amount (within float precision)
        assert!((coins_back - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_citation_cost_increases_with_demand() {
        let mut pool = UniswapPool::launch("n".into(), 50.0).unwrap();
        let cost_1 = pool.get_amount_in(100.0).unwrap();
        pool.swap_coin_for_token(cost_1).unwrap();
        let cost_2 = pool.get_amount_in(100.0).unwrap();
        assert!(cost_2 > cost_1, "Second citation should cost more");
    }

    #[test]
    fn test_invalid_inputs_rejected() {
        assert!(UniswapPool::launch("n".into(), 0.0).is_err());
        assert!(UniswapPool::launch("n".into(), -1.0).is_err());
        assert!(UniswapPool::launch("n".into(), f64::NAN).is_err());

        let mut pool = UniswapPool::launch("n".into(), 50.0).unwrap();
        assert!(pool.get_amount_in(0.0).is_err());
        assert!(pool.get_amount_in(-1.0).is_err());
        assert!(pool.get_amount_in(9_000.0).is_err()); // >= reserve
        assert!(pool.swap_coin_for_token(0.0).is_err());
        assert!(pool.swap_token_for_coin(-5.0).is_err());
        assert!(pool.inject_liquidity(0.0).is_err());
    }

    #[test]
    fn test_inject_liquidity_resets_k() {
        let mut pool = UniswapPool::launch("n".into(), 50.0).unwrap();
        let k_before = pool.k;
        pool.inject_liquidity(100.0).unwrap();
        assert_eq!(pool.coin_reserve, 150.0);
        assert!(pool.k > k_before);
        assert!((pool.k - 150.0 * 9_000.0).abs() < 1e-6);
    }
}
