use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field_name: String,
    pub rule_type: ValidationRuleType,
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub allowed_values: Option<Vec<String>>,
    pub custom_validator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRuleType {
    TEXT,
    NUMBER,
    EMAIL,
    URL,
    PHONE,
    DATE,
    JSON,
    ARRAY,
    OBJECT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub quality_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityMetrics {
    pub completeness: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub timeliness: f64,
    pub validity: f64,
    pub overall_score: f64,
}

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    static ref URL_REGEX: Regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
    static ref PHONE_REGEX: Regex = Regex::new(r"^[\+]?[1-9][\d]{0,15}$").unwrap();
    static ref DATE_REGEX: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
}

pub struct DataValidator {
    rules: HashMap<String, Vec<ValidationRule>>,
}

impl DataValidator {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }
    
    pub fn add_schema(&mut self, schema_name: &str, rules: Vec<ValidationRule>) {
        self.rules.insert(schema_name.to_string(), rules);
    }
    
    pub fn validate_text(&self, text: &str, rules: &[ValidationRule]) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        for rule in rules {
            if rule.rule_type != ValidationRuleType::TEXT {
                continue;
            }
            
            // Check required field
            if rule.required && text.trim().is_empty() {
                errors.push(ValidationError {
                    field: rule.field_name.clone(),
                    message: "Field is required".to_string(),
                    severity: ValidationSeverity::HIGH,
                    code: "REQUIRED_FIELD".to_string(),
                });
                continue;
            }
            
            // Check length constraints
            if let Some(min_len) = rule.min_length {
                if text.len() < min_len {
                    errors.push(ValidationError {
                        field: rule.field_name.clone(),
                        message: format!("Minimum length is {} characters", min_len),
                        severity: ValidationSeverity::MEDIUM,
                        code: "MIN_LENGTH".to_string(),
                    });
                }
            }
            
            if let Some(max_len) = rule.max_length {
                if text.len() > max_len {
                    errors.push(ValidationError {
                        field: rule.field_name.clone(),
                        message: format!("Maximum length is {} characters", max_len),
                        severity: ValidationSeverity::MEDIUM,
                        code: "MAX_LENGTH".to_string(),
                    });
                }
            }
            
            // Check pattern
            if let Some(pattern) = &rule.pattern {
                if let Ok(regex) = Regex::new(pattern) {
                    if !regex.is_match(text) {
                        errors.push(ValidationError {
                            field: rule.field_name.clone(),
                            message: "Text does not match required pattern".to_string(),
                            severity: ValidationSeverity::MEDIUM,
                            code: "PATTERN_MISMATCH".to_string(),
                        });
                    }
                }
            }
            
            // Check allowed values
            if let Some(allowed) = &rule.allowed_values {
                if !allowed.contains(&text.to_string()) {
                    errors.push(ValidationError {
                        field: rule.field_name.clone(),
                        message: "Value not in allowed list".to_string(),
                        severity: ValidationSeverity::MEDIUM,
                        code: "INVALID_VALUE".to_string(),
                    });
                }
            }
        }
        
        let quality_score = self.calculate_quality_score(&errors, &warnings);
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            quality_score,
        }
    }
    
    pub fn validate_email(&self, email: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if email.trim().is_empty() {
            errors.push(ValidationError {
                field: "email".to_string(),
                message: "Email is required".to_string(),
                severity: ValidationSeverity::HIGH,
                code: "REQUIRED_FIELD".to_string(),
            });
        } else if !EMAIL_REGEX.is_match(email) {
            errors.push(ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
                severity: ValidationSeverity::HIGH,
                code: "INVALID_EMAIL".to_string(),
            });
        }
        
        // Check for common email issues
        if email.contains(" ") {
            warnings.push(ValidationWarning {
                field: "email".to_string(),
                message: "Email contains spaces".to_string(),
                suggestion: "Remove spaces from email address".to_string(),
            });
        }
        
        if email.to_lowercase() != email {
            warnings.push(ValidationWarning {
                field: "email".to_string(),
                message: "Email contains uppercase letters".to_string(),
                suggestion: "Consider using lowercase email address".to_string(),
            });
        }
        
        let quality_score = self.calculate_quality_score(&errors, &warnings);
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            quality_score,
        }
    }
    
    pub fn validate_numeric_data(&self, data: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        let numbers: Vec<&str> = data.split(|c| c == ',' || c == ' ' || c == '\n' || c == '\t').collect();
        let mut valid_numbers = Vec::new();
        let mut invalid_count = 0;
        
        for (i, num_str) in numbers.iter().enumerate() {
            match num_str.trim().parse::<f64>() {
                Ok(num) => {
                    valid_numbers.push(num);
                    
                    // Check for outliers
                    if valid_numbers.len() > 1 {
                        let mean = valid_numbers.iter().sum::<f64>() / valid_numbers.len() as f64;
                        let std_dev = (valid_numbers.iter()
                            .map(|x| (x - mean).powi(2))
                            .sum::<f64>() / valid_numbers.len() as f64).sqrt();
                        
                        if (num - mean).abs() > 3.0 * std_dev {
                            warnings.push(ValidationWarning {
                                field: format!("data[{}]", i),
                                message: "Potential outlier detected".to_string(),
                                suggestion: "Review this value for accuracy".to_string(),
                            });
                        }
                    }
                },
                Err(_) => {
                    invalid_count += 1;
                    errors.push(ValidationError {
                        field: format!("data[{}]", i),
                        message: format!("Invalid number: {}", num_str),
                        severity: ValidationSeverity::MEDIUM,
                        code: "INVALID_NUMBER".to_string(),
                    });
                }
            }
        }
        
        // Check data quality
        if valid_numbers.is_empty() {
            errors.push(ValidationError {
                field: "data".to_string(),
                message: "No valid numeric data found".to_string(),
                severity: ValidationSeverity::CRITICAL,
                code: "NO_VALID_DATA".to_string(),
            });
        } else if invalid_count > valid_numbers.len() {
            errors.push(ValidationError {
                field: "data".to_string(),
                message: "Too many invalid values".to_string(),
                severity: ValidationSeverity::HIGH,
                code: "LOW_DATA_QUALITY".to_string(),
            });
        }
        
        let quality_score = self.calculate_quality_score(&errors, &warnings);
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            quality_score,
        }
    }
    
    pub fn validate_json(&self, json_str: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Basic JSON structure validation
        if json_str.trim().is_empty() {
            errors.push(ValidationError {
                field: "json".to_string(),
                message: "JSON string is empty".to_string(),
                severity: ValidationSeverity::HIGH,
                code: "EMPTY_JSON".to_string(),
            });
        } else {
            // Check for balanced braces/brackets
            let mut brace_count = 0;
            let mut bracket_count = 0;
            let mut in_string = false;
            let mut escaped = false;
            
            for ch in json_str.chars() {
                if escaped {
                    escaped = false;
                    continue;
                }
                
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                
                if ch == '"' && !escaped {
                    in_string = !in_string;
                    continue;
                }
                
                if !in_string {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => brace_count -= 1,
                        '[' => bracket_count += 1,
                        ']' => bracket_count -= 1,
                        _ => {}
                    }
                }
                
                if brace_count < 0 || bracket_count < 0 {
                    errors.push(ValidationError {
                        field: "json".to_string(),
                        message: "Unbalanced JSON structure".to_string(),
                        severity: ValidationSeverity::HIGH,
                        code: "UNBALANCED_JSON".to_string(),
                    });
                    break;
                }
            }
            
            if brace_count != 0 || bracket_count != 0 {
                errors.push(ValidationError {
                    field: "json".to_string(),
                    message: "Unbalanced JSON structure".to_string(),
                    severity: ValidationSeverity::HIGH,
                    code: "UNBALANCED_JSON".to_string(),
                });
            }
        }
        
        let quality_score = self.calculate_quality_score(&errors, &warnings);
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            quality_score,
        }
    }
    
    pub fn calculate_data_quality_metrics(&self, data: &str, data_type: &str) -> DataQualityMetrics {
        let mut completeness = 1.0;
        let mut accuracy = 1.0;
        let mut consistency = 1.0;
        let mut timeliness = 1.0;
        let mut validity = 1.0;
        
        match data_type {
            "text" => {
                if data.trim().is_empty() {
                    completeness = 0.0;
                }
                
                // Check for repeated patterns
                let words: Vec<&str> = data.split_whitespace().collect();
                let unique_words = words.iter().collect::<std::collections::HashSet<_>>().len();
                if words.len() > 0 {
                    consistency = unique_words as f64 / words.len() as f64;
                }
            },
            "numeric" => {
                let numbers: Vec<&str> = data.split(|c| c == ',' || c == ' ' || c == '\n' || c == '\t').collect();
                let valid_numbers = numbers.iter().filter(|s| s.trim().parse::<f64>().is_ok()).count();
                
                if numbers.len() > 0 {
                    validity = valid_numbers as f64 / numbers.len() as f64;
                    completeness = if valid_numbers > 0 { 1.0 } else { 0.0 };
                } else {
                    completeness = 0.0;
                    validity = 0.0;
                }
            },
            "email" => {
                if data.trim().is_empty() {
                    completeness = 0.0;
                } else if !EMAIL_REGEX.is_match(data) {
                    validity = 0.0;
                }
            },
            _ => {}
        }
        
        let overall_score = (completeness + accuracy + consistency + timeliness + validity) / 5.0;
        
        DataQualityMetrics {
            completeness,
            accuracy,
            consistency,
            timeliness,
            validity,
            overall_score,
        }
    }
    
    fn calculate_quality_score(&self, errors: &[ValidationError], warnings: &[ValidationWarning]) -> f64 {
        let total_issues = errors.len() + warnings.len();
        if total_issues == 0 {
            return 1.0;
        }
        
        let error_weight = 0.7;
        let warning_weight = 0.3;
        
        let error_score = errors.len() as f64 * error_weight;
        let warning_score = warnings.len() as f64 * warning_weight;
        
        let total_score = error_score + warning_score;
        
        (1.0 - (total_score / 10.0)).max(0.0)
    }
}

