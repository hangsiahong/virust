# Virust Bun Renderer

This directory contains the Bun-side renderer script for SSR.

## Installation

```bash
cd crates/virust-bun/bundled
bun install
```

## Files

- `renderer.js` - Main renderer that imports and renders React components
- `package.json` - Dependencies (React, ReactDOM)

## Usage

The renderer is invoked by Virust's BunRenderer via IPC.
