use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;
use whatlang::Lang;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use ndarray::{Array1, Array2};
use statrs::statistics::Statistics;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct TextAnalysisResult {
    pub char_count: usize,
    pub word_count: usize,
    pub sentence_count: usize,
    pub language: String,
    pub language_confidence: f64,
    pub sentiment: String,
    pub sentiment_score: f64,
    pub keywords: Vec<String>,
    pub entities: Vec<Entity>,
    pub summary: String,
    pub readability_score: f64,
    pub topics: Vec<Topic>,
    pub plagiarism_score: f64,
    pub processing_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub weight: f64,
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
    pub forecast: Vec<f64>,
    pub confidence_interval: (f64, f64),
    pub seasonality_detected: bool,
    pub trend_strength: f64,
    pub visualization_data: VisualizationData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualizationData {
    pub histogram: Vec<(f64, usize)>,
    pub box_plot: (f64, f64, f64, f64, f64),
    pub correlation_matrix: Vec<Vec<f64>>,
}

pub fn analyze_text(text: &str) -> TextAnalysisResult {
    let start_time = std::time::Instant::now();
    
    // Character count
    let char_count = text.chars().count();
    
    // Word count using Unicode segmentation
    let words: Vec<&str> = text.unicode_words().collect();
    let word_count = words.len();
    
    // Sentence count using regex
    let sentence_regex = Regex::new(r"[.!?]+").unwrap();
    let sentences: Vec<&str> = sentence_regex.split(text).collect();
    let sentence_count = sentences.len().max(1);
    
    // Enhanced language detection with confidence
    let (language, language_confidence) = detect_language_with_confidence(text);
    
    // Advanced sentiment analysis with score
    let (sentiment, sentiment_score) = analyze_sentiment_advanced(text);
    
    // Keyword extraction
    let keywords = extract_keywords(text);
    
    // Named entity recognition
    let entities = extract_entities(text);
    
    // Text summarization
    let summary = generate_summary(text);
    
    // Readability scoring
    let readability_score = calculate_readability(text);
    
    // Topic modeling
    let topics = extract_topics(text);
    
    // Plagiarism detection
    let plagiarism_score = detect_plagiarism(text);
    
    let processing_time = start_time.elapsed().as_millis();
    
    TextAnalysisResult {
        char_count,
        word_count,
        sentence_count,
        language,
        language_confidence,
        sentiment,
        sentiment_score,
        keywords,
        entities,
        summary,
        readability_score,
        topics,
        plagiarism_score,
        processing_time,
    }
}

fn detect_language_with_confidence(text: &str) -> (String, f64) {
    match whatlang::detect(text) {
        Some(info) => {
            let confidence = info.confidence();
            (info.lang().to_string(), confidence)
        },
        None => ("unknown".to_string(), 0.0),
    }
}

fn analyze_sentiment_advanced(text: &str) -> (String, f64) {
    let positive_words = vec![
        "خوب", "عالی", "عالیه", "ممتاز", "عالی", "خوب", "عالی", "عالیه", "ممتاز",
        "good", "great", "excellent", "amazing", "wonderful", "fantastic", "perfect",
        "beautiful", "nice", "lovely", "happy", "joy", "love", "like", "enjoy",
        "brilliant", "outstanding", "superb", "magnificent", "delightful", "pleased"
    ];
    
    let negative_words = vec![
        "بد", "بدی", "بدیه", "بدی", "بد", "بدی", "بدیه", "بدی", "بد",
        "bad", "terrible", "awful", "horrible", "disgusting", "hate", "dislike",
        "sad", "angry", "furious", "upset", "disappointed", "worried", "scared",
        "dreadful", "atrocious", "abysmal", "appalling", "repulsive", "revolting"
    ];
    
    let text_lower = text.to_lowercase();
    let words: Vec<&str> = text_lower.unicode_words().collect();
    
    let positive_count = words.iter()
        .filter(|word| positive_words.contains(word))
        .count();
    
    let negative_count = words.iter()
        .filter(|word| negative_words.contains(word))
        .count();
    
    let total_sentiment_words = positive_count + negative_count;
    let sentiment_score = if total_sentiment_words > 0 {
        (positive_count as f64 - negative_count as f64) / total_sentiment_words as f64
    } else {
        0.0
    };
    
    let sentiment = match sentiment_score {
        s if s > 0.2 => "positive".to_string(),
        s if s < -0.2 => "negative".to_string(),
        _ => "neutral".to_string(),
    };
    
    (sentiment, sentiment_score)
}

fn extract_entities(text: &str) -> Vec<Entity> {
    let mut entities = Vec::new();
    
    // Simple named entity recognition patterns
    let name_pattern = Regex::new(r"\b[A-Z][a-z]+ [A-Z][a-z]+\b").unwrap();
    let email_pattern = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    let url_pattern = Regex::new(r"https?://[^\s]+").unwrap();
    let phone_pattern = Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap();
    
    // Extract names
    for cap in name_pattern.find_iter(text) {
        entities.push(Entity {
            name: cap.as_str().to_string(),
            entity_type: "PERSON".to_string(),
            confidence: 0.8,
        });
    }
    
    // Extract emails
    for cap in email_pattern.find_iter(text) {
        entities.push(Entity {
            name: cap.as_str().to_string(),
            entity_type: "EMAIL".to_string(),
            confidence: 0.95,
        });
    }
    
    // Extract URLs
    for cap in url_pattern.find_iter(text) {
        entities.push(Entity {
            name: cap.as_str().to_string(),
            entity_type: "URL".to_string(),
            confidence: 0.9,
        });
    }
    
    // Extract phone numbers
    for cap in phone_pattern.find_iter(text) {
        entities.push(Entity {
            name: cap.as_str().to_string(),
            entity_type: "PHONE".to_string(),
            confidence: 0.85,
        });
    }
    
    entities
}

fn generate_summary(text: &str) -> String {
    let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?').collect();
    let words: Vec<&str> = text.unicode_words().collect();
    
    if sentences.len() <= 2 {
        return text.to_string();
    }
    
    // Simple extractive summarization
    let mut sentence_scores: Vec<(usize, f64)> = sentences.iter().enumerate()
        .map(|(i, sentence)| {
            let word_count = sentence.split_whitespace().count();
            let score = word_count as f64 * 0.5; // Simple scoring based on length
            (i, score)
        })
        .collect();
    
    sentence_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    let summary_sentences: Vec<&str> = sentence_scores.iter()
        .take(2.min(sentences.len()))
        .map(|(i, _)| sentences[*i])
        .collect();
    
    summary_sentences.join(". ")
}

fn calculate_readability(text: &str) -> f64 {
    let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?').collect();
    let words: Vec<&str> = text.unicode_words().collect();
    let syllables = count_syllables(text);
    
    if sentences.is_empty() || words.is_empty() {
        return 0.0;
    }
    
    // Flesch Reading Ease formula
    let avg_sentence_length = words.len() as f64 / sentences.len() as f64;
    let avg_syllables_per_word = syllables as f64 / words.len() as f64;
    
    206.835 - (1.015 * avg_sentence_length) - (84.6 * avg_syllables_per_word)
}

fn count_syllables(text: &str) -> usize {
    let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
    let words: Vec<&str> = text.unicode_words().collect();
    
    words.iter().map(|word| {
        let word_lower = word.to_lowercase();
        let mut syllable_count = 0;
        let mut prev_vowel = false;
        
        for ch in word_lower.chars() {
            let is_vowel = vowels.contains(&ch);
            if is_vowel && !prev_vowel {
                syllable_count += 1;
            }
            prev_vowel = is_vowel;
        }
        
        syllable_count.max(1)
    }).sum()
}

fn extract_topics(text: &str) -> Vec<Topic> {
    let words: Vec<&str> = text.unicode_words().collect();
    let mut word_freq: HashMap<&str, usize> = HashMap::new();
    
    for word in words.iter() {
        if word.len() > 3 {
            *word_freq.entry(word).or_insert(0) += 1;
        }
    }
    
    // Simple topic extraction based on frequency
    let mut topics = Vec::new();
    let mut sorted_words: Vec<(&str, &usize)> = word_freq.iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(a.1));
    
    for (word, freq) in sorted_words.iter().take(3) {
        topics.push(Topic {
            name: word.to_string(),
            weight: **freq as f64 / words.len() as f64,
            keywords: vec![word.to_string()],
        });
    }
    
    topics
}