// Public validation functions
pub fn validate_text_input(text: &str) -> ValidationResult {
    let validator = DataValidator::new();
    let rules = vec![
        ValidationRule {
            field_name: "text".to_string(),
            rule_type: ValidationRuleType::TEXT,
            required: true,
            min_length: Some(1),
            max_length: Some(10000),
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
            custom_validator: None,
        }
    ];
    
    validator.validate_text(text, &rules)
}

pub fn validate_email_input(email: &str) -> ValidationResult {
    let validator = DataValidator::new();
    validator.validate_email(email)
}

pub fn validate_numeric_input(data: &str) -> ValidationResult {
    let validator = DataValidator::new();
    validator.validate_numeric_data(data)
}

pub fn validate_json_input(json: &str) -> ValidationResult {
    let validator = DataValidator::new();
    validator.validate_json(json)
}

pub fn get_data_quality_metrics(data: &str, data_type: &str) -> DataQualityMetrics {
    let validator = DataValidator::new();
    validator.calculate_data_quality_metrics(data, data_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_validation() {
        let result = validate_text_input("Hello world");
        assert!(result.is_valid);
        assert!(result.quality_score > 0.8);
        
        let result = validate_text_input("");
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "REQUIRED_FIELD"));
    }
    
    #[test]
    fn test_email_validation() {
        let result = validate_email_input("test@example.com");
        assert!(result.is_valid);
        
        let result = validate_email_input("invalid-email");
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "INVALID_EMAIL"));
    }
    
    #[test]
    fn test_numeric_validation() {
        let result = validate_numeric_input("1,2,3,4,5");
        assert!(result.is_valid);
        
        let result = validate_numeric_input("1,abc,3,def,5");
        assert!(!result.is_valid);
        assert!(result.errors.len() >= 2);
    }
    
    #[test]
    fn test_json_validation() {
        let result = validate_json_input(r#"{"name": "test", "value": 123}"#);
        assert!(result.is_valid);
        
        let result = validate_json_input(r#"{"name": "test", "value": 123"#);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "UNBALANCED_JSON"));
    }
    
    #[test]
    fn test_data_quality_metrics() {
        let metrics = get_data_quality_metrics("1,2,3,4,5", "numeric");
        assert!(metrics.overall_score > 0.8);
        assert_eq!(metrics.validity, 1.0);
        
        let metrics = get_data_quality_metrics("1,abc,3,def,5", "numeric");
        assert!(metrics.overall_score < 0.8);
        assert!(metrics.validity < 1.0);
    }
} 