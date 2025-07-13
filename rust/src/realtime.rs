use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rayon::prelude::*;
use dashmap::DashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct RealtimeResult {
    pub status: String,
    pub processing_speed: f64,
    pub quality: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealtimeData {
    pub timestamp: f64,
    pub user_id: u64,
    pub data_type: String,
    pub content: String,
}

// Global state for real-time processing
lazy_static::lazy_static! {
    static ref PROCESSING_STATS: Arc<DashMap<String, ProcessingStats>> = Arc::new(DashMap::new());
    static ref DATA_BUFFER: Arc<Mutex<Vec<RealtimeData>>> = Arc::new(Mutex::new(Vec::new()));
}

#[derive(Debug, Clone)]
struct ProcessingStats {
    total_processed: u64,
    avg_processing_time: f64,
    last_update: Instant,
}

impl ProcessingStats {
    fn new() -> Self {
        Self {
            total_processed: 0,
            avg_processing_time: 0.0,
            last_update: Instant::now(),
        }
    }
    
    fn update(&mut self, processing_time: f64) {
        self.total_processed += 1;
        let alpha = 0.1; // Exponential moving average
        self.avg_processing_time = alpha * processing_time + (1.0 - alpha) * self.avg_processing_time;
        self.last_update = Instant::now();
    }
}

pub fn process_realtime_data(data_json: &str) -> RealtimeResult {
    let start_time = Instant::now();
    
    // Parse input data
    let data: RealtimeData = match serde_json::from_str(data_json) {
        Ok(d) => d,
        Err(_) => {
            return RealtimeResult {
                status: "error".to_string(),
                processing_speed: 0.0,
                quality: "invalid_data".to_string(),
                timestamp: Utc::now().timestamp(),
            };
        }
    };
    
    // Store data in buffer
    {
        let mut buffer = DATA_BUFFER.lock().unwrap();
        buffer.push(data.clone());
        
        // Keep only last 1000 items
        if buffer.len() > 1000 {
            buffer.drain(0..buffer.len() - 1000);
        }
    }
    
    // Process data with different algorithms based on type
    let processing_result = match data.data_type.as_str() {
        "telegram_message" => process_telegram_message(&data),
        "numeric_data" => process_numeric_data(&data),
        "text_data" => process_text_data(&data),
        _ => process_generic_data(&data),
    };
    
    // Update processing statistics
    let processing_time = start_time.elapsed().as_millis() as f64;
    let mut stats = PROCESSING_STATS
        .entry(data.data_type.clone())
        .or_insert_with(ProcessingStats::new);
    stats.update(processing_time);
    
    // Calculate processing speed (operations per second)
    let processing_speed = if processing_time > 0.0 {
        1000.0 / processing_time
    } else {
        0.0
    };
    
    // Determine quality based on processing time and data characteristics
    let quality = determine_quality(processing_time, &data);
    
    RealtimeResult {
        status: processing_result.status,
        processing_speed,
        quality,
        timestamp: Utc::now().timestamp(),
    }
}

fn process_telegram_message(data: &RealtimeData) -> ProcessingResult {
    // Simulate message processing with NLP tasks
    let words: Vec<&str> = data.content.split_whitespace().collect();
    let word_count = words.len();
    
    // Parallel processing of words
    let processed_words: Vec<String> = words
        .par_iter()
        .map(|word| {
            // Simulate complex word processing
            let mut processed = word.to_lowercase();
            processed = processed.chars().filter(|c| c.is_alphanumeric()).collect();
            processed
        })
        .collect();
    
    // Calculate message complexity
    let complexity = calculate_complexity(&processed_words);
    
    ProcessingResult {
        status: "processed".to_string(),
        complexity,
        word_count,
        processing_notes: vec!["NLP processing completed".to_string()],
    }
}

fn process_numeric_data(data: &RealtimeData) -> ProcessingResult {
    // Parse numeric data
    let numbers: Vec<f64> = data.content
        .split(|c| c == ',' || c == ' ' || c == '\n')
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .collect();
    
    if numbers.is_empty() {
        return ProcessingResult {
            status: "error".to_string(),
            complexity: 0.0,
            word_count: 0,
            processing_notes: vec!["No valid numeric data found".to_string()],
        };
    }
    
    // Parallel statistical analysis
    let (mean, std_dev) = rayon::join(
        || numbers.iter().sum::<f64>() / numbers.len() as f64,
        || {
            let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
            let variance = numbers.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / numbers.len() as f64;
            variance.sqrt()
        }
    );
    
    let complexity = std_dev / mean; // Coefficient of variation
    
    ProcessingResult {
        status: "processed".to_string(),
        complexity,
        word_count: numbers.len(),
        processing_notes: vec![
            format!("Mean: {:.2}", mean),
            format!("Std Dev: {:.2}", std_dev),
            "Statistical analysis completed".to_string(),
        ],
    }
}

fn process_text_data(data: &RealtimeData) -> ProcessingResult {
    let text = &data.content;
    
    // Parallel text analysis
    let (char_count, word_count, sentence_count) = rayon::join(
        || text.chars().count(),
        || text.split_whitespace().count(),
        || text.split(|c| c == '.' || c == '!' || c == '?').count()
    );
    
    // Calculate text complexity
    let complexity = if word_count > 0 {
        (char_count as f64 / word_count as f64) * (sentence_count as f64 / word_count as f64)
    } else {
        0.0
    };
    
    ProcessingResult {
        status: "processed".to_string(),
        complexity,
        word_count,
        processing_notes: vec![
            format!("Characters: {}", char_count),
            format!("Words: {}", word_count),
            format!("Sentences: {}", sentence_count),
            "Text analysis completed".to_string(),
        ],
    }
}

fn process_generic_data(data: &RealtimeData) -> ProcessingResult {
    // Generic data processing
    let content_length = data.content.len();
    let complexity = content_length as f64 / 100.0; // Normalize complexity
    
    ProcessingResult {
        status: "processed".to_string(),
        complexity,
        word_count: content_length,
        processing_notes: vec!["Generic processing completed".to_string()],
    }
}

#[derive(Debug)]
struct ProcessingResult {
    status: String,
    complexity: f64,
    word_count: usize,
    processing_notes: Vec<String>,
}

fn calculate_complexity(words: &[String]) -> f64 {
    if words.is_empty() {
        return 0.0;
    }
    
    // Calculate various complexity metrics
    let avg_word_length: f64 = words.iter()
        .map(|word| word.len() as f64)
        .sum::<f64>() / words.len() as f64;
    
    let unique_words = words.iter().collect::<std::collections::HashSet<_>>().len();
    let vocabulary_richness = unique_words as f64 / words.len() as f64;
    
    // Combine metrics
    avg_word_length * vocabulary_richness
}

fn determine_quality(processing_time: f64, data: &RealtimeData) -> String {
    let content_length = data.content.len();
    
    // Quality based on processing time and data size
    let efficiency = if processing_time > 0.0 {
        content_length as f64 / processing_time
    } else {
        0.0
    };
    
    match efficiency {
        e if e > 1000.0 => "excellent".to_string(),
        e if e > 500.0 => "good".to_string(),
        e if e > 100.0 => "fair".to_string(),
        _ => "poor".to_string(),
    }
}

// Additional real-time processing utilities
pub fn get_processing_stats() -> HashMap<String, ProcessingStats> {
    PROCESSING_STATS.iter().map(|entry| {
        (entry.key().clone(), entry.value().clone())
    }).collect()
}

pub fn clear_data_buffer() {
    let mut buffer = DATA_BUFFER.lock().unwrap();
    buffer.clear();
}

pub fn get_buffer_size() -> usize {
    DATA_BUFFER.lock().unwrap().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_realtime_processing() {
        let data = RealtimeData {
            timestamp: 1234567890.0,
            user_id: 12345,
            data_type: "telegram_message".to_string(),
            content: "Hello world! This is a test message.".to_string(),
        };
        
        let json_data = serde_json::to_string(&data).unwrap();
        let result = process_realtime_data(&json_data);
        
        assert_eq!(result.status, "processed");
        assert!(result.processing_speed > 0.0);
        assert!(!result.quality.is_empty());
    }
    
    #[test]
    fn test_numeric_processing() {
        let data = RealtimeData {
            timestamp: 1234567890.0,
            user_id: 12345,
            data_type: "numeric_data".to_string(),
            content: "1,2,3,4,5,6,7,8,9,10".to_string(),
        };
        
        let json_data = serde_json::to_string(&data).unwrap();
        let result = process_realtime_data(&json_data);
        
        assert_eq!(result.status, "processed");
        assert!(result.processing_speed > 0.0);
    }
    
    #[test]
    fn test_complexity_calculation() {
        let words = vec!["hello".to_string(), "world".to_string(), "test".to_string()];
        let complexity = calculate_complexity(&words);
        
        assert!(complexity > 0.0);
    }
} 