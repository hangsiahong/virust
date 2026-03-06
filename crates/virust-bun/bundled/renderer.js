import { renderToStaticMarkup } from 'react-dom/server';
import path from 'path';

// Validate component path is within allowed directory and has valid extension
function validateComponentPath(componentPath) {
  // Check if path is absolute
  if (!path.isAbsolute(componentPath)) {
    throw new Error('Component path must be absolute');
  }

  // Check for valid extension
  const validExtensions = ['.jsx', '.js', '.tsx', '.ts'];
  const ext = path.extname(componentPath).toLowerCase();
  if (!validExtensions.includes(ext)) {
    throw new Error(`Invalid component extension: ${ext}`);
  }

  // Check for path traversal attempts
  const normalizedPath = path.normalize(componentPath);
  if (normalizedPath !== componentPath) {
    throw new Error('Invalid component path (possible traversal attack)');
  }

  return normalizedPath;
}

// Validate props is a plain object
function validateProps(props) {
  if (props === null || typeof props !== 'object' || Array.isArray(props)) {
    throw new Error('Props must be a plain object');
  }
  return props;
}

export default {
  async render(componentPath, props) {
    try {
      // Validate inputs
      const validatedPath = validateComponentPath(componentPath);
      const validatedProps = validateProps(props);

      // Import component with timeout
      const importPromise = import(validatedPath);
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Component import timeout')), 5000)
      );

      const component = await Promise.race([importPromise, timeoutPromise]);

      // Validate component has default export
      if (!component || typeof component.default !== 'function') {
        throw new Error('Component must have a default export that is a function');
      }

      // Import React (cached at module level)
      const React = await import('react');

      // Render component
      const html = renderToStaticMarkup(
        React.createElement(component.default, validatedProps)
      );

      return {
        html,
        hydrationData: JSON.stringify(validatedProps)
      };
    } catch (error) {
      // Don't expose stack traces or internal details
      return {
        success: false,
        error: error.message
      };
    }
  }
};
