# wayvid Documentation

Professional documentation built with [mdBook](https://rust-lang.github.io/mdBook/).

## Features

- 🌐 **Multi-language Support**: English (en) and Simplified Chinese (zh-CN)
- 🔄 **Live Language Switcher**: Toggle between languages in top-right corner
- 📱 **Responsive Design**: Works on desktop and mobile
- 🔍 **Full-text Search**: Fast client-side search
- 🎨 **Dark/Light Themes**: Multiple color schemes

## Building

### Prerequisites

```bash
cargo install mdbook
```

### Build HTML

```bash
cd docs
mdbook build
```

Output: `docs/book/`

### Local Preview

```bash
cd docs
mdbook serve --open
```

Visit: http://localhost:3000

## Structure

```
docs/
├── book.toml              # mdBook configuration
├── src/                   # English documentation
│   ├── SUMMARY.md         # Table of contents
│   ├── introduction.md
│   ├── user-guide/        # User documentation
│   ├── features/          # Feature documentation
│   ├── dev/               # Developer documentation
│   ├── reference/         # Reference documentation
│   └── zh_cn/             # Chinese translations
│       ├── introduction.md
│       └── user-guide/
├── theme/                 # Custom theme
│   ├── custom.css         # Language switcher styles
│   └── language-switcher.js  # Language switcher logic
└── book/                  # Generated HTML (gitignored)
```

## Adding New Pages

### English

1. Create markdown file in `src/`
2. Add entry to `src/SUMMARY.md`

### Chinese Translation

1. Create corresponding file in `src/zh_cn/`
2. Add entry to `src/SUMMARY_ZH_CN.md` (for reference)
3. Update `theme/language-switcher.js` page mapping if needed

## Language Switcher

The language switcher appears in the top-right corner and:

- Automatically detects current language from URL path
- Maps corresponding pages between languages
- Falls back to introduction page if translation unavailable
- Persists across page navigation

### Adding New Translations

Edit `theme/language-switcher.js`:

```javascript
const languages = {
    'en': { name: 'English', path: '' },
    'zh-CN': { name: '简体中文', path: '/zh_cn' },
    // Add new language:
    // 'ja': { name: '日本語', path: '/ja' }
};
```

## Deployment

### GitHub Pages

```bash
# Build documentation
cd docs && mdbook build

# Deploy book/ directory to gh-pages branch
# (GitHub Actions can automate this)
```

### Custom Server

Serve `docs/book/` directory as static files.

## Maintenance

### Update Dependencies

```bash
cargo install mdbook --force
```

### Check for Broken Links

```bash
cd docs
mdbook test
```

## License

MIT - See LICENSE-MIT in repository root.
