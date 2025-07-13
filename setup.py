#!/usr/bin/env python3
"""
Setup script for Telegram Bot with Python + Rust Integration
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

def run_command(command, cwd=None):
    """Run a command and return success status"""
    try:
        result = subprocess.run(
            command,
            shell=True,
            cwd=cwd,
            check=True,
            capture_output=True,
            text=True
        )
        print(f"✅ {command}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ {command}")
        print(f"Error: {e.stderr}")
        return False

def check_requirements():
    """Check if required tools are installed"""
    print("🔍 Checking requirements...")
    
    # Check Python
    if not shutil.which("python"):
        print("❌ Python not found")
        return False
    print("✅ Python found")
    
    # Check pip
    if not shutil.which("pip"):
        print("❌ pip not found")
        return False
    print("✅ pip found")
    
    # Check Rust
    if not shutil.which("cargo"):
        print("❌ Rust not found. Please install Rust from https://rustup.rs/")
        return False
    print("✅ Rust found")
    
    return True

def install_python_dependencies():
    """Install Python dependencies"""
    print("\n📦 Installing Python dependencies...")
    
    requirements_file = "python/requirements.txt"
    if not os.path.exists(requirements_file):
        print(f"❌ Requirements file not found: {requirements_file}")
        return False
    
    return run_command(f"pip install -r {requirements_file}")

def build_rust_library():
    """Build the Rust library"""
    print("\n🔨 Building Rust library...")
    
    rust_dir = "rust"
    if not os.path.exists(rust_dir):
        print(f"❌ Rust directory not found: {rust_dir}")
        return False
    
    # Build in release mode
    return run_command("cargo build --release", cwd=rust_dir)

def create_env_file():
    """Create .env file with example configuration"""
    print("\n📝 Creating environment file...")
    
    env_content = """# Telegram Bot Configuration
BOT_TOKEN=YOUR_BOT_TOKEN_HERE
BOT_USERNAME=your_bot_username

# Rust Library Path
RUST_LIB_PATH=../rust/target/release/libaiogram_rust.so

# Processing Settings
MAX_TEXT_LENGTH=4096
ENCRYPTION_KEY_SIZE=32
ANALYSIS_TIMEOUT=30
"""
    
    env_file = ".env"
    if not os.path.exists(env_file):
        with open(env_file, "w") as f:
            f.write(env_content)
        print("✅ Created .env file")
    else:
        print("ℹ️ .env file already exists")
    
    return True

def main():
    """Main setup function"""
    print("🚀 Setting up Telegram Bot with Python + Rust Integration")
    print("=" * 60)
    
    # Check requirements
    if not check_requirements():
        print("\n❌ Setup failed: Requirements not met")
        sys.exit(1)
    
    # Install Python dependencies
    if not install_python_dependencies():
        print("\n❌ Setup failed: Could not install Python dependencies")
        sys.exit(1)
    
    # Build Rust library
    if not build_rust_library():
        print("\n❌ Setup failed: Could not build Rust library")
        sys.exit(1)
    
    # Create environment file
    if not create_env_file():
        print("\n❌ Setup failed: Could not create environment file")
        sys.exit(1)
    
    print("\n🎉 Setup completed successfully!")
    print("\n📋 Next steps:")
    print("1. Edit .env file and add your bot token")
    print("2. Run the bot: python python/run.py")
    print("3. Test the bot with: /start")
    print("\n💡 Available commands:")
    print("  /analyze <text> - Analyze text with Rust")
    print("  /encrypt <message> - Encrypt message with Rust")
    print("  /realtime - Real-time processing with Rust")
    print("  /analyze_data <data> - Analyze data with Rust")

if __name__ == "__main__":
    main() 