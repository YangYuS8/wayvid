#!/bin/bash
# Build multi-language documentation with mdbook-i18n-helpers

set -e

cd "$(dirname "$0")"

echo "=== Building wayvid Multi-Language Documentation ==="
echo

# Clean previous build
rm -rf book/
mkdir -p book/

# Build English (default language)
echo "📘 Building English documentation..."
mdbook build
echo "✅ English documentation built -> book/"
echo

# Build Chinese
echo "📘 Building Chinese documentation..."
MDBOOK_BOOK__LANGUAGE=zh-CN mdbook build -d book/zh-cn
echo "✅ Chinese documentation built -> book/zh-cn/"
echo

# Create language selector index
cat > book/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>wayvid Documentation</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: #1a1a1a;
            color: #e0e0e0;
        }
        .container {
            text-align: center;
        }
        h1 {
            margin-bottom: 2rem;
        }
        p {
            color: #999;
            margin-bottom: 2rem;
        }
        .language-links {
            display: flex;
            gap: 2rem;
            justify-content: center;
        }
        a {
            display: inline-block;
            padding: 1rem 2rem;
            background: #2a2a2a;
            color: #61afef;
            text-decoration: none;
            border-radius: 8px;
            border: 2px solid #61afef;
            transition: all 0.3s;
        }
        a:hover {
            background: #61afef;
            color: #1a1a1a;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>wayvid Documentation</h1>
        <p>Choose your language / 选择语言</p>
        <div class="language-links">
            <a href="./introduction.html">English</a>
            <a href="./zh-cn/introduction.html">简体中文</a>
        </div>
    </div>
</body>
</html>
EOF

echo "📄 Created language selector index page"
echo
echo "✅ Build complete!"
echo "   📂 Root:    book/index.html (language selector)"
echo "   📂 English: book/introduction.html"
echo "   📂 Chinese: book/zh-cn/introduction.html"
echo
echo "To serve locally:"
echo "  ./serve.sh"
