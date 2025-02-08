FROM rust:1.75-slim-bullseye as builder

# Install system dependencies with specific versions
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    nodejs \
    npm \
    git \
    curl \
    ca-certificates \
    build-essential \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Set up rust toolchain explicitly
RUN rustup default stable && \
    rustup target add wasm32-unknown-unknown

# Install trunk with explicit version
RUN cargo install trunk --version 0.17.5 --locked

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /app
COPY . .

# Build steps
RUN npm install -D tailwindcss && \
    npx tailwindcss init && \
    npx tailwindcss -i ./input.css -o ./style/output.css

# Build the application
RUN trunk build --release

# Runtime stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]