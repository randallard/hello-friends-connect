
# Friends Connect

A real-time web application for connecting friends with [friends-connect](https://github.com/randallard/friends-connect) with Leptos and Rust.

I have this running at

http://64.181.233.1/hello-friends

## Dependencies

- Rust (latest stable)
- wasm-pack
- Node.js (for Tailwind CSS)
- Trunk (`cargo install trunk`)

## Setup

1. Clone the repository:
```bash
git clone ssh://git@ssh.github.com:443/randallard/hello-friends-connect.git
cd hello-friends-connect
```

2. Install Tailwind CSS:
```bash
npm install -D tailwindcss
npx tailwindcss init
```

## Running Locally

### Windows
```bash
# Terminal 1 - Run Tailwind
npx tailwindcss -i ./input.css -o ./style/output.css --watch

# Terminal 2 - Run the app
trunk serve --open
```

### Linux
```bash
# Terminal 1 - Run Tailwind
npx tailwindcss -i ./input.css -o ./style/output.css --watch

# Terminal 2 - Run the app
trunk serve --open
```

## Testing

Run WASM tests:
```bash
wasm-pack test --headless --firefox
```
on the first run this failed to test - but a firefox update ran automatically and I ran the command again successfully

Run Rust tests:
```bash
cargo test
```

## Development

The application uses:
- Leptos for the frontend framework
- Tailwind CSS for styling
- WebAssembly for browser-side logic
- Rust for type-safe, performant code

## License

MIT
```

Would you like me to add any additional sections or details to the README?