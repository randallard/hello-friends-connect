FROM rust:1.78-slim-bullseye

# Prevent tzdata from requesting interactive input
ENV DEBIAN_FRONTEND=noninteractive

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

# Install Tailwind CSS
# RUN npm install -D tailwindcss@latest
# RUN npx tailwindcss init

# # Build the project
RUN trunk build

# # Expose port
EXPOSE 8080

# # Start command
CMD ["trunk", "serve", "--address", "0.0.0.0"]