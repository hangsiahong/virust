import { renderToStaticMarkup } from 'react-dom/server';
import readline from 'readline';
import path from 'path';

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

const renderer = {
  async render(componentPath, props) {
    try {
      const validatedPath = validateComponentPath(componentPath);
      const validatedProps = validateProps(props);

      const importPromise = import(validatedPath);
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Component import timeout')), 5000)
      );

      const component = await Promise.race([importPromise, timeoutPromise]);

      if (!component || typeof component.default !== 'function') {
        throw new Error('Component must have a default export that is a function');
      }

      const React = await import('react');

      const html = renderToStaticMarkup(
        React.createElement(component.default, validatedProps)
      );

      return {
        html,
        hydrationData: JSON.stringify(validatedProps)
      };
    } catch (error) {
      return {
        success: false,
        error: error.message
      };
    }
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

    if (request.type === 'render') {
      const result = await renderer.render(request.component, request.props);
      console.log(JSON.stringify(result));
    }
  } catch (error) {
    console.log(JSON.stringify({ error: error.message }));
  }
});
