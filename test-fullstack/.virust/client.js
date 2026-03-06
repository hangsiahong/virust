import React, { hydrateRoot } from 'react-dom/client';

/**
 * Global component registry for client-side hydration.
 *
 * Components must be registered before DOMContentLoaded for automatic hydration.
 *
 * @example
 * // Register a component (must be done before page load)
 * window.VIRUST_COMPONENTS['/path/to/MyComponent.tsx'] = MyComponent;
 *
 * @type {Object.<string, React.ComponentType>}
 */
window.VIRUST_COMPONENTS = {};

async function hydrate() {
  const rootEl = document.getElementById('root');
  if (!rootEl) return;

  // Find all client component placeholders
  const clientComponents = rootEl.querySelectorAll('[data-client]');

  for (const el of clientComponents) {
    const componentName = el.getAttribute('data-component');

    // Parse props with error handling
    let props;
    try {
      props = JSON.parse(el.getAttribute('data-props'));
    } catch (e) {
      console.error(`Virust: Failed to parse props for "${componentName}":`, e);
      continue; // Skip this component but continue with others
    }

    // Load component from registry
    if (window.VIRUST_COMPONENTS[componentName]) {
      const Component = window.VIRUST_COMPONENTS[componentName];

      // Create container
      const container = document.createElement('div');
      el.parentNode.replaceChild(container, el);

      // Hydrate
      hydrateRoot(container, React.createElement(Component, props));
    } else {
      console.warn(`Virust: Component "${componentName}" not found in registry. Skipping hydration.`);
    }
  }
}

// Auto-hydrate on load
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', hydrate);
} else {
  hydrate();
}
