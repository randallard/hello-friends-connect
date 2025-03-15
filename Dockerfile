FROM rust:1.81 as builder

ARG API_BASE=http://64.181.233.1/friends

# Prevent tzdata from requesting interactive input
ENV DEBIAN_FRONTEND=noninteractive

ENV API_BASE=$API_BASE

# Install basic dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    nodejs \
    npm \
    git \
    && rm -rf /var/lib/apt/lists/*

RUN npm install n -g && n stable

# Add wasm target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install trunk
RUN cargo install trunk

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

RUN npm install n -g && n stable    
RUN npm install tailwindcss @tailwindcss/cli
RUN npx @tailwindcss/cli -i ./input.css -o ./output.css

RUN cat Trunk.toml
RUN echo "Building with public_url = /hello-friends/ and API_BASE = $API_BASE"
RUN trunk build --release --public-url="/hello-friends/"

# Expose port
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80