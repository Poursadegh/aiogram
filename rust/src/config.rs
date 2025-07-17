use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::RwLock;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub max_text_length: usize,
    pub max_data_points: usize,
    pub sentiment_threshold: f64,
    pub language_confidence_threshold: f64,
    pub plagiarism_threshold: f64,
    pub cache_enabled: bool,
    pub cache_ttl_seconds: u64,
    pub rate_limit_requests_per_minute: u32,
    pub enable_logging: bool,
    pub log_level: String,
    pub performance_monitoring: bool,
    pub security_enabled: bool,
    pub allowed_languages: Vec<String>,
    pub custom_stop_words: Vec<String>,
    pub api_keys: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub key_rotation_days: u32,
    pub max_key_age_days: u32,
    pub allowed_origins: Vec<String>,
    pub rate_limit_enabled: bool,
    pub max_request_size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_requests: usize,
    pub worker_threads: usize,
    pub memory_limit_mb: usize,
    pub timeout_seconds: u64,
    pub enable_profiling: bool,
    pub cache_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub analysis: AnalysisConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub environment: String,
    pub version: String,
}

lazy_static! {
    static ref CONFIG: RwLock<AppConfig> = RwLock::new(AppConfig::default());
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            analysis: AnalysisConfig {
                max_text_length: 10000,
                max_data_points: 10000,
                sentiment_threshold: 0.2,
                language_confidence_threshold: 0.7,
                plagiarism_threshold: 0.8,
                cache_enabled: true,
                cache_ttl_seconds: 3600,
                rate_limit_requests_per_minute: 100,
                enable_logging: true,
                log_level: "info".to_string(),
                performance_monitoring: true,
                security_enabled: true,
                allowed_languages: vec![
                    "en".to_string(), "es".to_string(), "fr".to_string(), 
                    "de".to_string(), "it".to_string(), "pt".to_string(),
                    "ru".to_string(), "zh".to_string(), "ja".to_string(), "ko".to_string(),
                    "ar".to_string(), "hi".to_string(), "fa".to_string()
                ],
                custom_stop_words: vec![],
                api_keys: HashMap::new(),
            },
            security: SecurityConfig {
                encryption_enabled: true,
                key_rotation_days: 30,
                max_key_age_days: 90,
                allowed_origins: vec!["*".to_string()],
                rate_limit_enabled: true,
                max_request_size_bytes: 1024 * 1024, // 1MB
            },
            performance: PerformanceConfig {
                max_concurrent_requests: 100,
                worker_threads: num_cpus::get(),
                memory_limit_mb: 512,
                timeout_seconds: 30,
                enable_profiling: false,
                cache_size_mb: 100,
            },
            environment: "development".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl AppConfig {
    pub fn load_from_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if Path::new(path).exists() {
            let config_content = fs::read_to_string(path)?;
            let config: AppConfig = serde_json::from_str(&config_content)?;
            let mut global_config = CONFIG.write().unwrap();
            *global_config = config;
        }
        Ok(())
    }
    
    pub fn get() -> AppConfig {
        CONFIG.read().unwrap().clone()
    }
    
    pub fn update(updates: AppConfig) {
        let mut config = CONFIG.write().unwrap();
        *config = updates;
    }
    
    pub fn get_analysis_config() -> AnalysisConfig {
        CONFIG.read().unwrap().analysis.clone()
    }
    
    pub fn get_security_config() -> SecurityConfig {
        CONFIG.read().unwrap().security.clone()
    }
    
    pub fn get_performance_config() -> PerformanceConfig {
        CONFIG.read().unwrap().performance.clone()
    }
    
    pub fn is_production() -> bool {
        CONFIG.read().unwrap().environment == "production"
    }
    
    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.analysis.max_text_length == 0 {
            errors.push("max_text_length must be greater than 0".to_string());
        }
        
        if self.analysis.max_data_points == 0 {
            errors.push("max_data_points must be greater than 0".to_string());
        }
        
        if self.performance.worker_threads == 0 {
            errors.push("worker_threads must be greater than 0".to_string());
        }
        
        if self.performance.memory_limit_mb == 0 {
            errors.push("memory_limit_mb must be greater than 0".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn initialize_config(config_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = config_path {
        AppConfig::load_from_file(path)?;
    }
    
    let config = AppConfig::get();
    config.validate_config()?;
    
    println!("Configuration loaded successfully");
    println!("Environment: {}", config.environment);
    println!("Version: {}", config.version);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.analysis.max_text_length > 0);
        assert!(config.performance.worker_threads > 0);
        assert!(config.validate_config().is_ok());
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.analysis.max_text_length = 0;
        assert!(config.validate_config().is_err());
    }
    
    #[test]
    fn test_environment_detection() {
        let config = AppConfig::default();
        assert!(!AppConfig::is_production());
    }
} 