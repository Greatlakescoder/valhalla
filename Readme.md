# Valhalla

A Personal project of mine to get a better understanding of rust and generative AI its a system analysis tool built that combines real-time resource monitoring with AI-powered security insights.

## Features

- Real-time system resource monitoring (CPU, memory, network, disk usage)
- AI-powered security analysis of running processes
- Clean, intuitive metrics visualization

## Prerequisites

- Rust (1.75.0 or later)
- Docker and Docker Compose

## Getting Started

### Local Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/Greatlakescoder/valhalla
   cd valhalla
   ```

2. Install dependencies:
   ```bash
   cargo install --path .
   ```

3. Run the tests:
   ```bash
   cargo test
   ```

4. Start the application:
   ```bash
   cargo run
   ```

### Docker Setup

The application runs in a containerized environment with three main services:

1. **Odin Service** (Backend API)
   - Runs on port 3000
   - Built from custom Rust backend Dockerfile
   - Handles system analysis and AI processing

2. **Frontend Service**
   - Runs on port 5173
   - Built with Vite
   - Communicates with Odin service
   - Configurable API URL through environment variables

3. **Ollama Server**
   - Local AI model server
   - Runs on port 11434
   - GPU-accelerated using NVIDIA support
   - Persistent model storage in `/mnt/valhalla/ollama-server`

#### Requirements
- Docker and Docker Compose (v3.8 or later)
- NVIDIA GPU with appropriate drivers
- NVIDIA Container Toolkit installed

#### Starting the Application

1. Build and start all services:
   ```bash
   docker compose up -d
   ```

2. View service logs:
   ```bash
   # All services
   docker compose logs -f
   
   # Specific service
   docker compose logs -f odin
   docker compose logs -f frontend
   docker compose logs -f ollama-server
   ```

3. Stop and remove containers:
   ```bash
   docker compose down
   ```

#### Network Configuration
- All services communicate over a bridged network named `app-network`
- Frontend can access the backend via `http://localhost:3000`
- Ollama server is restricted to localhost access on port 11434

#### GPU Support
The Ollama server is configured to utilize all available NVIDIA GPUs for model inference. Ensure your system has:
- NVIDIA GPU drivers installed
- NVIDIA Container Toolkit configured
- Appropriate GPU capabilities

## Configuration

### Environment Variables

### Application Configuration

Update the local file in the configuration folder in project root

Example:
```yaml
monitor:
  ollama_url: "http://localhost:11434"
  model: mistral
  context_size: 5000
  offline: true
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test
```

### Code Style

This project follows the official Rust style guidelines. To ensure your code is properly formatted:

```bash
# Check formatting
cargo fmt -- --check

# Apply formatting
cargo fmt

# Run clippy for linting
cargo clippy
```

## Deployment

### Building for Production

```bash
cargo build --release
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- sysinfo crate is key component https://crates.io/crates/sysinfo

## Support

