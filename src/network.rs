/// Convert a per-sample network byte delta to kibibytes.
///
/// `bytes` is the byte count reported for the most recent sampling interval
/// (`sysinfo`'s `NetworkData::received`/`transmitted` report bytes observed
/// since the last refresh). It is a per-interval delta, not a cumulative
/// counter and not a rate.
pub fn sample_kibibytes(bytes: u64) -> f64 {
    bytes as f64 / 1024.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_bytes_convert_to_kibibytes() {
        assert!((sample_kibibytes(2048) - 2.0).abs() < f64::EPSILON);
        assert_eq!(sample_kibibytes(0), 0.0);
    }
}
