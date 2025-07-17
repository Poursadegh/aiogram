use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub module: String,
    pub function: String,
    pub line: u32,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
    pub duration_ms: Option<u64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub operation: String,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub disk_usage_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: String,
    pub uptime_seconds: u64,
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
    pub active_connections: usize,
    pub error_rate: f64,
    pub response_time_avg_ms: f64,
}

lazy_static! {
    static ref LOGGER: Arc<Mutex<Logger>> = Arc::new(Mutex::new(Logger::new()));
    static ref METRICS: Arc<Mutex<MetricsCollector>> = Arc::new(Mutex::new(MetricsCollector::new()));
}

pub struct Logger {
    entries: Vec<LogEntry>,
    max_entries: usize,
    enabled: bool,
    log_level: LogLevel,
}

pub struct MetricsCollector {
    metrics: Vec<PerformanceMetric>,
    max_metrics: usize,
    system_health: SystemHealth,
    start_time: Instant,
}

impl Logger {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 10000,
            enabled: true,
            log_level: LogLevel::INFO,
        }
    }
    
    pub fn log(&mut self, level: LogLevel, message: &str, module: &str, function: &str, line: u32) {
        if !self.enabled || !self.should_log(&level) {
            return;
        }
        
        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            module: module.to_string(),
            function: function.to_string(),
            line,
            user_id: None,
            request_id: None,
            duration_ms: None,
            metadata: HashMap::new(),
        };
        
        self.entries.push(entry);
        
        // Keep only the latest entries
        if self.entries.len() > self.max_entries {
            self.entries.drain(0..self.entries.len() - self.max_entries);
        }
        
        // Print to console in development
        if crate::config::AppConfig::is_production() {
            println!("[{}] {} - {}:{} - {}", 
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                format!("{:?}", entry.level),
                entry.module,
                entry.line,
                entry.message
            );
        }
    }
    
    fn should_log(&self, level: &LogLevel) -> bool {
        match (&self.log_level, level) {
            (LogLevel::DEBUG, _) => true,
            (LogLevel::INFO, LogLevel::INFO | LogLevel::WARN | LogLevel::ERROR | LogLevel::CRITICAL) => true,
            (LogLevel::WARN, LogLevel::WARN | LogLevel::ERROR | LogLevel::CRITICAL) => true,
            (LogLevel::ERROR, LogLevel::ERROR | LogLevel::CRITICAL) => true,
            (LogLevel::CRITICAL, LogLevel::CRITICAL) => true,
            _ => false,
        }
    }
    
    pub fn get_entries(&self, level: Option<LogLevel>, limit: usize) -> Vec<LogEntry> {
        let filtered: Vec<LogEntry> = self.entries.iter()
            .filter(|entry| {
                if let Some(ref filter_level) = level {
                    matches!((&entry.level, filter_level), 
                        (LogLevel::CRITICAL, LogLevel::CRITICAL) |
                        (LogLevel::ERROR, LogLevel::ERROR | LogLevel::CRITICAL) |
                        (LogLevel::WARN, LogLevel::WARN | LogLevel::ERROR | LogLevel::CRITICAL) |
                        (LogLevel::INFO, LogLevel::INFO | LogLevel::WARN | LogLevel::ERROR | LogLevel::CRITICAL) |
                        (LogLevel::DEBUG, _)
                    )
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        
        filtered.into_iter().rev().take(limit).collect()
    }
    
    pub fn clear_entries(&mut self) {
        self.entries.clear();
    }
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            metrics: Vec::new(),
            max_metrics: 10000,
            system_health: SystemHealth {
                status: "healthy".to_string(),
                uptime_seconds: 0,
                memory_usage_percent: 0.0,
                cpu_usage_percent: 0.0,
                active_connections: 0,
                error_rate: 0.0,
                response_time_avg_ms: 0.0,
            },
            start_time: Instant::now(),
        }
    }
    
    pub fn record_metric(&mut self, operation: &str, duration_ms: u64, success: bool, error_message: Option<String>) {
        let metric = PerformanceMetric {
            operation: operation.to_string(),
            duration_ms,
            timestamp: Utc::now(),
            success,
            error_message,
            resource_usage: self.get_current_resource_usage(),
        };
        
        self.metrics.push(metric);
        
        // Keep only the latest metrics
        if self.metrics.len() > self.max_metrics {
            self.metrics.drain(0..self.metrics.len() - self.max_metrics);
        }
        
        self.update_system_health();
    }
    
    fn get_current_resource_usage(&self) -> ResourceUsage {
        // Simplified resource monitoring
        ResourceUsage {
            memory_mb: 0.0, // Would integrate with system monitoring
            cpu_percent: 0.0,
            disk_usage_mb: 0.0,
        }
    }
    
    fn update_system_health(&mut self) {
        let uptime = self.start_time.elapsed().as_secs();
        let recent_metrics: Vec<&PerformanceMetric> = self.metrics.iter()
            .filter(|m| m.timestamp > Utc::now() - chrono::Duration::minutes(5))
            .collect();
        
        let error_count = recent_metrics.iter().filter(|m| !m.success).count();
        let total_count = recent_metrics.len();
        let error_rate = if total_count > 0 {
            error_count as f64 / total_count as f64
        } else {
            0.0
        };
        
        let avg_response_time = if !recent_metrics.is_empty() {
            recent_metrics.iter().map(|m| m.duration_ms as f64).sum::<f64>() / recent_metrics.len() as f64
        } else {
            0.0
        };
        
        self.system_health = SystemHealth {
            status: if error_rate < 0.05 { "healthy".to_string() } else { "degraded".to_string() },
            uptime_seconds: uptime,
            memory_usage_percent: 0.0, // Would integrate with system monitoring
            cpu_usage_percent: 0.0,
            active_connections: 0,
            error_rate,
            response_time_avg_ms: avg_response_time,
        };
    }
    
    pub fn get_system_health(&self) -> SystemHealth {
        self.system_health.clone()
    }
    
    pub fn get_metrics(&self, operation: Option<&str>, limit: usize) -> Vec<PerformanceMetric> {
        let filtered: Vec<PerformanceMetric> = self.metrics.iter()
            .filter(|metric| {
                if let Some(ref op) = operation {
                    metric.operation == *op
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        
        filtered.into_iter().rev().take(limit).collect()
    }
}

// Public logging functions
pub fn debug(message: &str, module: &str, function: &str, line: u32) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log(LogLevel::DEBUG, message, module, function, line);
    }
}

pub fn info(message: &str, module: &str, function: &str, line: u32) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log(LogLevel::INFO, message, module, function, line);
    }
}

