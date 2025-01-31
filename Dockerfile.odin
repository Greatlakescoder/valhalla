FROM lukemathwalker/cargo-chef:latest-rust-1.83.0 AS chef
WORKDIR /app
RUN apt update && apt install lld clang protobuf-compiler -y
FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json
EXPOSE 11434
EXPOSE 3000
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Build our project
RUN cargo build --release --bin odin

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends stress-ng openssl protobuf-compiler ca-certificates procps \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*


COPY --from=builder /app/target/release/odin /usr/local/bin/odin
COPY configuration configuration
# Create a start script
COPY <<'EOF' /start.sh
#!/bin/bash
odin &
sleep 5
# Run stress-ng with 2 CPU workers at 80% load
stress-ng --cpu 2 --cpu-load 80
EOF
RUN chmod +x /start.sh
CMD ["/start.sh"]