use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use dashmap::DashMap;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub requests: u32,
    pub window_start: Instant,
    pub blocked_until: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: Instant,
    pub event_type: String,
    pub source_ip: Option<String>,
    pub user_id: Option<String>,
    pub details: String,
    pub severity: SecuritySeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub max_requests_per_minute: u32,
    pub max_request_size_bytes: usize,
    pub enable_input_validation: bool,
    pub enable_threat_detection: bool,
    pub blocked_ips: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub max_concurrent_connections: usize,
}

pub struct SecurityManager {
    rate_limits: DashMap<String, RateLimitInfo>,
    security_events: Arc<Mutex<Vec<SecurityEvent>>>,
    blocked_ips: DashMap<String, Instant>,
    config: SecurityConfig,
    threat_patterns: Vec<Regex>,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Self {
        let threat_patterns = vec![
            Regex::new(r"(?i)(script|javascript|vbscript|onload|onerror)").unwrap(),
            Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter)").unwrap(),
            Regex::new(r"(?i)(eval|exec|system|shell|cmd)").unwrap(),
            Regex::new(r"(?i)(<script|javascript:|vbscript:)").unwrap(),
        ];
        
        Self {
            rate_limits: DashMap::new(),
            security_events: Arc::new(Mutex::new(Vec::new())),
            blocked_ips: DashMap::new(),
            config,
            threat_patterns,
        }
    }
    
    pub fn check_rate_limit(&self, identifier: &str) -> bool {
        let now = Instant::now();
        let window_duration = Duration::from_secs(60);
        
        if let Some(mut info) = self.rate_limits.get_mut(identifier) {
            // Check if still blocked
            if let Some(blocked_until) = info.blocked_until {
                if now < blocked_until {
                    return false;
                }
            }
            
            // Check if window has expired
            if now.duration_since(info.window_start) > window_duration {
                info.requests = 1;
                info.window_start = now;
                info.blocked_until = None;
                return true;
            }
            
            // Check if limit exceeded
            if info.requests >= self.config.max_requests_per_minute {
                info.blocked_until = Some(now + Duration::from_secs(300)); // 5 minute block
                self.record_security_event(
                    "RATE_LIMIT_EXCEEDED",
                    Some(identifier.to_string()),
                    None,
                    format!("Rate limit exceeded for {}", identifier),
                    SecuritySeverity::MEDIUM,
                );
                return false;
            }
            
            info.requests += 1;
            true
        } else {
            // First request
            let info = RateLimitInfo {
                requests: 1,
                window_start: now,
                blocked_until: None,
            };
            self.rate_limits.insert(identifier.to_string(), info);
            true
        }
    }
    
    pub fn validate_input(&self, input: &str, input_type: &str) -> Result<(), String> {
        if !self.config.enable_input_validation {
            return Ok(());
        }
        
        // Check for threat patterns
        for pattern in &self.threat_patterns {
            if pattern.is_match(input) {
                self.record_security_event(
                    "THREAT_DETECTED",
                    None,
                    None,
                    format!("Threat pattern detected in {}: {}", input_type, input),
                    SecuritySeverity::HIGH,
                );
                return Err(format!("Invalid input detected in {}", input_type));
            }
        }
        
        // Check input size
        if input.len() > self.config.max_request_size_bytes {
            return Err("Input size exceeds maximum allowed size".to_string());
        }
        
        // Check for null bytes
        if input.contains('\0') {
            return Err("Input contains null bytes".to_string());
        }
        
        // Validate based on input type
        match input_type {
            "text" => self.validate_text_input(input)?,
            "data" => self.validate_data_input(input)?,
            "json" => self.validate_json_input(input)?,
            _ => Ok(()),
        }
    }
    
    fn validate_text_input(&self, input: &str) -> Result<(), String> {
        // Check for excessive whitespace
        if input.chars().filter(|c| c.is_whitespace()).count() > input.len() / 2 {
            return Err("Input contains excessive whitespace".to_string());
        }
        
        // Check for repeated characters
        let mut prev_char = None;
        let mut repeat_count = 0;
        for ch in input.chars() {
            if Some(ch) == prev_char {
                repeat_count += 1;
                if repeat_count > 10 {
                    return Err("Input contains excessive repeated characters".to_string());
                }
            } else {
                repeat_count = 0;
            }
            prev_char = Some(ch);
        }
        
        Ok(())
    }
    
    fn validate_data_input(&self, input: &str) -> Result<(), String> {
        // Check if input contains valid numeric data
        let numbers: Vec<&str> = input.split(|c| c == ',' || c == ' ' || c == '\n' || c == '\t').collect();
        let valid_numbers = numbers.iter().filter(|s| s.trim().parse::<f64>().is_ok()).count();
        
        if valid_numbers == 0 {
            return Err("No valid numeric data found".to_string());
        }
        
        if valid_numbers < numbers.len() / 2 {
            return Err("Too many invalid numeric values".to_string());
        }
        
        Ok(())
    }
    
    fn validate_json_input(&self, input: &str) -> Result<(), String> {
        // Basic JSON validation
        if !input.trim().starts_with('{') && !input.trim().starts_with('[') {
            return Err("Invalid JSON format".to_string());
        }
        
        // Check for balanced braces/brackets
        let mut brace_count = 0;
        let mut bracket_count = 0;
        
        for ch in input.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                _ => {}
            }
            
            if brace_count < 0 || bracket_count < 0 {
                return Err("Unbalanced JSON structure".to_string());
            }
        }
        
        if brace_count != 0 || bracket_count != 0 {
            return Err("Unbalanced JSON structure".to_string());
        }
        
        Ok(())
    }
    
    pub fn is_ip_blocked(&self, ip: &str) -> bool {
        if let Some(blocked_until) = self.blocked_ips.get(ip) {
            if Instant::now() < *blocked_until {
                return true;
            } else {
                self.blocked_ips.remove(ip);
            }
        }
        false
    }
    
    pub fn block_ip(&self, ip: &str, duration_seconds: u64) {
        let blocked_until = Instant::now() + Duration::from_secs(duration_seconds);
        self.blocked_ips.insert(ip.to_string(), blocked_until);
        
        self.record_security_event(
            "IP_BLOCKED",
            Some(ip.to_string()),
            None,
            format!("IP {} blocked for {} seconds", ip, duration_seconds),
            SecuritySeverity::MEDIUM,
        );
    }
    
    pub fn record_security_event(&self, event_type: &str, source_ip: Option<String>, user_id: Option<String>, details: String, severity: SecuritySeverity) {
        let event = SecurityEvent {
            timestamp: Instant::now(),
            event_type: event_type.to_string(),
            source_ip,
            user_id,
            details,
            severity,
        };
        
        if let Ok(mut events) = self.security_events.lock() {
            events.push(event);
            
            // Keep only last 1000 events
            if events.len() > 1000 {
                events.drain(0..events.len() - 1000);
            }
        }
    }
    
    pub fn get_security_events(&self, severity: Option<SecuritySeverity>, limit: usize) -> Vec<SecurityEvent> {
        if let Ok(events) = self.security_events.lock() {
            let filtered: Vec<SecurityEvent> = events.iter()
                .filter(|event| {
                    if let Some(ref filter_severity) = severity {
                        matches!((&event.severity, filter_severity),
                            (SecuritySeverity::CRITICAL, SecuritySeverity::CRITICAL) |
                            (SecuritySeverity::HIGH, SecuritySeverity::HIGH | SecuritySeverity::CRITICAL) |
                            (SecuritySeverity::MEDIUM, SecuritySeverity::MEDIUM | SecuritySeverity::HIGH | SecuritySeverity::CRITICAL) |
                            (SecuritySeverity::LOW, _)
                        )
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();
            
            filtered.into_iter().rev().take(limit).collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn get_rate_limit_info(&self, identifier: &str) -> Option<RateLimitInfo> {
        self.rate_limits.get(identifier).map(|info| info.clone())
    }
    
    pub fn clear_rate_limits(&self) {
        self.rate_limits.clear();
    }
    
    pub fn get_blocked_ips(&self) -> Vec<String> {
        self.blocked_ips.iter().map(|entry| entry.key().clone()).collect()
    }
}

// Global security manager
lazy_static! {
    static ref SECURITY_MANAGER: Arc<SecurityManager> = Arc::new(SecurityManager::new(SecurityConfig {
        max_requests_per_minute: 100,
        max_request_size_bytes: 1024 * 1024, // 1MB
        enable_input_validation: true,
        enable_threat_detection: true,
        blocked_ips: vec![],
        allowed_origins: vec!["*".to_string()],
        max_concurrent_connections: 1000,
    }));
}

// Public security functions
pub fn check_rate_limit(identifier: &str) -> bool {
    SECURITY_MANAGER.check_rate_limit(identifier)
}

pub fn validate_input(input: &str, input_type: &str) -> Result<(), String> {
    SECURITY_MANAGER.validate_input(input, input_type)
}

pub fn is_ip_blocked(ip: &str) -> bool {
    SECURITY_MANAGER.is_ip_blocked(ip)
}

pub fn block_ip(ip: &str, duration_seconds: u64) {
    SECURITY_MANAGER.block_ip(ip, duration_seconds);
}

pub fn record_security_event(event_type: &str, source_ip: Option<String>, user_id: Option<String>, details: String, severity: SecuritySeverity) {
    SECURITY_MANAGER.record_security_event(event_type, source_ip, user_id, details, severity);
}

pub fn get_security_events(severity: Option<SecuritySeverity>, limit: usize) -> Vec<SecurityEvent> {
    SECURITY_MANAGER.get_security_events(severity, limit)
}

pub fn get_rate_limit_info(identifier: &str) -> Option<RateLimitInfo> {
    SECURITY_MANAGER.get_rate_limit_info(identifier)
}

pub fn get_blocked_ips() -> Vec<String> {
    SECURITY_MANAGER.get_blocked_ips()
}

// Utility functions
pub fn sanitize_input(input: &str) -> String {
    // Remove null bytes and control characters
    input.chars()
        .filter(|c| *c != '\0' && !c.is_control())
        .collect()
}

pub fn generate_request_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u64 = rng.gen();
    format!("req_{:x}", id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rate_limiting() {
        let manager = SecurityManager::new(SecurityConfig {
            max_requests_per_minute: 5,
            max_request_size_bytes: 1000,
            enable_input_validation: true,
            enable_threat_detection: true,
            blocked_ips: vec![],
            allowed_origins: vec!["*".to_string()],
            max_concurrent_connections: 100,
        });
        
        // Should allow first 5 requests
        for i in 0..5 {
            assert!(manager.check_rate_limit("test_user"));
        }
        
        // 6th request should be blocked
        assert!(!manager.check_rate_limit("test_user"));
    }
    
    #[test]
    fn test_input_validation() {
        let manager = SecurityManager::new(SecurityConfig {
            max_requests_per_minute: 100,
            max_request_size_bytes: 1000,
            enable_input_validation: true,
            enable_threat_detection: true,
            blocked_ips: vec![],
            allowed_origins: vec!["*".to_string()],
            max_concurrent_connections: 100,
        });
        
        // Valid input
        assert!(manager.validate_input("Hello world", "text").is_ok());
        
        // Invalid input with script tag
        assert!(manager.validate_input("<script>alert('xss')</script>", "text").is_err());
        
        // Valid numeric data
        assert!(manager.validate_input("1,2,3,4,5", "data").is_ok());
        
        // Invalid numeric data
        assert!(manager.validate_input("abc,def,ghi", "data").is_err());
    }
    
    #[test]
    fn test_ip_blocking() {
        let manager = SecurityManager::new(SecurityConfig {
            max_requests_per_minute: 100,
            max_request_size_bytes: 1000,
            enable_input_validation: true,
            enable_threat_detection: true,
            blocked_ips: vec![],
            allowed_origins: vec!["*".to_string()],
            max_concurrent_connections: 100,
        });
        
        assert!(!manager.is_ip_blocked("192.168.1.1"));
        manager.block_ip("192.168.1.1", 60);
        assert!(manager.is_ip_blocked("192.168.1.1"));
    }
    
    #[test]
    fn test_security_events() {
        let manager = SecurityManager::new(SecurityConfig {
            max_requests_per_minute: 100,
            max_request_size_bytes: 1000,
            enable_input_validation: true,
            enable_threat_detection: true,
            blocked_ips: vec![],
            allowed_origins: vec!["*".to_string()],
            max_concurrent_connections: 100,
        });
        
        manager.record_security_event(
            "TEST_EVENT",
            Some("192.168.1.1".to_string()),
            Some("user123".to_string()),
            "Test security event".to_string(),
            SecuritySeverity::MEDIUM,
        );
        
        let events = manager.get_security_events(None, 10);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "TEST_EVENT");
    }
} 