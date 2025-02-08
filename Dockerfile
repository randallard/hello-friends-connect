# Build stage
FROM rust:1.75-slim-bullseye as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    nodejs \
    npm \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack and trunk
RUN cargo install trunk && \
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Create and set working directory
WORKDIR /app

# Copy project files
COPY . .

# Install npm dependencies and build Tailwind
RUN npm install -D tailwindcss && \
    npx tailwindcss init && \
    npx tailwindcss -i ./input.css -o ./style/output.css

# Build the application
RUN trunk build --release

# Runtime stage
FROM nginx:alpine

# Copy the built assets from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]