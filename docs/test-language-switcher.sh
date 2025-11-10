#!/bin/bash
# Test language switcher paths

echo "=== Testing Language Switcher Paths ==="
echo

cd "$(dirname "$0")"

# Start mdbook server in background
echo "Starting mdbook server..."
mdbook serve --hostname 127.0.0.1 --port 3000 > /dev/null 2>&1 &
SERVER_PID=$!

# Wait for server to start
sleep 2

echo "Server started (PID: $SERVER_PID)"
echo

# Test URLs
TEST_URLS=(
    "http://127.0.0.1:3000/introduction.html"
    "http://127.0.0.1:3000/zh_cn/introduction.html"
    "http://127.0.0.1:3000/user-guide/quick-start.html"
    "http://127.0.0.1:3000/zh_cn/user-guide/quick-start.html"
    "http://127.0.0.1:3000/zh_cn/user-guide/installation.html"
)

for url in "${TEST_URLS[@]}"; do
    status=$(curl -s -o /dev/null -w "%{http_code}" "$url")
    if [ "$status" = "200" ]; then
        echo "✅ $url"
    else
        echo "❌ $url (HTTP $status)"
    fi
done

echo
echo "Stopping server..."
kill $SERVER_PID 2>/dev/null

echo "✅ Test complete"
echo
echo "To manually test:"
echo "  cd docs && mdbook serve --open"
echo "  Then click the language switcher in top-right corner"
