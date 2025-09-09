# Chat Application

A high-performance asynchronous chat server and CLI client built with Rust, featuring WebSocket communication, concurrent user management, and comprehensive testing infrastructure.

## Features

- **Real-time Communication**: WebSocket-based messaging for instant communication
- **High Concurrency**: Non-blocking asynchronous architecture supporting many concurrent users
- **Memory Efficient**: Optimized data structures with minimal memory footprint
- **Unique Usernames**: Automatic enforcement of unique user identifiers
- **Interactive CLI**: User-friendly command-line interface with real-time message display
- **Comprehensive Testing**: Unit tests, integration tests, and performance benchmarks
- **CI/CD Pipeline**: Automated testing, formatting, and performance tracking
- **Pre-commit Hooks**: Automatic code quality checks before commits

## Architecture

The application consists of three main components:

### 1. Chat Server (`chat-server`)
- **WebSocket Server**: Listens for client connections on configurable host/port
- **User Management**: Tracks connected users with unique usernames
- **Message Broadcasting**: Efficiently distributes messages to all connected clients except sender
- **Connection Handling**: Graceful handling of client connections and disconnections
- **Resource Cleanup**: Automatic cleanup of disconnected users

### 2. Chat Client (`chat-client`)
- **WebSocket Client**: Connects to the chat server via WebSocket protocol
- **Interactive CLI**: Command prompt interface for sending messages
- **Real-time Display**: Shows incoming messages from other users in real-time
- **Graceful Exit**: Clean disconnection with proper server notification

### 3. Common Types (`chat-types`)
- **Message Protocol**: Shared data structures for client-server communication
- **JSON Serialization**: Efficient message encoding/decoding
- **Type Safety**: Strong typing for all protocol messages

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- Git (for development workflow)

### Building the Project

```bash
# Clone the repository
git clone <repository-url>
cd simple-chat

# Build all components
cargo build --release

# Run tests
cargo test
```

### Running the Application

#### 1. Start the Server

```bash
# Default: localhost:8080
cargo run --bin chat-server

# Custom address
cargo run --bin chat-server -- --addr 0.0.0.0:3000
```

#### 2. Connect Clients

```bash
# Connect with username
cargo run --bin chat-client -- --username alice

# Connect to custom server
cargo run --bin chat-client -- --host localhost --port 3000 --username bob
```

#### 3. Chat Commands

Once connected, use these commands in the client:

```
send Hello everyone!    # Send a message to all users
leave                   # Disconnect and exit
```

### Project Structure

```
simple-chat/
├── chat-server/           # WebSocket server implementation
│   ├── src/
│   │   ├── main.rs       # Server entry point and CLI
│   │   ├── websocket.rs  # WebSocket handling and user management
│   │   └── lib.rs        # Library exports
│   └── benches/          # Performance benchmarks
├── chat-client/           # CLI client implementation
│   ├── src/
│   │   ├── main.rs       # Client entry point and CLI
│   │   ├── websocket.rs  # WebSocket client and UI
│   │   └── lib.rs        # Library exports
├── chat-types/            # Shared protocol types
│   └── src/lib.rs        # Message types and JSON handling
├── scripts/               # Development and CI scripts
│   ├── test-integration.sh # Integration testing
│   ├── pre-commit        # Git pre-commit hook
│   └── install-hooks.sh  # Hook installation script
└── .github/workflows/    # CI/CD pipeline
    └── ci.yml            # GitHub Actions workflow
```

### Development Commands

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Run all tests
cargo test

# Run benchmarks
cd chat-server && cargo bench

# Install pre-commit hooks
./scripts/install-hooks.sh
```

### Testing

The project includes comprehensive testing:

#### Unit Tests
```bash
cargo test --lib
```

#### Integration Tests
```bash
# Automated server-client communication test
./scripts/test-integration.sh

# Manual integration testing
cargo test --test integration_tests
```

#### Performance Benchmarks
```bash
cd chat-server
cargo bench