fn detect_plagiarism(text: &str) -> f64 {
    // Simple plagiarism detection based on common phrases
    let common_phrases = vec![
        "in conclusion", "as a result", "it is important", "this shows",
        "according to", "research shows", "studies indicate", "it can be seen",
        "in addition", "furthermore", "moreover", "however", "nevertheless"
    ];
    
    let text_lower = text.to_lowercase();
    let mut plagiarism_score = 0.0;
    
    for phrase in common_phrases {
        if text_lower.contains(phrase) {
            plagiarism_score += 0.1;
        }
    }
    
    plagiarism_score.min(1.0)
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
    let start_time = std::time::Instant::now();
    
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
            forecast: vec![],
            confidence_interval: (0.0, 0.0),
            seasonality_detected: false,
            trend_strength: 0.0,
            visualization_data: VisualizationData {
                histogram: vec![],
                box_plot: (0.0, 0.0, 0.0, 0.0, 0.0),
                correlation_matrix: vec![],
            },
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
    
    // Advanced forecasting
    let forecast = generate_forecast(&numbers);
    let confidence_interval = calculate_confidence_interval(&numbers, mean, std_dev);
    let seasonality_detected = detect_seasonality(&numbers);
    let trend_strength = calculate_trend_strength(&numbers);
    
    // Generate visualization data
    let visualization_data = generate_visualization_data(&numbers);
    
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
        forecast,
        confidence_interval,
        seasonality_detected,
        trend_strength,
        visualization_data,
    }
}