pub fn warn(message: &str, module: &str, function: &str, line: u32) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log(LogLevel::WARN, message, module, function, line);
    }
}

pub fn error(message: &str, module: &str, function: &str, line: u32) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log(LogLevel::ERROR, message, module, function, line);
    }
}

pub fn critical(message: &str, module: &str, function: &str, line: u32) {
    if let Ok(mut logger) = LOGGER.lock() {
        logger.log(LogLevel::CRITICAL, message, module, function, line);
    }
}

// Performance monitoring functions
pub fn record_performance(operation: &str, duration_ms: u64, success: bool, error_message: Option<String>) {
    if let Ok(mut metrics) = METRICS.lock() {
        metrics.record_metric(operation, duration_ms, success, error_message);
    }
}

pub fn get_system_health() -> SystemHealth {
    if let Ok(metrics) = METRICS.lock() {
        metrics.get_system_health()
    } else {
        SystemHealth {
            status: "unknown".to_string(),
            uptime_seconds: 0,
            memory_usage_percent: 0.0,
            cpu_usage_percent: 0.0,
            active_connections: 0,
            error_rate: 0.0,
            response_time_avg_ms: 0.0,
        }
    }
}

pub fn get_recent_logs(level: Option<LogLevel>, limit: usize) -> Vec<LogEntry> {
    if let Ok(logger) = LOGGER.lock() {
        logger.get_entries(level, limit)
    } else {
        Vec::new()
    }
}

pub fn get_performance_metrics(operation: Option<&str>, limit: usize) -> Vec<PerformanceMetric> {
    if let Ok(metrics) = METRICS.lock() {
        metrics.get_metrics(operation, limit)
    } else {
        Vec::new()
    }
}

// Macro for easier logging
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logging::debug(&format!($($arg)*), module_path!(), function_name!(), line!())
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logging::info(&format!($($arg)*), module_path!(), function_name!(), line!())
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logging::warn(&format!($($arg)*), module_path!(), function_name!(), line!())
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logging::error(&format!($($arg)*), module_path!(), function_name!(), line!())
    };
}

#[macro_export]
macro_rules! log_critical {
    ($($arg:tt)*) => {
        $crate::logging::critical(&format!($($arg)*), module_path!(), function_name!(), line!())
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logging() {
        info("Test info message", "test", "test_function", 1);
        warn("Test warning message", "test", "test_function", 2);
        error("Test error message", "test", "test_function", 3);
        
        let logs = get_recent_logs(None, 10);
        assert!(!logs.is_empty());
    }
    
    #[test]
    fn test_performance_monitoring() {
        record_performance("test_operation", 100, true, None);
        record_performance("test_operation", 200, false, Some("Test error".to_string()));
        
        let metrics = get_performance_metrics(Some("test_operation"), 10);
        assert_eq!(metrics.len(), 2);
    }
    
    #[test]
    fn test_system_health() {
        let health = get_system_health();
        assert!(!health.status.is_empty());
        assert!(health.uptime_seconds >= 0);
    }
} 