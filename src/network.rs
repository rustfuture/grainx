/// KB transferred since last sample (non-negative delta).
pub fn throughput_kbps(current_bytes: u64, previous_bytes: u64) -> f64 {
    current_bytes.saturating_sub(previous_bytes) as f64 / 1024.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throughput_uses_saturating_delta() {
        assert!((throughput_kbps(2048, 1024) - 1.0).abs() < f64::EPSILON);
        assert_eq!(throughput_kbps(512, 1024), 0.0);
    }
}
