#!/bin/bash
# Serve documentation locally

cd "$(dirname "$0")/book"

echo "=== wayvid Documentation Server ==="
echo
echo "📚 Serving documentation at:"
echo "   http://localhost:3000"
echo
echo "   Language Selector: http://localhost:3000/"
echo "   English:           http://localhost:3000/introduction.html"
echo "   Chinese:           http://localhost:3000/zh-cn/introduction.html"
echo
echo "Press Ctrl+C to stop"
echo

python3 -m http.server 3000
