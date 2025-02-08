# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Install Node.js and npm
RUN apt-get update && apt-get install -y \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install and configure Tailwind
RUN npm install -D tailwindcss && \
    npx tailwindcss init

# Install trunk
RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown

# Build the application
RUN npx tailwindcss -i ./input.css -o ./style/output.css || true
RUN trunk build --release

# Runtime stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]