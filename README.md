# Rust Canvas App

A lightweight, high-performance web-based canvas drawing tool built with **Rust** and **WebAssembly**.

## Features

- **Drawing Tools:** Create rectangles, circles, lines, and text.
- **Selection & Manipulation:** Select shapes to move them or edit their properties.
- **Dynamic Styling:** Change colors, text content, and font sizes in real-time.
- **Canvas Management:** Resize the canvas dimensions dynamically.
- **Interactive UI:** Includes a toolbar for tool selection and a status bar for feedback.
- **Performance:** Powered by Rust and the browser's native 2D Canvas API via `web-sys`.

## Tech Stack

- **Language:** [Rust](https://www.rust-lang.org/)
- **Target:** WebAssembly (WASM)
- **Frameworks/Libraries:**
  - `wasm-bindgen` for Rust-JS interop.
  - `web-sys` for browser API bindings (Canvas, DOM, etc.).
  - `once_cell` for lazy initialization.
  - `thiserror` and `anyhow` for robust error handling.

## Prerequisites

To build and run this project, you need:

1.  **Rust & Cargo:** [Install Rust](https://www.rust-lang.org/tools/install)
2.  **wasm-pack:** [Install wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
3.  **A local web server:** (e.g., `python -m http.server`, `npx serve`, or the Live Server extension for VS Code).

## Getting Started

### 1. Build the Project

Use `wasm-pack` to compile the Rust code to WebAssembly and generate the JavaScript glue code:

```bash
wasm-pack build --target web
```

This will create a `pkg/` directory containing the WASM binary and generated JS files.

### 2. Run the Application

Serve the root directory using a local web server. For example:

```bash
# Using Python
python -m http.server 8000

# Using Node.js
npx serve .
```

Then, open your browser and navigate to `http://localhost:8000` (or the port provided by your server).

## Project Structure

- `src/lib.rs`: Main application logic, state management, and event handling.
- `src/errors.rs`: Custom error types and status reporting.
- `index.html`: The frontend entry point, containing the canvas, UI controls, and initialization script.
- `Cargo.toml`: Project dependencies and configuration.

## Testing

Run the Rust unit tests:

```bash
cargo test
```
