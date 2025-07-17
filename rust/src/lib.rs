use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

mod crypto;
mod analysis;
mod realtime;
mod config;
mod logging;
mod cache;
mod security;
mod validation;
mod performance;

#[no_mangle]
pub extern "C" fn analyze_text(text: *const c_char) -> *mut c_char {
    let start_time = std::time::Instant::now();
    
    let text_str = unsafe {
        match CStr::from_ptr(text).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let result = analysis::analyze_text(text_str);
    let processing_time = start_time.elapsed().as_millis();
    
    let response = serde_json::json!({
        "char_count": result.char_count,
        "word_count": result.word_count,
        "sentence_count": result.sentence_count,
        "language": result.language,
        "sentiment": result.sentiment,
        "keywords": result.keywords,
        "processing_time": processing_time
    });
    
    let response_str = response.to_string();
    let c_string = match CString::new(response_str) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn encrypt_message(message: *const c_char, key: *const c_char) -> *mut c_char {
    let message_str = unsafe {
        match CStr::from_ptr(message).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let key_str = unsafe {
        match CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let encrypted = match crypto::encrypt(message_str, key_str) {
        Ok(result) => result,
        Err(_) => return ptr::null_mut(),
    };
    
    let c_string = match CString::new(encrypted) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn decrypt_message(encrypted_message: *const c_char, key: *const c_char) -> *mut c_char {
    let encrypted_str = unsafe {
        match CStr::from_ptr(encrypted_message).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let key_str = unsafe {
        match CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let decrypted = match crypto::decrypt(encrypted_str, key_str) {
        Ok(result) => result,
        Err(_) => return ptr::null_mut(),
    };
    
    let c_string = match CString::new(decrypted) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn process_realtime(data: *const c_char) -> *mut c_char {
    let start_time = std::time::Instant::now();
    
    let data_str = unsafe {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let result = realtime::process_realtime_data(data_str);
    let processing_time = start_time.elapsed().as_millis();
    
    let response = serde_json::json!({
        "status": result.status,
        "processing_speed": result.processing_speed,
        "latency": processing_time,
        "quality": result.quality,
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    let response_str = response.to_string();
    let c_string = match CString::new(response_str) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn analyze_data(data: *const c_char) -> *mut c_char {
    let start_time = std::time::Instant::now();
    
    let data_str = unsafe {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let result = analysis::analyze_data(data_str);
    let analysis_time = start_time.elapsed().as_millis();
    
    let response = serde_json::json!({
        "record_count": result.record_count,
        "mean": result.mean,
        "std_dev": result.std_dev,
        "min": result.min,
        "max": result.max,
        "patterns": result.patterns,
        "anomalies": result.anomalies,
        "prediction": result.prediction,
        "analysis_time": analysis_time
    });
    
    let response_str = response.to_string();
    let c_string = match CString::new(response_str) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
} 