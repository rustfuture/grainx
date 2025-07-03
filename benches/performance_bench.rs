use criterion::{black_box, criterion_group, criterion_main, Criterion};
use grainx::analytics::{calculate_correlation, predict_next_value, evaluate_metric_formula};
use grainx::performance::PerformanceMonitor;
use std::collections::HashMap;

fn benchmark_correlation(c: &mut Criterion) {
    let data1: Vec<f64> = (0..1000).map(|i| i as f64).collect();
    let data2: Vec<f64> = (0..1000).map(|i| (i * 2) as f64).collect();
    
    c.bench_function("correlation_1000_points", |b| {
        b.iter(|| calculate_correlation(black_box(&data1), black_box(&data2)))
    });
}

fn benchmark_prediction(c: &mut Criterion) {
    let history: Vec<f64> = (0..1000).map(|i| i as f64).collect();
    
    c.bench_function("prediction_window_10", |b| {
        b.iter(|| predict_next_value(black_box(&history), black_box(10)))
    });
    
    c.bench_function("prediction_window_100", |b| {
        b.iter(|| predict_next_value(black_box(&history), black_box(100)))
    });
}

fn benchmark_metric_formula(c: &mut Criterion) {
    let mut metrics = HashMap::new();
    metrics.insert("cpu_usage", 75.5);
    metrics.insert("memory_usage", 60.2);
    
    c.bench_function("simple_formula", |b| {
        b.iter(|| evaluate_metric_formula(
            black_box("cpu_usage * 1.5"), 
            black_box(&metrics)
        ))
    });
    
    c.bench_function("complex_formula", |b| {
        b.iter(|| evaluate_metric_formula(
            black_box("cpu_usage * 1.5 + memory_usage * 0.8"), 
            black_box(&metrics)
        ))
    });
}

fn benchmark_performance_monitor(c: &mut Criterion) {
    c.bench_function("performance_monitor_frame_cycle", |b| {
        b.iter(|| {
            let mut perf = PerformanceMonitor::new(60.0);
            perf.start_frame();
            std::thread::sleep(std::time::Duration::from_millis(1));
            perf.end_frame();
            perf.get_fps()
        })
    });
    
    c.bench_function("adaptive_refresh_calculation", |b| {
        b.iter(|| {
            let mut perf = PerformanceMonitor::new(60.0);
            perf.calculate_adaptive_refresh(black_box(75.0))
        })
    });
}

criterion_group!(
    benches, 
    benchmark_correlation, 
    benchmark_prediction, 
    benchmark_metric_formula,
    benchmark_performance_monitor
);
criterion_main!(benches);