use rand_core::{OsRng, TryRngCore, OsError};
use rand_distr::{Distribution, Normal};

pub fn random_u128() -> Result<u128, OsError> {
    let mut bytes = [0u8; 16];
    OsRng.try_fill_bytes(&mut bytes)?;
    Ok(u128::from_le_bytes(bytes))
}

fn sample_trunc_normal_ms(mu: f64, sigma: f64, lo: f64, hi: f64) -> u64 {
    let normal = Normal::new(mu, sigma).expect("bad normal params");

    // rejection sampling（截断正态）
    loop {
        let mut rng = rand::rng();
        let x = normal.sample(&mut rng);
        if x.is_finite() && x >= lo && x <= hi {
            return x.round().max(0.0) as u64;
        }
    }
}

// 例：500ms ± 正态抖动（σ=50ms），截断到 [350,650]
pub fn sample_prove_time_ms(mu: u64, sigma: u64) -> u64 {
    let mu = mu as f64;
    let sigma = sigma as f64;
    sample_trunc_normal_ms(mu, sigma, mu - 3.0 * sigma, mu + 3.0 * sigma)
}
