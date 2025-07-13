#!/usr/bin/env python3
"""
Example usage of Telegram Bot with Python + Rust Integration

This file demonstrates how to use the bot's features programmatically.
"""

import asyncio
import json
import sys
import os

# Add the bot directory to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python', 'bot'))

from bot.rust_integration import rust_integration

def test_rust_integration():
    """Test Rust integration functions"""
    print("🧪 Testing Rust Integration")
    print("=" * 40)
    
    # Test text analysis
    print("\n📊 Testing Text Analysis:")
    sample_text = "This is a sample text for analysis. It contains multiple sentences and should be processed by Rust for high performance."
    result = rust_integration.analyze_text(sample_text)
    print(f"Input: {sample_text}")
    print(f"Result: {json.dumps(result, indent=2)}")
    
    # Test encryption
    print("\n🔐 Testing Encryption:")
    message = "This is a secret message that needs to be encrypted!"
    encrypted = rust_integration.encrypt_message(message)
    print(f"Original: {message}")
    print(f"Encrypted: {encrypted}")
    
    # Test decryption
    print("\n🔓 Testing Decryption:")
    decrypted = rust_integration.decrypt_message(encrypted)
    print(f"Decrypted: {decrypted}")
    print(f"Match: {message == decrypted}")
    
    # Test real-time processing
    print("\n⚡ Testing Real-time Processing:")
    realtime_data = {
        "timestamp": 1234567890.0,
        "user_id": 12345,
        "data_type": "telegram_message",
        "content": "Hello world! This is a test message for real-time processing."
    }
    result = rust_integration.process_realtime(json.dumps(realtime_data))
    print(f"Input: {json.dumps(realtime_data, indent=2)}")
    print(f"Result: {json.dumps(result, indent=2)}")
    
    # Test data analysis
    print("\n📈 Testing Data Analysis:")
    sample_data = "1,2,3,4,5,6,7,8,9,10,15,20,25,30,35,40,45,50"
    result = rust_integration.analyze_data(sample_data)
    print(f"Input: {sample_data}")
    print(f"Result: {json.dumps(result, indent=2)}")

def test_performance_comparison():
    """Compare Python vs Rust performance"""
    print("\n⚡ Performance Comparison")
    print("=" * 40)
    
    import time
    
    # Test text analysis performance
    sample_text = "This is a performance test. " * 1000
    
    # Python implementation (simplified)
    def python_analyze_text(text):
        return {
            "char_count": len(text),
            "word_count": len(text.split()),
            "sentence_count": len(text.split('.')),
            "language": "en",
            "sentiment": "neutral",
            "keywords": ["test", "performance"],
            "processing_time": 0
        }
    
    # Time Python implementation
    start_time = time.time()
    for _ in range(100):
        python_analyze_text(sample_text)
    python_time = time.time() - start_time
    
    # Time Rust implementation
    start_time = time.time()
    for _ in range(100):
        rust_integration.analyze_text(sample_text)
    rust_time = time.time() - start_time
    
    print(f"Python time: {python_time:.4f}s")
    print(f"Rust time: {rust_time:.4f}s")
    print(f"Speedup: {python_time / rust_time:.2f}x")

def test_error_handling():
    """Test error handling in Rust integration"""
    print("\n🚨 Error Handling Test")
    print("=" * 40)
    
    # Test with invalid data
    print("\nTesting with invalid JSON:")
    result = rust_integration.process_realtime("invalid json")
    print(f"Result: {result}")
    
    # Test with empty data
    print("\nTesting with empty data:")
    result = rust_integration.analyze_data("")
    print(f"Result: {result}")
    
    # Test with very long text
    print("\nTesting with very long text:")
    long_text = "test " * 10000
    result = rust_integration.analyze_text(long_text)
    print(f"Result: {result}")

def main():
    """Main test function"""
    print("🤖 Telegram Bot with Python + Rust Integration")
    print("Example Usage and Testing")
    print("=" * 60)
    
    try:
        # Test Rust integration
        test_rust_integration()
        
        # Test performance comparison
        test_performance_comparison()
        
        # Test error handling
        test_error_handling()
        
        print("\n🎉 All tests completed successfully!")
        print("\n💡 To run the actual bot:")
        print("1. Set your bot token in .env file")
        print("2. Run: python python/run.py")
        print("3. Test with: /start")
        
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        print("Make sure you have:")
        print("1. Built the Rust library: cd rust && cargo build --release")
        print("2. Installed Python dependencies: pip install -r python/requirements.txt")
        sys.exit(1)

if __name__ == "__main__":
    main() 