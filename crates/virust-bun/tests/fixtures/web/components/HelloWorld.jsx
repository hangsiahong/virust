import React from 'react';

export default function HelloWorld({ name }) {
  return React.createElement('div', null, `Hello ${name}`);
}
