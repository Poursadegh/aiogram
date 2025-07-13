#!/usr/bin/env python3
"""
Telegram Bot Runner
Python + Rust Integration for High-Performance Processing
"""

import sys
import os

# Add the bot directory to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bot'))

from bot.main import main
import asyncio

if __name__ == "__main__":
    print("🤖 Starting Telegram Bot with Python + Rust Integration...")
    print("📝 Make sure you have:")
    print("  • Set your bot token in bot/config.py")
    print("  • Built the Rust library: cd rust && cargo build --release")
    print("  • Installed Python dependencies: pip install -r requirements.txt")
    print()
    
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n🛑 Bot stopped by user")
    except Exception as e:
        print(f"❌ Error starting bot: {e}")
        sys.exit(1) 