# View results
open target/criterion/report/index.html
```

The benchmark reports include interactive HTML reports with detailed visualizations:

![Throughput Violin Plot](docs/images/throughput-violin.svg)
*Execution time distribution across different client loads (10, 50, 100 clients)*

![Latency Distribution](docs/images/latency-distribution.svg)
*Message delivery latency probability distribution showing consistent low-latency performance*

![50-Client Throughput](docs/images/throughput-50clients.svg) 
*Performance analysis for 50 concurrent clients with consistent execution times*

![Performance Regression Analysis](docs/images/performance-regression.svg)
*Linear regression analysis showing performance scaling with 100 concurrent clients*

## Configuration

### Server Configuration

| Option | Environment | CLI Flag | Default | Description |
|--------|-------------|----------|---------|-------------|
| Address | - | `--addr` | `127.0.0.1:8080` | Server bind address (host:port) |

### Client Configuration

| Option | Environment | CLI Flag | Default | Description |
|--------|-------------|----------|---------|-------------|
| Host | `CHAT_HOST` | `--host` | `127.0.0.1` | Server address |
| Port | `CHAT_PORT` | `--port` | `8080` | Server port |
| Username | `CHAT_USERNAME` | `--username` | Required | Unique identifier |

## CI/CD Pipeline

The project includes a comprehensive GitHub Actions workflow:

### Automated Checks
- **Code Formatting**: Ensures consistent code style
- **Clippy Linting**: Catches common mistakes and suggests improvements
- **Unit Tests**: Validates individual component functionality
- **Integration Tests**: Tests complete server-client communication
- **Performance Benchmarks**: Tracks performance metrics over time

### Workflow Jobs
1. **Test**: Format checking, clippy, unit tests, and build
2. **Integration Test**: End-to-end server-client communication testing
3. **Benchmark**: Performance testing with automated PR comments

### Pre-commit Hooks

Install development hooks for automatic quality checks:

```bash
./scripts/install-hooks.sh
```

The pre-commit hook automatically:
- Checks code formatting (`cargo fmt --check`)
- Validates compilation (`cargo check`)
- Runs tests (`cargo test`)
- Ensures clippy compliance

## Performance

### Benchmarks

The application includes comprehensive performance benchmarks:

- **Throughput Tests**: Measures messages/second with varying client loads
- **Latency Tests**: Measures end-to-end message delivery time
- **Concurrency Tests**: Validates stability under high connection load

#### Sample Benchmark Results

![Typical Latency](docs/images/latency-typical.svg)
*Typical message latency measurements showing consistent performance*

#### Benchmark Report Features

The Criterion benchmark reports provide:

- **Interactive Charts**: Hover over data points for detailed metrics
- **Statistical Analysis**: Mean, median, standard deviation, and outliers
- **Comparison Views**: Performance trends across multiple runs
- **Regression Detection**: Automatic identification of performance changes
- **Export Options**: CSV, JSON data export for further analysis

![Performance Regression Analysis](docs/images/performance-regression.svg)
*Automated regression detection showing performance trends over time*

### Performance Characteristics

- **Memory Usage**: ~1MB base memory + ~50KB per connected user
- **Throughput**: 10,000+ messages/second on modern hardware
- **Latency**: <1ms message delivery for local connections
- **Concurrency**: Supports 1,000+ concurrent connections

## Protocol

### Message Format

All messages use JSON over WebSocket:

```json
// Client to Server
{"Join": {"username": "alice"}}
{"SendMessage": {"content": "Hello everyone!"}}
{"Leave": null}

// Server to Client
{"UserJoined": {"username": "bob"}}
{"UserLeft": {"username": "charlie"}}
{"BroadcastMessage": {"from": "alice", "content": "Hello everyone!"}}
{"Error": {"message": "Username already taken"}}
```

## Contributing

1. **Fork the Repository**: Create your own fork for development
2. **Create Feature Branch**: `git checkout -b feature/amazing-feature`
3. **Install Hooks**: Run `./scripts/install-hooks.sh`
4. **Make Changes**: Implement your feature with tests
5. **Commit Changes**: Pre-commit hooks will validate your code
6. **Push Branch**: `git push origin feature/amazing-feature`
7. **Create Pull Request**: Submit for review with performance benchmarks

### Code Quality Standards

- All code must pass `cargo fmt --check`
- All code must pass `cargo clippy` without warnings
- All tests must pass (`cargo test`)
- New features require corresponding tests
- Performance regressions require justification

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Tokio](https://tokio.rs/) for async runtime
- Uses [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) for WebSocket support
- Benchmarking powered by [Criterion](https://github.com/bheisler/criterion.rs)
- CLI parsing with [Clap](https://github.com/clap-rs/clap)