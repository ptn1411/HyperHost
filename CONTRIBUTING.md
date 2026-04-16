# Contributing to HyperHost

Thank you for your interest in contributing to **HyperHost** — a Local HTTPS Domain Manager built with Tauri v2, Rust, React, and TypeScript.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [How to Contribute](#how-to-contribute)
- [Commit Convention](#commit-convention)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Reporting Issues](#reporting-issues)

## Getting Started

1. **Fork** the repository on GitHub
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/HyperHost.git
   cd HyperHost
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/ptn1411/HyperHost.git
   ```

## Development Setup

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) (stable toolchain)
- [Tauri CLI v2](https://tauri.app/start/prerequisites/)

On Windows, you also need:
- Microsoft Visual Studio Build Tools (C++ workload)
- WebView2 Runtime

### Install Dependencies

```bash
npm install
```

### Run in Development Mode

```bash
npm run tauri dev
```

### Build for Production

```bash
npm run tauri build
```

### Run Frontend Only (Vite)

```bash
npm run dev
```

## Project Structure

```
HyperHost/
├── src/                    # React + TypeScript frontend
├── src-tauri/              # Rust backend (Tauri)
│   ├── src/
│   │   ├── bin/cli.rs      # CLI binary (hyh)
│   │   └── ...             # Core domain management logic
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── tsconfig.json
```

## How to Contribute

### Fixing Bugs

- Check [existing issues](https://github.com/ptn1411/HyperHost/issues) first
- Create a new issue describing the bug before submitting a fix

### Adding Features

- Open an issue to discuss the feature before implementing
- Keep changes focused — one feature per PR

### Improving Documentation

Documentation improvements are always welcome. No issue required for typo fixes or small clarifications.

## Commit Convention

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add mkcert auto-renewal support
fix: resolve nginx config path on Windows
docs: update development setup instructions
refactor: simplify domain validation logic
chore: bump Tauri to v2.x
```

## Pull Request Guidelines

- Branch from `main` and target `main`
- Keep PRs small and focused
- Include a clear description of what changed and why
- Ensure the project builds without errors:
  ```bash
  npm run build
  cargo check --manifest-path src-tauri/Cargo.toml
  ```
- Reference the related issue in the PR description (`Closes #123`)

## Reporting Issues

Use [GitHub Issues](https://github.com/ptn1411/HyperHost/issues) and include:

- OS and version (Windows 10/11, macOS, Linux distro)
- HyperHost version
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or screenshots

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).

