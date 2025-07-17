use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use rayon::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub operation_name: String,
    pub total_calls: u64,
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub description: String,
    pub potential_improvement: f64,
    pub implementation_difficulty: String,
    pub priority: OptimizationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPriority {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub allocated_mb: f64,
    pub used_mb: f64,
    pub peak_mb: f64,
    pub fragmentation_percent: f64,
    pub gc_pressure: f64,
}

pub struct PerformanceOptimizer {
    profiles: Arc<Mutex<HashMap<String, PerformanceProfile>>>,
    memory_metrics: Arc<Mutex<MemoryMetrics>>,
    optimization_suggestions: Arc<Mutex<Vec<OptimizationSuggestion>>>,
    start_time: Instant,
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(Mutex::new(HashMap::new())),
            memory_metrics: Arc::new(Mutex::new(MemoryMetrics {
                allocated_mb: 0.0,
                used_mb: 0.0,
                peak_mb: 0.0,
                fragmentation_percent: 0.0,
                gc_pressure: 0.0,
            })),
            optimization_suggestions: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        }
    }
    
    pub fn record_operation(&self, operation_name: &str, duration_ms: u64, memory_mb: f64, cpu_percent: f64, cache_hit: bool, success: bool) {
        if let Ok(mut profiles) = self.profiles.lock() {
            let profile = profiles.entry(operation_name.to_string()).or_insert_with(|| PerformanceProfile {
                operation_name: operation_name.to_string(),
                total_calls: 0,
                total_duration_ms: 0,
                avg_duration_ms: 0.0,
                min_duration_ms: u64::MAX,
                max_duration_ms: 0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                cache_hit_rate: 0.0,
                error_rate: 0.0,
            });
            
            profile.total_calls += 1;
            profile.total_duration_ms += duration_ms;
            profile.avg_duration_ms = profile.total_duration_ms as f64 / profile.total_calls as f64;
            profile.min_duration_ms = profile.min_duration_ms.min(duration_ms);
            profile.max_duration_ms = profile.max_duration_ms.max(duration_ms);
            
            // Update memory and CPU metrics (simplified)
            profile.memory_usage_mb = (profile.memory_usage_mb + memory_mb) / 2.0;
            profile.cpu_usage_percent = (profile.cpu_usage_percent + cpu_percent) / 2.0;
            
            // Update cache hit rate
            let total_cache_attempts = profile.total_calls;
            let current_hits = if cache_hit { 1 } else { 0 };
            profile.cache_hit_rate = (profile.cache_hit_rate * (total_cache_attempts - 1) as f64 + current_hits as f64) / total_cache_attempts as f64;
            
            // Update error rate
            let current_errors = if success { 0 } else { 1 };
            profile.error_rate = (profile.error_rate * (total_cache_attempts - 1) as f64 + current_errors as f64) / total_cache_attempts as f64;
        }
        
        self.update_memory_metrics(memory_mb);
        self.analyze_performance(operation_name, duration_ms);
    }
    
    fn update_memory_metrics(&self, memory_mb: f64) {
        if let Ok(mut metrics) = self.memory_metrics.lock() {
            metrics.used_mb = memory_mb;
            metrics.peak_mb = metrics.peak_mb.max(memory_mb);
            metrics.allocated_mb = memory_mb * 1.1; // Estimate allocated memory
            
            // Calculate fragmentation (simplified)
            if metrics.allocated_mb > 0.0 {
                metrics.fragmentation_percent = ((metrics.allocated_mb - metrics.used_mb) / metrics.allocated_mb) * 100.0;
            }
            
            // Calculate GC pressure (simplified)
            metrics.gc_pressure = if metrics.fragmentation_percent > 50.0 {
                metrics.fragmentation_percent / 100.0
            } else {
                0.0
            };
        }
    }
    
    fn analyze_performance(&self, operation_name: &str, duration_ms: u64) {
        if let Ok(profiles) = self.profiles.lock() {
            if let Some(profile) = profiles.get(operation_name) {
                let mut suggestions = Vec::new();
                
                // Analyze slow operations
                if profile.avg_duration_ms > 1000.0 {
                    suggestions.push(OptimizationSuggestion {
                        category: "Performance".to_string(),
                        description: format!("Operation '{}' is slow (avg: {:.2}ms)", operation_name, profile.avg_duration_ms),
                        potential_improvement: 0.5,
                        implementation_difficulty: "Medium".to_string(),
                        priority: OptimizationPriority::HIGH,
                    });
                }
                
                // Analyze memory usage
                if profile.memory_usage_mb > 100.0 {
                    suggestions.push(OptimizationSuggestion {
                        category: "Memory".to_string(),
                        description: format!("Operation '{}' uses high memory ({}MB)", operation_name, profile.memory_usage_mb),
                        potential_improvement: 0.3,
                        implementation_difficulty: "High".to_string(),
                        priority: OptimizationPriority::MEDIUM,
                    });
                }
                
                // Analyze cache efficiency
                if profile.cache_hit_rate < 0.5 {
                    suggestions.push(OptimizationSuggestion {
                        category: "Caching".to_string(),
                        description: format!("Low cache hit rate for '{}' ({:.1}%)", operation_name, profile.cache_hit_rate * 100.0),
                        potential_improvement: 0.4,
                        implementation_difficulty: "Low".to_string(),
                        priority: OptimizationPriority::MEDIUM,
                    });
                }
                
                // Analyze error rates
                if profile.error_rate > 0.1 {
                    suggestions.push(OptimizationSuggestion {
                        category: "Reliability".to_string(),
                        description: format!("High error rate for '{}' ({:.1}%)", operation_name, profile.error_rate * 100.0),
                        potential_improvement: 0.8,
                        implementation_difficulty: "Medium".to_string(),
                        priority: OptimizationPriority::CRITICAL,
                    });
                }
                
                if let Ok(mut opt_suggestions) = self.optimization_suggestions.lock() {
                    opt_suggestions.extend(suggestions);
                }
            }
        }
    }
    
    pub fn get_performance_profiles(&self) -> Vec<PerformanceProfile> {
        if let Ok(profiles) = self.profiles.lock() {
            profiles.values().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn get_memory_metrics(&self) -> MemoryMetrics {
        if let Ok(metrics) = self.memory_metrics.lock() {
            metrics.clone()
        } else {
            MemoryMetrics {
                allocated_mb: 0.0,
                used_mb: 0.0,
                peak_mb: 0.0,
                fragmentation_percent: 0.0,
                gc_pressure: 0.0,
            }
        }
    }
    
    pub fn get_optimization_suggestions(&self) -> Vec<OptimizationSuggestion> {
        if let Ok(suggestions) = self.optimization_suggestions.lock() {
            suggestions.clone()
        } else {
            Vec::new()
        }
    }
    
    pub fn optimize_text_processing(&self, text: &str) -> String {
        // Apply various text processing optimizations
        let mut optimized_text = text.to_string();
        
        // Remove excessive whitespace
        optimized_text = optimized_text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        
        // Normalize unicode
        optimized_text = optimized_text.chars()
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .collect();
        
        // Remove duplicate spaces
        while optimized_text.contains("  ") {
            optimized_text = optimized_text.replace("  ", " ");
        }
        
        optimized_text
    }
    
    pub fn optimize_data_processing(&self, data: &str) -> String {
        // Optimize numeric data processing
        let numbers: Vec<&str> = data
            .split(|c| c == ',' || c == ' ' || c == '\n' || c == '\t')
            .filter(|s| !s.trim().is_empty())
            .collect();
        
        // Use parallel processing for large datasets
        if numbers.len() > 1000 {
            let processed_numbers: Vec<String> = numbers
                .par_iter()
                .map(|num| num.trim().to_string())
                .collect();
            processed_numbers.join(",")
        } else {
            numbers.iter().map(|s| s.trim()).collect::<Vec<_>>().join(",")
        }
    }
    
    pub fn get_performance_summary(&self) -> HashMap<String, f64> {
        let mut summary = HashMap::new();
        
        if let Ok(profiles) = self.profiles.lock() {
            let total_operations: u64 = profiles.values().map(|p| p.total_calls).sum();
            let total_duration: u64 = profiles.values().map(|p| p.total_duration_ms).sum();
            let avg_duration = if total_operations > 0 {
                total_duration as f64 / total_operations as f64
            } else {
                0.0
            };
            
            let avg_cache_hit_rate = profiles.values()
                .map(|p| p.cache_hit_rate)
                .sum::<f64>() / profiles.len().max(1) as f64;
            
            let avg_error_rate = profiles.values()
                .map(|p| p.error_rate)
                .sum::<f64>() / profiles.len().max(1) as f64;
            
            summary.insert("total_operations".to_string(), total_operations as f64);
            summary.insert("avg_duration_ms".to_string(), avg_duration);
            summary.insert("avg_cache_hit_rate".to_string(), avg_cache_hit_rate);
            summary.insert("avg_error_rate".to_string(), avg_error_rate);
        }
        
        if let Ok(memory_metrics) = self.memory_metrics.lock() {
            summary.insert("memory_used_mb".to_string(), memory_metrics.used_mb);
            summary.insert("memory_peak_mb".to_string(), memory_metrics.peak_mb);
            summary.insert("fragmentation_percent".to_string(), memory_metrics.fragmentation_percent);
        }
        
        summary
    }
}

// Global performance optimizer
lazy_static! {
    static ref PERFORMANCE_OPTIMIZER: Arc<PerformanceOptimizer> = Arc::new(PerformanceOptimizer::new());
}

// Public performance functions
pub fn record_operation_performance(operation_name: &str, duration_ms: u64, memory_mb: f64, cpu_percent: f64, cache_hit: bool, success: bool) {
    PERFORMANCE_OPTIMIZER.record_operation(operation_name, duration_ms, memory_mb, cpu_percent, cache_hit, success);
}

pub fn get_performance_profiles() -> Vec<PerformanceProfile> {
    PERFORMANCE_OPTIMIZER.get_performance_profiles()
}

pub fn get_memory_metrics() -> MemoryMetrics {
    PERFORMANCE_OPTIMIZER.get_memory_metrics()
}

pub fn get_optimization_suggestions() -> Vec<OptimizationSuggestion> {
    PERFORMANCE_OPTIMIZER.get_optimization_suggestions()
}

pub fn optimize_text_processing(text: &str) -> String {
    PERFORMANCE_OPTIMIZER.optimize_text_processing(text)
}

pub fn optimize_data_processing(data: &str) -> String {
    PERFORMANCE_OPTIMIZER.optimize_data_processing(data)
}

pub fn get_performance_summary() -> HashMap<String, f64> {
    PERFORMANCE_OPTIMIZER.get_performance_summary()
}

// Performance monitoring macros
#[macro_export]
macro_rules! measure_performance {
    ($operation_name:expr, $block:expr) => {{
        let start_time = std::time::Instant::now();
        let result = $block;
        let duration = start_time.elapsed().as_millis() as u64;
        
        $crate::performance::record_operation_performance(
            $operation_name,
            duration,
            0.0, // memory_mb
            0.0, // cpu_percent
            false, // cache_hit
            true, // success
        );
        
        result
    }};
}

#[macro_export]
macro_rules! measure_performance_with_cache {
    ($operation_name:expr, $cache_key:expr, $block:expr) => {{
        // Check cache first
        if let Some(cached_result) = $crate::cache::get_cached_result($cache_key) {
            $crate::performance::record_operation_performance(
                $operation_name,
                0, // duration_ms
                0.0, // memory_mb
                0.0, // cpu_percent
                true, // cache_hit
                true, // success
            );
            return cached_result;
        }
        
        // Execute operation
        let start_time = std::time::Instant::now();
        let result = $block;
        let duration = start_time.elapsed().as_millis() as u64;
        
        // Cache the result
        $crate::cache::set_cached_result($cache_key, result.clone());
        
        $crate::performance::record_operation_performance(
            $operation_name,
            duration,
            0.0, // memory_mb
            0.0, // cpu_percent
            false, // cache_hit
            true, // success
        );
        
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_recording() {
        record_operation_performance("test_operation", 100, 10.0, 5.0, true, true);
        record_operation_performance("test_operation", 200, 15.0, 8.0, false, true);
        
        let profiles = get_performance_profiles();
        assert!(!profiles.is_empty());
        
        let profile = profiles.iter().find(|p| p.operation_name == "test_operation").unwrap();
        assert_eq!(profile.total_calls, 2);
        assert_eq!(profile.avg_duration_ms, 150.0);
    }
    
    #[test]
    fn test_text_optimization() {
        let input = "  Hello   world  \n\n  test  ";
        let optimized = optimize_text_processing(input);
        assert_eq!(optimized, "Hello world test");
    }
    
    #[test]
    fn test_data_optimization() {
        let input = "1, 2, 3, 4, 5";
        let optimized = optimize_data_processing(input);
        assert_eq!(optimized, "1,2,3,4,5");
    }
    
    #[test]
    fn test_performance_summary() {
        record_operation_performance("test_op", 100, 10.0, 5.0, true, true);
        let summary = get_performance_summary();
        assert!(summary.contains_key("total_operations"));
        assert!(summary.contains_key("avg_duration_ms"));
    }
} 