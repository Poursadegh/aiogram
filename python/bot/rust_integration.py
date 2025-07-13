import ctypes
import os
import json
from typing import Optional, Dict, Any
from .config import RUST_LIB_PATH

class RustIntegration:
    """Integration with Rust functions for heavy processing tasks"""
    
    def __init__(self):
        self.lib = None
        self._load_rust_library()
    
    def _load_rust_library(self):
        """Load the Rust library using ctypes"""
        try:
            if os.path.exists(RUST_LIB_PATH):
                self.lib = ctypes.CDLL(RUST_LIB_PATH)
                self._setup_function_signatures()
                print("✅ Rust library loaded successfully")
            else:
                print(f"⚠️ Rust library not found at {RUST_LIB_PATH}")
                print("📝 Please build the Rust library first: cd rust && cargo build --release")
        except Exception as e:
            print(f"❌ Error loading Rust library: {e}")
    
    def _setup_function_signatures(self):
        """Setup function signatures for Rust functions"""
        if not self.lib:
            return
            
        # Text analysis function
        self.lib.analyze_text.argtypes = [ctypes.c_char_p]
        self.lib.analyze_text.restype = ctypes.c_char_p
        
        # Encryption function
        self.lib.encrypt_message.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        self.lib.encrypt_message.restype = ctypes.c_char_p
        
        # Decryption function
        self.lib.decrypt_message.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        self.lib.decrypt_message.restype = ctypes.c_char_p
        
        # Real-time processing function
        self.lib.process_realtime.argtypes = [ctypes.c_char_p]
        self.lib.process_realtime.restype = ctypes.c_char_p
        
        # Data analysis function
        self.lib.analyze_data.argtypes = [ctypes.c_char_p]
        self.lib.analyze_data.restype = ctypes.c_char_p
    
    def analyze_text(self, text: str) -> Dict[str, Any]:
        """Analyze text using Rust processing"""
        if not self.lib:
            return {"error": "Rust library not loaded"}
        
        try:
            text_bytes = text.encode('utf-8')
            result = self.lib.analyze_text(text_bytes)
            result_str = ctypes.string_at(result).decode('utf-8')
            return json.loads(result_str)
        except Exception as e:
            return {"error": f"Analysis failed: {str(e)}"}
    
    def encrypt_message(self, message: str, key: str = "default_key") -> str:
        """Encrypt message using Rust cryptography"""
        if not self.lib:
            return "Rust library not loaded"
        
        try:
            message_bytes = message.encode('utf-8')
            key_bytes = key.encode('utf-8')
            result = self.lib.encrypt_message(message_bytes, key_bytes)
            return ctypes.string_at(result).decode('utf-8')
        except Exception as e:
            return f"Encryption failed: {str(e)}"
    
    def decrypt_message(self, encrypted_message: str, key: str = "default_key") -> str:
        """Decrypt message using Rust cryptography"""
        if not self.lib:
            return "Rust library not loaded"
        
        try:
            message_bytes = encrypted_message.encode('utf-8')
            key_bytes = key.encode('utf-8')
            result = self.lib.decrypt_message(message_bytes, key_bytes)
            return ctypes.string_at(result).decode('utf-8')
        except Exception as e:
            return f"Decryption failed: {str(e)}"
    
    def process_realtime(self, data: str) -> Dict[str, Any]:
        """Process real-time data using Rust"""
        if not self.lib:
            return {"error": "Rust library not loaded"}
        
        try:
            data_bytes = data.encode('utf-8')
            result = self.lib.process_realtime(data_bytes)
            result_str = ctypes.string_at(result).decode('utf-8')
            return json.loads(result_str)
        except Exception as e:
            return {"error": f"Real-time processing failed: {str(e)}"}
    
    def analyze_data(self, data: str) -> Dict[str, Any]:
        """Analyze data using Rust processing"""
        if not self.lib:
            return {"error": "Rust library not loaded"}
        
        try:
            data_bytes = data.encode('utf-8')
            result = self.lib.analyze_data(data_bytes)
            result_str = ctypes.string_at(result).decode('utf-8')
            return json.loads(result_str)
        except Exception as e:
            return {"error": f"Data analysis failed: {str(e)}"}

# Global instance
rust_integration = RustIntegration() 