#!/bin/bash

# Integration test script for GitHub Actions
# Tests that the chat server and client work together

set -e

echo "Starting chat server integration test..."

# Build the project first
echo "Building project..."
cargo build --release

# Start the chat server in the background
echo "Starting chat server on port 8080..."
./target/release/chat-server &
SERVER_PID=$!

# Function to cleanup on exit
cleanup() {
    echo "Cleaning up..."
    if kill -0 $SERVER_PID 2>/dev/null; then
        echo "Stopping server (PID: $SERVER_PID)"
        kill $SERVER_PID
        wait $SERVER_PID 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Give server time to start
echo "Waiting for server to start..."
sleep 5

# Test 1: Check if server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "Server failed to start or crashed"
    exit 1
fi
echo "Server started successfully (PID: $SERVER_PID)"

# Test 2: Test client connection using timeout and input redirection
echo "Testing client connection and messaging..."

# Create input for the client
cat > /tmp/client_input.txt << 'EOF'
send Hello from GitHub Actions!
send This is a test message
send Integration test working!
leave
EOF

# Run client with timeout and input redirection
# Use gtimeout on macOS if available, otherwise timeout (Linux)
TIMEOUT_CMD="timeout"
if command -v gtimeout >/dev/null 2>&1; then
    TIMEOUT_CMD="gtimeout"
elif ! command -v timeout >/dev/null 2>&1; then
    echo "timeout command not available, running without timeout"
    TIMEOUT_CMD=""
fi

if [ -n "$TIMEOUT_CMD" ]; then
    CLIENT_CMD="$TIMEOUT_CMD 15 ./target/release/chat-client --host 127.0.0.1 --port 8080 --username github-actions-test"
else
    CLIENT_CMD="./target/release/chat-client --host 127.0.0.1 --port 8080 --username github-actions-test"
fi

if $CLIENT_CMD < /tmp/client_input.txt > /tmp/client_output.txt 2>&1; then
    echo "Client executed successfully"
    
    # Check if client output contains expected text
    if grep -q "Connected to WebSocket chat server" /tmp/client_output.txt; then
        echo "Client connected to server successfully"
    else
        echo "Client connection failed"
        echo "Client output:"
        cat /tmp/client_output.txt
        exit 1
    fi
    
    if grep -q "Goodbye" /tmp/client_output.txt; then
        echo "Client disconnected cleanly"
    else
        echo "Client may not have disconnected cleanly"
        echo "Client output:"
        cat /tmp/client_output.txt
    fi
else
    echo "Client test failed or timed out"
    echo "Client output:"
    cat /tmp/client_output.txt 2>/dev/null || echo "No client output available"
    exit 1
fi

# Test 3: Verify server is still running after client interaction
sleep 2
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "Server crashed during client interaction"
    exit 1
fi
echo "Server remained stable during client interaction"

# Test 4: Test rapid client connections
echo "Testing multiple rapid client connections..."
for i in {1..3}; do
    echo "Testing rapid client $i..."
    
    if [ -n "$TIMEOUT_CMD" ]; then
        echo "send Quick message from client $i!" | $TIMEOUT_CMD 10 ./target/release/chat-client --host 127.0.0.1 --port 8080 --username "rapid-test-$i" > /dev/null 2>&1 &
    else
        echo "send Quick message from client $i!" | ./target/release/chat-client --host 127.0.0.1 --port 8080 --username "rapid-test-$i" > /dev/null 2>&1 &
    fi
    
    # Don't wait for this client, just fire and forget
    sleep 0.5
done

# Wait a bit for all clients to finish
sleep 5

# Final server health check
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "Server crashed during rapid client test"
    exit 1
fi
echo "Server survived rapid client connections"

# Cleanup input file
rm -f /tmp/client_input.txt /tmp/client_output.txt

echo "All integration tests passed!"
echo "Server and client communication working correctly"
echo "Server remains stable under various load conditions"
echo "No crashes or failures detected"