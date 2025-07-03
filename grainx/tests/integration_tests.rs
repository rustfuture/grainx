use grainx::analytics::{calculate_correlation, predict_next_value, evaluate_metric_formula};
use grainx::config::DashboardConfig;
use grainx::performance::PerformanceMonitor;
use std::collections::HashMap;

#[test]
fn test_correlation_calculation() {
    let data1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let data2 = vec![2.0, 4.0, 6.0, 8.0, 10.0];
    
    let correlation = calculate_correlation(&data1, &data2).unwrap();
    assert!((correlation - 1.0).abs() < 0.001, "Perfect positive correlation should be ~1.0");
}

#[test]
fn test_correlation_negative() {
    let data1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let data2 = vec![5.0, 4.0, 3.0, 2.0, 1.0];
    
    let correlation = calculate_correlation(&data1, &data2).unwrap();
    assert!((correlation + 1.0).abs() < 0.001, "Perfect negative correlation should be ~-1.0");
}

#[test]
fn test_correlation_empty_data() {
    let data1: Vec<f64> = vec![];
    let data2: Vec<f64> = vec![];
    
    let correlation = calculate_correlation(&data1, &data2);
    assert!(correlation.is_none(), "Empty data should return None");
}

#[test]
fn test_prediction_simple_average() {
    let history = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    let prediction = predict_next_value(&history, 3).unwrap();
    
    // Should predict average of last 3 values: (30+40+50)/3 = 40
    assert!((prediction - 40.0).abs() < 0.001, "Prediction should be 40.0");
}

#[test]
fn test_prediction_insufficient_data() {
    let history = vec![10.0, 20.0];
    let prediction = predict_next_value(&history, 5);
    
    assert!(prediction.is_none(), "Insufficient data should return None");
}

#[test]
fn test_metric_formula_evaluation() {
    let mut metrics = HashMap::new();
    metrics.insert("cpu_usage", 50.0);
    
    let result = evaluate_metric_formula("cpu_usage * 2.0", &metrics).unwrap();
    assert!((result - 100.0).abs() < 0.001, "50 * 2 should be 100");
}

#[test]
fn test_metric_formula_complex() {
    let mut metrics = HashMap::new();
    metrics.insert("cpu_usage", 60.0);
    
    let result = evaluate_metric_formula("cpu_usage * 1.5 + 10.0", &metrics).unwrap();
    assert!((result - 100.0).abs() < 0.001, "60 * 1.5 + 10 should be 100");
}

#[test]
fn test_dashboard_config_creation() {
    let config = DashboardConfig::default_config();
    
    assert_eq!(config.name, "grainx_advanced");
    assert!(config.refresh_interval_ms > 0);
    assert!(config.cpu_warning_threshold > 0.0);
    assert!(config.memory_warning_threshold > 0.0);
    assert!(config.max_processes > 0);
}

#[test]
fn test_dashboard_config_serialization() {
    let config = DashboardConfig::default_config();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: DashboardConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.name, deserialized.name);
    assert_eq!(config.refresh_interval_ms, deserialized.refresh_interval_ms);
}

#[test]
fn test_performance_monitor_fps_calculation() {
    let mut perf = PerformanceMonitor::new(60.0);
    
    // Simulate some frames
    for _ in 0..10 {
        perf.start_frame();
        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
        perf.end_frame();
    }
    
    let fps = perf.get_fps();
    assert!(fps > 50.0 && fps < 70.0, "FPS should be around 60, got {}", fps);
}

#[test]
fn test_performance_monitor_adaptive_refresh() {
    let mut perf = PerformanceMonitor::new(60.0);
    
    // Test high CPU scenario
    let high_cpu_refresh = perf.calculate_adaptive_refresh(95.0);
    assert!(high_cpu_refresh >= 1000, "High CPU should result in slower refresh");
    
    // Test low CPU scenario
    let low_cpu_refresh = perf.calculate_adaptive_refresh(10.0);
    assert!(low_cpu_refresh <= 500, "Low CPU should result in faster refresh");
}

#[test]
fn test_performance_monitor_frame_skipping() {
    let perf = PerformanceMonitor::new(60.0);
    
    assert!(perf.should_skip_frame(98.0), "Very high CPU should skip frames");
    assert!(!perf.should_skip_frame(50.0), "Normal CPU should not skip frames");
}

#[test]
fn test_performance_monitor_toggle_adaptive() {
    let mut perf = PerformanceMonitor::new(60.0);
    let initial_state = perf.get_performance_stats().2;
    
    perf.toggle_adaptive_refresh();
    let toggled_state = perf.get_performance_stats().2;
    
    assert_ne!(initial_state, toggled_state, "Adaptive state should toggle");
}