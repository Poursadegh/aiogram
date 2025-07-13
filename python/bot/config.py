import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Bot Configuration
BOT_TOKEN = os.getenv('BOT_TOKEN', 'YOUR_BOT_TOKEN_HERE')
BOT_USERNAME = os.getenv('BOT_USERNAME', 'your_bot_username')

# Rust Integration Settings
RUST_LIB_PATH = os.getenv('RUST_LIB_PATH', '../rust/target/release/libaiogram_rust.so')

# Processing Settings
MAX_TEXT_LENGTH = 4096
ENCRYPTION_KEY_SIZE = 32
ANALYSIS_TIMEOUT = 30  # seconds

# Conversation Settings
WELCOME_MESSAGE = """
🤖 سلام! من ربات هوشمند شما هستم

🔧 قابلیت‌های من:
• تحلیل متن و داده‌ها
• رمزنگاری پیام‌ها
• پردازش real-time
• تحلیل آماری

💡 دستورات موجود:
/start - شروع
/analyze <متن> - تحلیل متن
/encrypt <پیام> - رمزنگاری
/realtime - پردازش real-time
/help - راهنما
"""

HELP_MESSAGE = """
📚 راهنمای استفاده:

🔍 تحلیل متن:
/analyze این یک متن نمونه است

🔐 رمزنگاری:
/encrypt پیام محرمانه

⚡ پردازش real-time:
/realtime

📊 تحلیل آماری:
/analyze data.csv
""" 