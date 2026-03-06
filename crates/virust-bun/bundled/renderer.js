import { renderToStaticMarkup } from 'react-dom/server';
import readline from 'readline';
import { readFileSync } from 'fs';
import path from 'path';
import { createElement } from 'react';

// Configure JSX transform for Bun
const React = { createElement };

const RENDER_CACHE = new Map();

// Validate component path is within allowed directory and has valid extension
function validateComponentPath(componentPath) {
  if (!path.isAbsolute(componentPath)) {
    throw new Error('Component path must be absolute');
  }

  const validExtensions = ['.jsx', '.js', '.tsx', '.ts'];
  const ext = path.extname(componentPath).toLowerCase();
  if (!validExtensions.includes(ext)) {
    throw new Error(`Invalid component extension: ${ext}`);
  }

  const normalizedPath = path.normalize(componentPath);
  if (normalizedPath !== componentPath) {
    throw new Error('Invalid component path (possible traversal attack)');
  }

  return normalizedPath;
}

function validateProps(props) {
  if (props === null || typeof props !== 'object' || Array.isArray(props)) {
    throw new Error('Props must be a plain object');
  }
  return props;
}

function isClientComponent(componentPath) {
  try {
    const source = readFileSync(componentPath, 'utf-8');
    return source.includes("'use client'") || source.includes('"use client"');
  } catch {
    return false;
  }
}

async function loadComponent(componentPath) {
  if (RENDER_CACHE.has(componentPath)) {
    return RENDER_CACHE.get(componentPath);
  }

  // Clear require cache for hot reload
  if (typeof require.cache[componentPath] !== 'undefined') {
    delete require.cache[componentPath];
  }

  const component = await import(componentPath);
  RENDER_CACHE.set(componentPath, component);
  return component;
}

const renderer = {
  async render(componentPath, props) {
    try {
      const validatedPath = validateComponentPath(componentPath);
      const validatedProps = validateProps(props);

      const isClient = isClientComponent(validatedPath);

      if (isClient) {
        // Client component - return placeholder
        return {
          html: `<div data-component="${validatedPath}" data-props='${JSON.stringify(validatedProps)}' data-client></div>`,
          hydrationData: JSON.stringify(validatedProps),
          isClientComponent: true
        };
      }

      // Server component - render normally
      const component = await loadComponent(validatedPath);

      if (!component || typeof component.default !== 'function') {
        throw new Error('Component must have a default export that is a function');
      }

      const React = await import('react');
      const html = renderToStaticMarkup(
        React.createElement(component.default, validatedProps)
      );

      return {
        html,
        hydrationData: JSON.stringify(validatedProps),
        isClientComponent: false
      };
    } catch (error) {
      return {
        error: error.message,
        stack: error.stack
      };
    }
  },

  invalidate(componentPath) {
    RENDER_CACHE.delete(componentPath);
    return { ok: true };
  }
};

// IPC loop
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

rl.on('line', async (line) => {
  if (line.trim() === 'EXIT') {
    process.exit(0);
  }

  try {
    const request = JSON.parse(line);

    if (request.type === 'ping') {
      console.log(JSON.stringify({ pong: true }));
    } else if (request.type === 'render') {
      const result = await renderer.render(request.component, request.props);
      console.log(JSON.stringify(result));
    } else if (request.type === 'invalidate') {
      const result = renderer.invalidate(request.component);
      console.log(JSON.stringify(result));
    }
  } catch (error) {
    console.log(JSON.stringify({ error: error.message }));
  }
});
