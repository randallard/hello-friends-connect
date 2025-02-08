# Update the builder stage in Dockerfile
FROM rust:1.75-slim-bullseye as builder

# Install dependencies more efficiently
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    nodejs \
    npm \
    git \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a new user for building
RUN useradd -m -u 1000 rust

# Set up working directory and permissions
WORKDIR /app
RUN chown rust:rust /app

# Switch to non-root user
USER rust

# Install wasm-pack and trunk with explicit version
RUN rustup target add wasm32-unknown-unknown && \
    cargo install --locked trunk && \
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Copy project files with proper ownership
COPY --chown=rust:rust . .

# Install npm dependencies and build Tailwind
RUN npm install -D tailwindcss && \
    npx tailwindcss init && \
    npx tailwindcss -i ./input.css -o ./style/output.css

# Build the application
RUN trunk build --release