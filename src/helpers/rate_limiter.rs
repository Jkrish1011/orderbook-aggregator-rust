use std::sync::Arc;
use std::time::{Duration, Instant};
use std::str::FromStr;
use tokio::sync::Mutex;
use rust_decimal::Decimal;

// Error returned when rate limit is exceeded
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitExceeded;

// Token bucket rate limiter that allows at most `capacity` tokens,
// with tokens refilling at `tokens_per_second` rate.
// 
// This implementation is non-blocking - it checks availability immediately
// without sleeping in the executing thread.
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
}

struct RateLimiterState {
    // Current number of available tokens (0.0 to capacity)
    tokens: Decimal,
    // Maximum capacity (bucketsize)
    capacity: Decimal,
    // Rate at which tokens are refilled per second
    tokens_per_second: Decimal,
    // Last time the state was updated (for calculating token refill)
    last_update: Instant,
}

impl RateLimiter {
    // Creates a new rate limiter with the specified capacity and refill rate.
    // 
    // # Arguments
    // * `capacity` - Maximum number of tokens (typically 1 for "once per interval")
    // * `tokens_per_second` - Rate at which tokens refill per second
    // 
    // # Panics
    // Panics if capacity or tokens_per_second is <= 0
    pub fn new(capacity: Decimal, tokens_per_second: Decimal) -> Self {
        if capacity <= Decimal::ZERO {
            panic!("Capacity must be greater than 0");
        }
        if tokens_per_second <= Decimal::ZERO {
            panic!("Tokens per second must be greater than 0");
        }

        Self {
            state: Arc::new(Mutex::new(RateLimiterState {
                tokens: capacity,
                capacity,
                tokens_per_second,
                last_update: Instant::now(),
            })),
        }
    }

    // Creates a rate limiter that allows at most one call per `interval`.
    // 
    // This is a convenience method for the common case where you want
    // "at most once every X seconds".
    pub fn new_per_interval(interval: Duration) -> Self {
        let capacity = Decimal::ONE;
        let interval_secs = interval.as_secs_f64();
        let tokens_per_second = Decimal::ONE / Decimal::from_str(&format!("{:.6}", interval_secs))
            .unwrap_or(Decimal::ONE);
        Self::new(capacity, tokens_per_second)
    }

    // Attempts to acquire a token without blocking.
    // 
    // Returns `Ok(())` if a token is available and consumed,
    // Returns `Err(RateLimitExceeded)` if no tokens are available.
    // 
    // This method is non-blocking and updates the internal state
    // based on elapsed time since last update.
    pub async fn try_acquire(&self) -> Result<(), RateLimitExceeded> {
        let mut state = self.state.lock().await;
        
        // Calculate time elapsed since last update
        let now = Instant::now();
        let elapsed = state.last_update.elapsed();
        
        // Refill tokens based on elapsed time
        let elapsed_secs = elapsed.as_secs_f64();
        if elapsed_secs > 0.0 {
            let elapsed_decimal = Decimal::from_str(&format!("{:.6}", elapsed_secs))
                .unwrap_or(Decimal::ZERO);
            let tokens_to_add = state.tokens_per_second * elapsed_decimal;
            state.tokens = (state.tokens + tokens_to_add).min(state.capacity);
            state.last_update = now;
        }
        
        // Check if we have at least one token
        if state.tokens < Decimal::ONE {
            return Err(RateLimitExceeded);
        }
        
        // Consume one token
        state.tokens -= Decimal::ONE;
        
        Ok(())
    }

    // Acquires a token, waiting asynchronously if necessary.
    // 
    // This method will wait until a token becomes available.
    // Note: This uses `tokio::time::sleep` which yields to the executor
    // and doesn't block the OS thread, making it suitable for async code.
    // 
    // If you need strictly non-blocking behavior, use `try_acquire` instead.
    pub async fn acquire(&self) {
        loop {
            match self.try_acquire().await {
                Ok(()) => return,
                Err(_) => {
                    // Calculate how long to wait until next token is available
                    let state = self.state.lock().await;
                    let tokens_needed = Decimal::ONE - state.tokens;
                    let wait_seconds_decimal = tokens_needed / state.tokens_per_second;
                    let wait_seconds = wait_seconds_decimal.to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0);
                    drop(state);
                    
                    if wait_seconds > 0.0 {
                        tokio::time::sleep(Duration::from_secs_f64(wait_seconds)).await;
                    }
                }
            }
        }
    }

    // Returns the current number of available tokens (approximate).
    pub async fn available_tokens(&self) -> Decimal {
        let mut state = self.state.lock().await;
        
        // Update tokens based on elapsed time
        let elapsed = state.last_update.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        if elapsed_secs > 0.0 {
            let elapsed_decimal = Decimal::from_str(&format!("{:.6}", elapsed_secs))
                .unwrap_or(Decimal::ZERO);
            let tokens_to_add = state.tokens_per_second * elapsed_decimal;
            state.tokens = (state.tokens + tokens_to_add).min(state.capacity);
            state.last_update = Instant::now();
        }
        
        state.tokens
    }
}