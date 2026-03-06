import React from 'react';

export default function ServerComponent({ message }) {
  return React.createElement('div', { className: 'server' }, message);
}
