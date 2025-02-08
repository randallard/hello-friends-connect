# Build stage
FROM rust:1.75 as builder

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install trunk
RUN cargo install trunk

# Create a new empty shell project
WORKDIR /app
COPY . .

# Build the project
RUN trunk build --release

# Production stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf