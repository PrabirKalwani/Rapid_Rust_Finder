# OS :- Rust based file system

This Readme provides step step instuctions on how run the application .

## Run this locally on you machine

1. For MacOS/Linux Devices
   ```bash
   brew tap prabirkalwani/rust-finder
   brew install rust-finder
   ```
2. For Windows Vist the release page and download the latest release and run the executable.

## Developnment Environment

Ensure you have the following software installed based on your operating system:

### macOS

1. [Homebrew](https://brew.sh/) (for managing packages)
2. Install dependencies:
   ```bash
   brew install rustup
   rustup-init
   brew install node
   ```

### Linux (Ubuntu/Debian)

1. Install dependencies:
   ```bash
   sudo apt update
   sudo apt install libwebkit2gtk-4.0-dev build-essential curl nodejs npm
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Windows

1. Install [Node.js](https://nodejs.org/) (includes npm).
2. Install [Rust](https://www.rust-lang.org/tools/install).
3. Install required Windows packages:
   ```bash
   npm install -g windows-build-tools
   ```

---

## Setup

1. **Clone the repository:**

   ```bash
   git clone <repository-url>
   cd <repository-folder>
   ```

2. **Install dependencies:**

   ```bash
   npm install
   ```

3. **Install Tauri CLI:**
   ```bash
   cargo install tauri-cli
   ```

---

## Development

To run the app in development mode:

```bash
npm run tauri dev
```

This will start the Vite dev server, bundle the app, and open the Tauri window with live reloading.

---

## Build

To run the app in release mode:

```bash
npm run tauri build
```

This will build a os specific binary in the `src-tauri/target/release` folder.

---
