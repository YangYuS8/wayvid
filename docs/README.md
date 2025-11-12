# Documentation

This directory contains wayvid documentation built with [mdBook](https://rust-lang.github.io/mdBook/).

## Translation

The documentation is internationalized using [mdbook-i18n-helpers](https://github.com/google/mdbook-i18n-helpers).

### Update translations

When English documentation is updated, follow these steps to sync translations:

```bash
# 1. Extract translatable messages
MDBOOK_OUTPUT='{"xgettext": {}}' mdbook build -d po

# 2. Merge into existing translation
msgmerge --update po/zh-CN.po po/messages.pot

# 3. Auto-translate untranslated messages (requires Azure Translator API key)
export AZURE_TRANSLATOR_KEY='your-api-key-here'
python3 auto_translate.py

# 4. Build documentation
./build.sh
```

### Manual translation

For better quality, manually review and edit translations:

```bash
# Using Poedit (recommended)
poedit po/zh-CN.po

# Or directly edit the .po file
vim po/zh-CN.po
```

### API Key Setup

To use `auto_translate.py`, you need a Microsoft Azure Translator API key:

1. Create an Azure account
2. Create a Translator resource
3. Copy the key from Azure portal
4. Set environment variable:
   ```bash
   export AZURE_TRANSLATOR_KEY='your-key-here'
   ```

**Important:** Never commit API keys to the repository!

## 🌍 Languages

- **English** (primary source)
- **简体中文** (Simplified Chinese)

## 🛠️ Prerequisites

```bash
# Install mdbook
cargo install mdbook

# Install mdbook-i18n-helpers (for translations)
cargo install mdbook-i18n-helpers

# Install gettext tools (for PO file management)
# Arch Linux:
sudo pacman -S gettext
# Debian/Ubuntu:
sudo apt install gettext
# macOS:
brew install gettext
```

## 📖 Building Documentation

### Quick Build

```bash
./build.sh
```

This will:
1. Build English documentation → `book/`
2. Build Chinese documentation → `book/zh-cn/`
3. Create a language selector index page

### Manual Build

```bash
# English (default)
mdbook build

# Chinese
MDBOOK_BOOK__LANGUAGE=zh-CN mdbook build -d book/zh-cn
```

### Local Preview

```bash
./serve.sh
# Opens http://localhost:3000
```

Or manually:

```bash
# English
mdbook serve

# Chinese
MDBOOK_BOOK__LANGUAGE=zh-CN mdbook serve -d book/zh-cn -p 3001
```

## 🌐 Translation Workflow

This project uses the [Gettext](https://www.gnu.org/software/gettext/) system for translations, following the [mdbook-i18n-helpers guide](https://github.com/google/mdbook-i18n-helpers/blob/main/i18n-helpers/USAGE.md).

### 1. Extract Translatable Messages

When you update English source files in `src/`, extract new messages:

```bash
MDBOOK_OUTPUT='{"xgettext": {}}' mdbook build -d po
```

This generates/updates `po/messages.pot` (the PO template).

### 2. Update Translation Files

Merge changes into existing translations:

```bash
msgmerge --update po/zh-CN.po po/messages.pot
```

### 3. Translate

**Option A: Auto-translate common terms** (quick start):

```bash
python3 translate_po.py
```

**Option B: Use a PO editor** (recommended for quality):

- [Poedit](https://poedit.net/) (GUI, cross-platform)
- [Lokalize](https://apps.kde.org/lokalize/) (KDE)
- [Gtranslator](https://wiki.gnome.org/Apps/Gtranslator) (GNOME)
- Online: [Weblate](https://weblate.org/), [Pontoon](https://pontoon.mozilla.org/)

**⚠️ Never edit PO files by hand-rf dev/ features/ reference/ user-guide/* Use proper tools to ensure correct encoding.

### 4. Remove Fuzzy Markers

After reviewing machine-translated entries, remove "fuzzy" flags in your PO editor. Fuzzy entries won't be translated in the output.

### 5. Build and Test

```bash
./build.sh
```

## 📁 Project Structure

```
docs/
├── book.toml              # mdBook configuration
├── build.sh               # Multi-language build script
├── serve.sh               # Local development server
├── translate_po.py        # Auto-translation helper
├── src/                   # English source (primary)
│   ├── SUMMARY.md
│   ├── introduction.md
│   ├── user-guide/
│   ├── features/
│   ├── dev/
│   └── reference/
├── po/                    # Translation files
│   ├── messages.pot       # PO template (auto-generated)
│   └── zh-CN.po           # Chinese translations
└── book/                  # Built documentation (ignored)
    ├── index.html         # Language selector
    ├── *.html             # English docs
    └── zh-cn/             # Chinese docs
        └── *.html
```

## 🔧 Adding a New Language

1. **Extract messages:**
   ```bash
   MDBOOK_OUTPUT='{"xgettext": {}}' mdbook build -d po
   ```

2. **Initialize translation:**
   ```bash
   msginit -i po/messages.pot -l <LANG_CODE> -o po/<LANG_CODE>.po
   ```
   Example: `msginit -i po/messages.pot -l fr -o po/fr.po`

3. **Translate using a PO editor**

4. **Add to build script:**
   ```bash
   # In build.sh, add:
   MDBOOK_BOOK__LANGUAGE=<LANG_CODE> mdbook build -d book/<LANG_CODE>
   ```

5. **Update language selector in `build.sh`**

## 📝 Writing Guidelines

- Use clear, concise language
- Include code examples with syntax highlighting
- Add tips/warnings where appropriate:
  ```markdown
  > **Note:** Important information
  > **Warning:** Caution required
  > **Tip:** Helpful suggestion
  ```
- Follow the [Rust mdBook Guide](https://rust-lang.github.io/mdBook/format/markdown.html) for Markdown syntax

## 🤝 Contributing

1. Edit English source in `src/`
2. Extract messages: `MDBOOK_OUTPUT='{"xgettext": {}}' mdbook build -d po`
3. Update translations: `msgmerge --update po/zh-CN.po po/messages.pot`
4. Translate in PO editor
5. Test: `./build.sh`
6. Submit PR

## 📚 Resources

- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [mdbook-i18n-helpers Guide](https://github.com/google/mdbook-i18n-helpers/blob/main/i18n-helpers/USAGE.md)
- [Gettext Manual](https://www.gnu.org/software/gettext/manual/)
- [ISO 639-1 Language Codes](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes)

## 📜 License

Same as wayvid project (MIT License).
