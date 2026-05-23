use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct AnomalyDetectorConfig {
    pub threshold_multiplier: f64,
}

pub struct TimeSeriesPoint<T> {
    pub timestamp: DateTime<Utc>,
    pub value: T,
}

#[allow(dead_code)]
pub struct Anomaly<T> {
    pub timestamp: DateTime<Utc>,
    pub value: T,
    pub severity: f64,
    pub message: String,
}

const MAX_HISTORY_SIZE: usize = 100;

/// Anomaly detector for numerical time-series data
pub struct AnomalyDetector {
    pub config: AnomalyDetectorConfig,
    history: Arc<RwLock<VecDeque<TimeSeriesPoint<f64>>>>,
    /// Detected anomalies history
    #[allow(dead_code)]
    pub anomalies: Arc<RwLock<Vec<Anomaly<f64>>>>,
    mean: Arc<RwLock<f64>>,
    std_dev: Arc<RwLock<f64>>,
    last_train_time: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl AnomalyDetector {
    pub fn new(config: AnomalyDetectorConfig) -> Self {
        AnomalyDetector {
            config,
            history: Arc::new(RwLock::new(VecDeque::new())),
            anomalies: Arc::new(RwLock::new(Vec::new())),
            mean: Arc::new(RwLock::new(0.0)),
            std_dev: Arc::new(RwLock::new(0.0)),
            last_train_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Add a data point to history and update statistics
    pub fn add_point(&self, value: f64) {
        let mut history = self.history.write();
        let point = TimeSeriesPoint {
            timestamp: Utc::now(),
            value,
        };
        history.push_back(point);

        // Keep history bounded
        while history.len() > MAX_HISTORY_SIZE {
            history.pop_front();
        }

        // Calculate mean
        let n = history.len() as f64;
        if n < 2.0 {
            return; // Need at least 2 points for meaningful std_dev
        }

        let sum: f64 = history.iter().map(|p| p.value).sum();
        let new_mean = sum / n;

        // Calculate standard deviation
        let variance: f64 = history
            .iter()
            .map(|p| {
                let diff = p.value - new_mean;
                diff * diff
            })
            .sum::<f64>()
            / n;
        let new_std_dev = variance.sqrt();

        *self.mean.write() = new_mean;
        *self.std_dev.write() = new_std_dev;
        *self.last_train_time.write() = Some(Utc::now());
    }

    /// Detect anomalies using statistical methods (z-score based)
    pub async fn detect_statistical_anomaly(
        &self,
        point: &TimeSeriesPoint<f64>,
    ) -> Option<Anomaly<f64>> {
        let mean = *self.mean.read();
        let std_dev = *self.std_dev.read();

        // Not enough data to detect anomalies yet
        if std_dev == 0.0 {
            return None;
        }

        // Calculate z-score
        let z_score = (point.value - mean).abs() / std_dev;

        // Flag as anomaly if z-score exceeds threshold
        if z_score > self.config.threshold_multiplier {
            let lower_bound = mean - self.config.threshold_multiplier * std_dev;
            let upper_bound = mean + self.config.threshold_multiplier * std_dev;

            Some(Anomaly {
                timestamp: point.timestamp,
                value: point.value,
                severity: z_score,
                message: format!(
                    "Anomaly: {:.1} (z={:.2}) is outside [{:.1}, {:.1}]",
                    point.value,
                    z_score,
                    lower_bound.max(0.0),
                    upper_bound.min(100.0)
                ),
            })
        } else {
            None
        }
    }
}

pub fn calculate_correlation(data1: &[f64], data2: &[f64]) -> Option<f64> {
    if data1.len() != data2.len() || data1.is_empty() {
        return None;
    }

    let n = data1.len() as f64;

    let sum_x: f64 = data1.iter().sum();
    let sum_y: f64 = data2.iter().sum();
    let sum_xy: f64 = data1.iter().zip(data2.iter()).map(|(x, y)| x * y).sum();
    let sum_x2: f64 = data1.iter().map(|x| x * x).sum();
    let sum_y2: f64 = data2.iter().map(|y| y * y).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

    if denominator == 0.0 {
        None
    } else {
        Some(numerator / denominator)
    }
}

pub fn evaluate_metric_formula(formula: &str, metrics: &HashMap<&str, f64>) -> Option<f64> {
    // Very basic formula evaluation for prototype
    // Supports only 'cpu_usage' and simple arithmetic (+, -, *, /)
    let mut result = formula.to_string();

    // Replace metric names with their values
    for (name, value) in metrics {
        result = result.replace(name, &value.to_string());
    }

    // Evaluate the expression (very basic, no operator precedence, just left to right)
    let parts: Vec<&str> = result.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let mut current_value = parts[0].parse::<f64>().ok()?;

    let mut i = 1;
    while i < parts.len() {
        let operator = parts[i];
        let operand = parts[i + 1].parse::<f64>().ok()?;

        match operator {
            "+" => current_value += operand,
            "-" => current_value -= operand,
            "*" => current_value *= operand,
            "/" => current_value /= operand,
            _ => return None, // Unsupported operator
        }
        i += 2;
    }

    Some(current_value)
}

pub fn predict_next_value(history: &[f64], window_size: usize) -> Option<f64> {
    if history.len() < window_size || window_size == 0 {
        return None;
    }
    let sum: f64 = history[history.len() - window_size..].iter().sum();
    Some(sum / window_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_correlation_perfect_positive() {
        let data1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let data2 = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let correlation = calculate_correlation(&data1, &data2).unwrap();
        assert!((correlation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_no_correlation() {
        let data1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let data2 = vec![5.0, 2.0, 8.0, 1.0, 6.0];
        let correlation = calculate_correlation(&data1, &data2);
        assert!(correlation.is_some());
    }

    #[test]
    fn test_prediction_basic() {
        let history = vec![10.0, 20.0, 30.0];
        let prediction = predict_next_value(&history, 2).unwrap();
        assert_eq!(prediction, 25.0); // (20+30)/2
    }

    #[test]
    fn test_metric_formula_basic() {
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage", 50.0);
        let result = evaluate_metric_formula("cpu_usage * 2.0", &metrics).unwrap();
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_metric_formula_addition() {
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage", 30.0);
        let result = evaluate_metric_formula("cpu_usage + 20.0", &metrics).unwrap();
        assert_eq!(result, 50.0);
    }
}
