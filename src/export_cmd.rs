use crate::error::{GrainxError, Result};
use crate::export::StatsSnapshot;
use crate::metrics::MetricBackend;

pub fn run(json_path: &str, csv_path: &str, remote: Option<&str>) -> Result<()> {
    let mut backend = match remote {
        Some(url) => MetricBackend::remote(url),
        None => MetricBackend::local(),
    };

    StatsSnapshot::save_both(json_path, csv_path, &mut backend).map_err(|err| {
        if remote.is_some() {
            GrainxError::RemoteMetrics(err.to_string())
        } else {
            GrainxError::Export(err.to_string())
        }
    })?;

    eprintln!("Exported stats to {json_path} and {csv_path}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn local_export_writes_json_and_csv() {
        let json = "test_export_cmd.json";
        let csv = "test_export_cmd.csv";
        fs::remove_file(json).ok();
        fs::remove_file(csv).ok();

        run(json, csv, None).unwrap();

        assert!(fs::metadata(json).unwrap().len() > 0);
        assert!(fs::metadata(csv).unwrap().len() > 0);

        fs::remove_file(json).ok();
        fs::remove_file(csv).ok();
    }

    #[test]
    fn remote_export_errors_when_agent_unreachable() {
        let err = run(
            "test_remote.json",
            "test_remote.csv",
            Some("http://127.0.0.1:59999"),
        )
        .unwrap_err();

        assert!(matches!(err, GrainxError::RemoteMetrics(_)));
    }
}
