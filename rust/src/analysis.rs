use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;
use whatlang::Lang;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use ndarray::{Array1, Array2};
use statrs::statistics::Statistics;

#[derive(Debug, Serialize, Deserialize)]
pub struct TextAnalysisResult {
    pub char_count: usize,
    pub word_count: usize,
    pub sentence_count: usize,
    pub language: String,
    pub sentiment: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataAnalysisResult {
    pub record_count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub patterns: Vec<String>,
    pub anomalies: Vec<f64>,
    pub prediction: f64,
}

pub fn analyze_text(text: &str) -> TextAnalysisResult {
    // Character count
    let char_count = text.chars().count();
    
    // Word count using Unicode segmentation
    let words: Vec<&str> = text.unicode_words().collect();
    let word_count = words.len();
    
    // Sentence count using regex
    let sentence_regex = Regex::new(r"[.!?]+").unwrap();
    let sentences: Vec<&str> = sentence_regex.split(text).collect();
    let sentence_count = sentences.len().max(1);
    
    // Language detection
    let language = match whatlang::detect(text) {
        Some(info) => info.lang().to_string(),
        None => "unknown".to_string(),
    };
    
    // Simple sentiment analysis
    let sentiment = analyze_sentiment(text);
    
    // Keyword extraction
    let keywords = extract_keywords(text);
    
    TextAnalysisResult {
        char_count,
        word_count,
        sentence_count,
        language,
        sentiment,
        keywords,
    }
}

fn analyze_sentiment(text: &str) -> String {
    let positive_words = vec![
        "خوب", "عالی", "عالیه", "ممتاز", "عالی", "خوب", "عالی", "عالیه", "ممتاز",
        "good", "great", "excellent", "amazing", "wonderful", "fantastic", "perfect",
        "beautiful", "nice", "lovely", "happy", "joy", "love", "like", "enjoy"
    ];
    
    let negative_words = vec![
        "بد", "بدی", "بدیه", "بدی", "بد", "بدی", "بدیه", "بدی", "بد",
        "bad", "terrible", "awful", "horrible", "disgusting", "hate", "dislike",
        "sad", "angry", "furious", "upset", "disappointed", "worried", "scared"
    ];
    
    let text_lower = text.to_lowercase();
    let words: Vec<&str> = text_lower.unicode_words().collect();
    
    let positive_count = words.iter()
        .filter(|word| positive_words.contains(word))
        .count();
    
    let negative_count = words.iter()
        .filter(|word| negative_words.contains(word))
        .count();
    
    match positive_count.cmp(&negative_count) {
        std::cmp::Ordering::Greater => "positive".to_string(),
        std::cmp::Ordering::Less => "negative".to_string(),
        std::cmp::Ordering::Equal => "neutral".to_string(),
    }
}

fn extract_keywords(text: &str) -> Vec<String> {
    let stop_words = vec![
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
        "این", "آن", "که", "را", "در", "به", "از", "با", "برای", "تا", "یا", "و", "اما"
    ];
    
    let text_lower = text.to_lowercase();
    let words: Vec<&str> = text_lower.unicode_words().collect();
    
    // Count word frequencies
    let mut word_freq: HashMap<&str, usize> = HashMap::new();
    for word in words.iter() {
        if !stop_words.contains(word) && word.len() > 2 {
            *word_freq.entry(word).or_insert(0) += 1;
        }
    }
    
    // Get top 5 keywords
    let mut keywords: Vec<(&&str, &usize)> = word_freq.iter().collect();
    keywords.sort_by(|a, b| b.1.cmp(a.1));
    
    keywords.into_iter()
        .take(5)
        .map(|(word, _)| word.to_string())
        .collect()
}

pub fn analyze_data(data: &str) -> DataAnalysisResult {
    // Parse data as numbers (comma-separated or space-separated)
    let numbers: Vec<f64> = data
        .split(|c| c == ',' || c == ' ' || c == '\n' || c == '\t')
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .collect();
    
    if numbers.is_empty() {
        return DataAnalysisResult {
            record_count: 0,
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            patterns: vec!["No valid numeric data found".to_string()],
            anomalies: vec![],
            prediction: 0.0,
        };
    }
    
    let record_count = numbers.len();
    let mean = numbers.iter().sum::<f64>() / record_count as f64;
    
    // Calculate standard deviation
    let variance = numbers.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / record_count as f64;
    let std_dev = variance.sqrt();
    
    let min = numbers.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = numbers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    // Detect patterns
    let patterns = detect_patterns(&numbers);
    
    // Detect anomalies (values more than 2 standard deviations from mean)
    let anomalies: Vec<f64> = numbers.iter()
        .filter(|&&x| (x - mean).abs() > 2.0 * std_dev)
        .cloned()
        .collect();
    
    // Simple prediction (linear trend)
    let prediction = if numbers.len() > 1 {
        let x_values: Vec<f64> = (0..numbers.len()).map(|i| i as f64).collect();
        let slope = calculate_slope(&x_values, &numbers);
        let last_value = numbers.last().unwrap();
        let next_x = numbers.len() as f64;
        last_value + slope * (next_x - (numbers.len() - 1) as f64)
    } else {
        mean
    };
    
    DataAnalysisResult {
        record_count,
        mean,
        std_dev,
        min,
        max,
        patterns,
        anomalies,
        prediction,
    }
}

fn detect_patterns(numbers: &[f64]) -> Vec<String> {
    let mut patterns = Vec::new();
    
    if numbers.len() < 2 {
        return patterns;
    }
    
    // Check for increasing trend
    let increasing_count = numbers.windows(2)
        .filter(|window| window[1] > window[0])
        .count();
    let decreasing_count = numbers.windows(2)
        .filter(|window| window[1] < window[0])
        .count();
    
    if increasing_count > numbers.len() * 3 / 4 {
        patterns.push("Strong increasing trend".to_string());
    } else if decreasing_count > numbers.len() * 3 / 4 {
        patterns.push("Strong decreasing trend".to_string());
    } else if increasing_count > decreasing_count {
        patterns.push("Generally increasing".to_string());
    } else if decreasing_count > increasing_count {
        patterns.push("Generally decreasing".to_string());
    } else {
        patterns.push("No clear trend".to_string());
    }
    
    // Check for periodicity
    if numbers.len() > 4 {
        let autocorr = calculate_autocorrelation(numbers);
        if autocorr > 0.7 {
            patterns.push("Periodic pattern detected".to_string());
        }
    }
    
    // Check for volatility
    let volatility = numbers.windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .sum::<f64>() / (numbers.len() - 1) as f64;
    
    if volatility > numbers.iter().map(|x| x.abs()).sum::<f64>() / numbers.len() as f64 {
        patterns.push("High volatility".to_string());
    } else {
        patterns.push("Low volatility".to_string());
    }
    
    patterns
}

fn calculate_slope(x_values: &[f64], y_values: &[f64]) -> f64 {
    let n = x_values.len() as f64;
    let sum_x: f64 = x_values.iter().sum();
    let sum_y: f64 = y_values.iter().sum();
    let sum_xy: f64 = x_values.iter().zip(y_values.iter())
        .map(|(x, y)| x * y)
        .sum();
    let sum_x2: f64 = x_values.iter().map(|x| x * x).sum();
    
    (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
}

fn calculate_autocorrelation(numbers: &[f64]) -> f64 {
    if numbers.len() < 2 {
        return 0.0;
    }
    
    let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
    let variance = numbers.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / numbers.len() as f64;
    
    if variance == 0.0 {
        return 0.0;
    }
    
    let lag = 1;
    let mut autocorr = 0.0;
    
    for i in 0..numbers.len() - lag {
        autocorr += (numbers[i] - mean) * (numbers[i + lag] - mean);
    }
    
    autocorr / ((numbers.len() - lag) as f64 * variance)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_analysis() {
        let text = "This is a test message. It contains multiple sentences. Hello world!";
        let result = analyze_text(text);
        
        assert!(result.char_count > 0);
        assert!(result.word_count > 0);
        assert!(result.sentence_count > 0);
        assert!(!result.language.is_empty());
        assert!(!result.sentiment.is_empty());
        assert!(!result.keywords.is_empty());
    }
    
    #[test]
    fn test_data_analysis() {
        let data = "1,2,3,4,5,6,7,8,9,10";
        let result = analyze_data(data);
        
        assert_eq!(result.record_count, 10);
        assert!((result.mean - 5.5).abs() < 0.001);
        assert!(result.std_dev > 0.0);
        assert_eq!(result.min, 1.0);
        assert_eq!(result.max, 10.0);
        assert!(!result.patterns.is_empty());
    }
    
    #[test]
    fn test_sentiment_analysis() {
        assert_eq!(analyze_sentiment("I love this! It's amazing!"), "positive");
        assert_eq!(analyze_sentiment("I hate this! It's terrible!"), "negative");
        assert_eq!(analyze_sentiment("This is normal."), "neutral");
    }
} 