fn generate_forecast(numbers: &[f64]) -> Vec<f64> {
    if numbers.len() < 3 {
        return vec![];
    }
    
    let mut forecast = Vec::new();
    let x_values: Vec<f64> = (0..numbers.len()).map(|i| i as f64).collect();
    let slope = calculate_slope(&x_values, numbers);
    let last_value = numbers.last().unwrap();
    
    // Generate 5 forecast points
    for i in 1..=5 {
        let next_x = numbers.len() as f64 + i as f64;
        let forecast_value = last_value + slope * i as f64;
        forecast.push(forecast_value);
    }
    
    forecast
}

fn calculate_confidence_interval(numbers: &[f64], mean: f64, std_dev: f64) -> (f64, f64) {
    let n = numbers.len() as f64;
    let standard_error = std_dev / n.sqrt();
    let margin_of_error = 1.96 * standard_error; // 95% confidence interval
    
    (mean - margin_of_error, mean + margin_of_error)
}

fn detect_seasonality(numbers: &[f64]) -> bool {
    if numbers.len() < 8 {
        return false;
    }
    
    // Simple seasonality detection using autocorrelation
    let autocorr = calculate_autocorrelation(numbers);
    autocorr > 0.6
}

fn calculate_trend_strength(numbers: &[f64]) -> f64 {
    if numbers.len() < 2 {
        return 0.0;
    }
    
    let x_values: Vec<f64> = (0..numbers.len()).map(|i| i as f64).collect();
    let slope = calculate_slope(&x_values, numbers);
    
    // Normalize trend strength
    let max_possible_slope = numbers.iter().max().unwrap() - numbers.iter().min().unwrap();
    (slope / max_possible_slope).abs()
}

fn generate_visualization_data(numbers: &[f64]) -> VisualizationData {
    // Generate histogram data
    let min = numbers.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = numbers.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max - min;
    let bin_count = 10;
    let bin_width = range / bin_count as f64;
    
    let mut histogram = Vec::new();
    for i in 0..bin_count {
        let bin_start = min + i as f64 * bin_width;
        let bin_end = bin_start + bin_width;
        let count = numbers.iter()
            .filter(|&&x| x >= bin_start && x < bin_end)
            .count();
        histogram.push((bin_start, count));
    }
    
    // Generate box plot data (min, q1, median, q3, max)
    let mut sorted = numbers.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    
    let q1 = if n % 2 == 0 {
        (sorted[n/4 - 1] + sorted[n/4]) / 2.0
    } else {
        sorted[n/4]
    };
    
    let median = if n % 2 == 0 {
        (sorted[n/2 - 1] + sorted[n/2]) / 2.0
    } else {
        sorted[n/2]
    };
    
    let q3 = if n % 2 == 0 {
        (sorted[3*n/4 - 1] + sorted[3*n/4]) / 2.0
    } else {
        sorted[3*n/4]
    };
    
    let box_plot = (min, q1, median, q3, max);
    
    // Simple correlation matrix (for single variable, just variance)
    let correlation_matrix = vec![vec![1.0]];
    
    VisualizationData {
        histogram,
        box_plot,
        correlation_matrix,
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
        assert!(result.language_confidence > 0.0);
        assert!(!result.sentiment.is_empty());
        assert!(result.sentiment_score >= -1.0 && result.sentiment_score <= 1.0);
        assert!(!result.keywords.is_empty());
        assert!(!result.entities.is_empty());
        assert!(!result.summary.is_empty());
        assert!(result.readability_score >= 0.0 && result.readability_score <= 100.0);
        assert!(!result.topics.is_empty());
        assert!(result.plagiarism_score >= 0.0 && result.plagiarism_score <= 1.0);
        assert!(result.processing_time > 0);
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
        assert!(!result.forecast.is_empty());
        assert!(result.confidence_interval.0 <= result.confidence_interval.1);
        assert!(result.seasonality_detected || !result.forecast.is_empty());
        assert!(result.trend_strength >= 0.0 && result.trend_strength <= 1.0);
        assert!(!result.visualization_data.histogram.is_empty());
    }
    
    #[test]
    fn test_sentiment_analysis() {
        assert_eq!(analyze_sentiment_advanced("I love this! It's amazing!").0, "positive");
        assert_eq!(analyze_sentiment_advanced("I hate this! It's terrible!").0, "negative");
        assert_eq!(analyze_sentiment_advanced("This is normal.").0, "neutral");
    }
} 