import asyncio
import json
from aiogram import Router, F
from aiogram.types import Message, CallbackQuery
from aiogram.filters import Command
from aiogram.fsm.context import FSMContext
from aiogram.fsm.state import State, StatesGroup
from .config import WELCOME_MESSAGE, HELP_MESSAGE, MAX_TEXT_LENGTH
from .rust_integration import rust_integration

router = Router()

class AnalysisStates(StatesGroup):
    waiting_for_text = State()

@router.message(Command("start"))
async def cmd_start(message: Message):
    """Handle /start command"""
    await message.answer(WELCOME_MESSAGE)

@router.message(Command("help"))
async def cmd_help(message: Message):
    """Handle /help command"""
    await message.answer(HELP_MESSAGE)

@router.message(Command("analyze"))
async def cmd_analyze(message: Message):
    """Handle /analyze command with text analysis using Rust"""
    text = message.text.replace("/analyze", "").strip()
    
    if not text:
        await message.answer("🔍 لطفاً متنی برای تحلیل وارد کنید:\n/analyze <متن>")
        return
    
    if len(text) > MAX_TEXT_LENGTH:
        await message.answer(f"⚠️ متن خیلی طولانی است. حداکثر {MAX_TEXT_LENGTH} کاراکتر مجاز است.")
        return
    
    # Show processing message
    processing_msg = await message.answer("🔍 در حال تحلیل متن با Rust...")
    
    try:
        # Use Rust for text analysis
        result = rust_integration.analyze_text(text)
        
        if "error" in result:
            await processing_msg.edit_text(f"❌ خطا در تحلیل: {result['error']}")
            return
        
        # Format analysis results
        analysis_text = f"""
📊 **نتایج تحلیل متن:**

📝 **متن ورودی:** {text[:100]}{'...' if len(text) > 100 else ''}

📈 **آمار:**
• تعداد کاراکترها: {result.get('char_count', 'N/A')}
• تعداد کلمات: {result.get('word_count', 'N/A')}
• تعداد جملات: {result.get('sentence_count', 'N/A')}

🔤 **تحلیل زبانی:**
• زبان غالب: {result.get('language', 'N/A')}
• احساس غالب: {result.get('sentiment', 'N/A')}
• کلیدواژه‌ها: {', '.join(result.get('keywords', []))}

⚡ **پردازش شده با:** Rust
⏱️ **زمان پردازش:** {result.get('processing_time', 'N/A')}ms
        """
        
        await processing_msg.edit_text(analysis_text)
        
    except Exception as e:
        await processing_msg.edit_text(f"❌ خطا در تحلیل متن: {str(e)}")

@router.message(Command("encrypt"))
async def cmd_encrypt(message: Message):
    """Handle /encrypt command with Rust cryptography"""
    text = message.text.replace("/encrypt", "").strip()
    
    if not text:
        await message.answer("🔐 لطفاً پیامی برای رمزنگاری وارد کنید:\n/encrypt <پیام>")
        return
    
    if len(text) > MAX_TEXT_LENGTH:
        await message.answer(f"⚠️ پیام خیلی طولانی است. حداکثر {MAX_TEXT_LENGTH} کاراکتر مجاز است.")
        return
    
    # Show processing message
    processing_msg = await message.answer("🔐 در حال رمزنگاری با Rust...")
    
    try:
        # Use Rust for encryption
        encrypted = rust_integration.encrypt_message(text)
        
        if encrypted.startswith("Encryption failed"):
            await processing_msg.edit_text(f"❌ خطا در رمزنگاری: {encrypted}")
            return
        
        result_text = f"""
🔐 **پیام رمزنگاری شده:**

📝 **پیام اصلی:** {text}
🔒 **پیام رمز شده:** `{encrypted}`

⚡ **رمزنگاری شده با:** Rust AES-256
🔑 **کلید:** پیش‌فرض (برای رمزگشایی از همان کلید استفاده کنید)
        """
        
        await processing_msg.edit_text(result_text)
        
    except Exception as e:
        await processing_msg.edit_text(f"❌ خطا در رمزنگاری: {str(e)}")

@router.message(Command("decrypt"))
async def cmd_decrypt(message: Message):
    """Handle /decrypt command with Rust cryptography"""
    text = message.text.replace("/decrypt", "").strip()
    
    if not text:
        await message.answer("🔓 لطفاً پیام رمز شده را وارد کنید:\n/decrypt <پیام رمز شده>")
        return
    
    # Show processing message
    processing_msg = await message.answer("🔓 در حال رمزگشایی با Rust...")
    
    try:
        # Use Rust for decryption
        decrypted = rust_integration.decrypt_message(text)
        
        if decrypted.startswith("Decryption failed"):
            await processing_msg.edit_text(f"❌ خطا در رمزگشایی: {decrypted}")
            return
        
        result_text = f"""
🔓 **پیام رمزگشایی شده:**

🔒 **پیام رمز شده:** {text}
📝 **پیام اصلی:** {decrypted}

⚡ **رمزگشایی شده با:** Rust AES-256
        """
        
        await processing_msg.edit_text(result_text)
        
    except Exception as e:
        await processing_msg.edit_text(f"❌ خطا در رمزگشایی: {str(e)}")

