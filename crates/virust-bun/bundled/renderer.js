import { renderToStaticMarkup } from 'react-dom/server';

export default {
  async render(componentPath, props) {
    try {
      const component = await import(componentPath);
      const React = await import('react');
      const html = renderToStaticMarkup(
        React.createElement(component.default, props)
      );
      return {
        html,
        hydrationData: JSON.stringify(props)
      };
    } catch (error) {
      return {
        error: error.message,
        stack: error.stack
      };
    }
  }
};
