#!/usr/bin/env python3
"""
Automatic translation helper using Microsoft Translator API.
Translates untranslated messages in PO file from English to Chinese.

Usage:
    export AZURE_TRANSLATOR_KEY='your-api-key-here'
    python3 auto_translate.py
"""

import os
import re
import urllib.request
import urllib.parse
import uuid
import json
import time
from pathlib import Path
from typing import List, Tuple

# Microsoft Translator API configuration
API_KEY = os.environ.get('AZURE_TRANSLATOR_KEY')
ENDPOINT = "https://api.cognitive.microsofttranslator.com"
# Location 可能需要根据你的 Azure 资源调整，常见值: eastasia, southeastasia, westus2, global
# 如果是全局资源，可以设置为 None
LOCATION = None  # 先尝试不设置 location

def translate_text(texts: List[str], from_lang: str = 'en', to_lang: str = 'zh-Hans') -> List[str]:
    """
    Translate a batch of texts using Microsoft Translator API.
    
    Args:
        texts: List of texts to translate
        from_lang: Source language (default: en)
        to_lang: Target language (default: zh-Hans for Simplified Chinese)
    
    Returns:
        List of translated texts
    
    Raises:
        ValueError: If AZURE_TRANSLATOR_KEY environment variable is not set
    """
    if not API_KEY:
        raise ValueError(
            "Azure Translator API 密钥未设置。\n"
            "请设置环境变量: export AZURE_TRANSLATOR_KEY='your-api-key-here'"
        )
    
    if not texts:
        return []
    
    path = '/translate'
    constructed_url = ENDPOINT + path
    
    params = {
        'api-version': '3.0',
        'from': from_lang,
        'to': [to_lang]
    }
    
    headers = {
        'Ocp-Apim-Subscription-Key': API_KEY,
        'Content-type': 'application/json',
        'X-ClientTraceId': str(uuid.uuid4())
    }
    
    # Only add location header if specified
    if LOCATION:
        headers['Ocp-Apim-Subscription-Region'] = LOCATION
    
    body = [{'text': text} for text in texts]
    
    try:
        # Construct URL with query parameters
        url_with_params = constructed_url + '?' + urllib.parse.urlencode(params, doseq=True)
        
        # Prepare request
        req = urllib.request.Request(
            url_with_params,
            data=json.dumps(body).encode('utf-8'),
            headers=headers,
            method='POST'
        )
        
        # Make request
        with urllib.request.urlopen(req) as response:
            result = json.loads(response.read().decode('utf-8'))
        
        translations = []
        for item in result:
            if 'translations' in item and len(item['translations']) > 0:
                translations.append(item['translations'][0]['text'])
            else:
                translations.append('')  # Empty if translation failed
        
        return translations
        
    except Exception as e:
        print(f"❌ API request failed: {e}")
        return [''] * len(texts)

def extract_untranslated_messages(po_content: str) -> List[Tuple[str, int]]:
    """
    Extract untranslated messages from PO file content.
    
    Returns:
        List of tuples (msgid, position_in_content)
    """
    # Pattern to match msgid "..." followed by msgstr ""
    pattern = r'msgid "([^"]+)"\nmsgstr ""'
    
    matches = []
    for match in re.finditer(pattern, po_content):
        msgid = match.group(1)
        position = match.start()
        
        # Skip empty msgids and metadata
        if msgid and not msgid.startswith('Project-Id-Version'):
            # Unescape common escape sequences
            msgid = msgid.replace('\\n', '\n').replace('\\t', '\t').replace('\\"', '"')
            matches.append((msgid, position))
    
    return matches

def translate_po_file(po_path: Path, batch_size: int = 50, delay: float = 1.0):
    """
    Translate untranslated messages in a PO file.
    
    Args:
        po_path: Path to the PO file
        batch_size: Number of messages to translate in one API call (max 100)
        delay: Delay between API calls in seconds
    """
    print(f"📖 Reading {po_path.name}...")
    
    with open(po_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    print("🔍 Extracting untranslated messages...")
    untranslated = extract_untranslated_messages(content)
    
    if not untranslated:
        print("✅ All messages are already translated!")
        return
    
    total = len(untranslated)
    print(f"📝 Found {total} untranslated messages")
    print(f"🌐 Translating in batches of {batch_size}...\n")
    
    translated_count = 0
    
    # Process in batches
    for i in range(0, total, batch_size):
        batch = untranslated[i:i+batch_size]
        batch_texts = [msg[0] for msg in batch]
        
        print(f"🔄 Translating batch {i//batch_size + 1}/{(total + batch_size - 1)//batch_size} ({len(batch)} messages)...", end=' ')
        
        # Translate batch
        translations = translate_text(batch_texts)
        
        # Replace in content
        for (original, position), translation in zip(batch, translations):
            if translation:
                # Escape quotes and newlines for PO format
                escaped_original = original.replace('\n', '\\n').replace('\t', '\\t').replace('"', '\\"')
                escaped_translation = translation.replace('"', '\\"').replace('\n', '\\n').replace('\t', '\\t')
                
                # Find and replace
                old_pattern = f'msgid "{escaped_original}"\nmsgstr ""'
                new_pattern = f'msgid "{escaped_original}"\nmsgstr "{escaped_translation}"'
                
                if old_pattern in content:
                    content = content.replace(old_pattern, new_pattern, 1)
                    translated_count += 1
        
        print(f"✓ ({translated_count}/{total})")
        
        # Delay to avoid rate limiting
        if i + batch_size < total:
            time.sleep(delay)
    
    print(f"\n💾 Writing translations to {po_path.name}...")
    with open(po_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"\n✅ Successfully translated {translated_count} messages!")
    print(f"📊 Translation progress: {translated_count}/{total} ({translated_count*100//total}%)")

def main():
    po_file = Path(__file__).parent / "po" / "zh-CN.po"
    
    if not po_file.exists():
        print(f"❌ Error: {po_file} not found")
        return 1
    
    print("=" * 60)
    print("🌐 Microsoft Translator - PO File Auto-Translation")
    print("=" * 60)
    print()
    
    try:
        translate_po_file(po_file, batch_size=25, delay=1.0)
        
        print("\n" + "=" * 60)
        print("🎉 Translation complete!")
        print("=" * 60)
        print("\nNext steps:")
        print("1. Review translations: poedit po/zh-CN.po")
        print("2. Build documentation: ./build.sh")
        print("3. Preview: ./serve.sh")
        
        return 0
        
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    exit(main())
