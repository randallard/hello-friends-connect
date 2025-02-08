FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Install Node.js and npm
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get update && apt-get install -y \
    nodejs \
    && rm -rf /var/lib/apt/lists/*

# Create dist directory
RUN mkdir -p dist

# Install Tailwind locally in the project
RUN npm install -D tailwindcss

# Build Tailwind CSS
RUN npx tailwindcss -i ./input.css -o ./dist/tailwind.css

# Install trunk and add wasm target
RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown

# Build the application
RUN trunk build --release

# Runtime stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]