@router.message(Command("realtime"))
async def cmd_realtime(message: Message):
    """Handle /realtime command with Rust real-time processing"""
    # Show processing message
    processing_msg = await message.answer("⚡ شروع پردازش real-time با Rust...")
    
    try:
        # Simulate real-time data
        realtime_data = {
            "timestamp": asyncio.get_event_loop().time(),
            "user_id": message.from_user.id,
            "data_type": "telegram_message",
            "content": "real-time processing test"
        }
        
        # Use Rust for real-time processing
        result = rust_integration.process_realtime(json.dumps(realtime_data))
        
        if "error" in result:
            await processing_msg.edit_text(f"❌ خطا در پردازش real-time: {result['error']}")
            return
        
        result_text = f"""
⚡ **نتایج پردازش Real-Time:**

📊 **داده‌های ورودی:**
• نوع داده: {realtime_data['data_type']}
• شناسه کاربر: {realtime_data['user_id']}
• زمان: {realtime_data['timestamp']:.2f}

🚀 **نتایج پردازش:**
• وضعیت: {result.get('status', 'N/A')}
• سرعت پردازش: {result.get('processing_speed', 'N/A')} ops/sec
• تاخیر: {result.get('latency', 'N/A')}ms
• کیفیت: {result.get('quality', 'N/A')}

⚡ **پردازش شده با:** Rust
        """
        
        await processing_msg.edit_text(result_text)
        
    except Exception as e:
        await processing_msg.edit_text(f"❌ خطا در پردازش real-time: {str(e)}")

@router.message(Command("analyze_data"))
async def cmd_analyze_data(message: Message):
    """Handle /analyze_data command with Rust data analysis"""
    data = message.text.replace("/analyze_data", "").strip()
    
    if not data:
        await message.answer("📊 لطفاً داده‌ای برای تحلیل وارد کنید:\n/analyze_data <داده>")
        return
    
    # Show processing message
    processing_msg = await message.answer("📊 در حال تحلیل داده با Rust...")
    
    try:
        # Use Rust for data analysis
        result = rust_integration.analyze_data(data)
        
        if "error" in result:
            await processing_msg.edit_text(f"❌ خطا در تحلیل داده: {result['error']}")
            return
        
        result_text = f"""
📊 **نتایج تحلیل داده:**

📈 **داده ورودی:** {data[:100]}{'...' if len(data) > 100 else ''}

📊 **آمار توصیفی:**
• تعداد رکوردها: {result.get('record_count', 'N/A')}
• میانگین: {result.get('mean', 'N/A')}
• انحراف معیار: {result.get('std_dev', 'N/A')}
• حداقل: {result.get('min', 'N/A')}
• حداکثر: {result.get('max', 'N/A')}

🔍 **تحلیل پیشرفته:**
• الگوهای شناسایی شده: {result.get('patterns', 'N/A')}
• ناهنجاری‌ها: {result.get('anomalies', 'N/A')}
• پیش‌بینی: {result.get('prediction', 'N/A')}

⚡ **تحلیل شده با:** Rust
⏱️ **زمان تحلیل:** {result.get('analysis_time', 'N/A')}ms
        """
        
        await processing_msg.edit_text(result_text)
        
    except Exception as e:
        await processing_msg.edit_text(f"❌ خطا در تحلیل داده: {str(e)}")

@router.message()
async def handle_text(message: Message):
    """Handle regular text messages with smart responses"""
    text = message.text.lower()
    
    # Simple conversation logic
    if any(word in text for word in ['سلام', 'hello', 'hi']):
        await message.answer("👋 سلام! چطور می‌تونم کمکتون کنم؟")
    
    elif any(word in text for word in ['خوب', 'good', 'عالی']):
        await message.answer("😊 خوشحالم که خوبید! برای استفاده از قابلیت‌های من /help رو بزنید.")
    
    elif any(word in text for word in ['متشکرم', 'thanks', 'thank you']):
        await message.answer("🙏 خواهش می‌کنم! همیشه در خدمت شما هستم.")
    
    elif any(word in text for word in ['خداحافظ', 'bye', 'goodbye']):
        await message.answer("👋 خداحافظ! امیدوارم دوباره ببینمتون! 👋")
    
    else:
        # Default response with suggestions
        await message.answer("""
💡 برای استفاده از قابلیت‌های من:

🔍 تحلیل متن: /analyze <متن>
🔐 رمزنگاری: /encrypt <پیام>
⚡ پردازش real-time: /realtime
📊 تحلیل داده: /analyze_data <داده>
❓ راهنما: /help
        """) 