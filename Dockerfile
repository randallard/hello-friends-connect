# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Install Node.js and npm with explicit version
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get update && apt-get install -y \
    nodejs \
    && rm -rf /var/lib/apt/lists/*

# Verify Node.js and npm installation
RUN node --version && npm --version

# Install Tailwind CSS globally and initialize
RUN npm install -g tailwindcss && \
    export PATH="/app/node_modules/.bin:$PATH" && \
    tailwindcss init

# Install trunk and add wasm target
RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown

# Build the application
RUN tailwindcss -i ./input.css -o ./style/output.css || true
RUN trunk build --release

# Runtime stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]