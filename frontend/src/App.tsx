import type { Component } from 'solid-js';
import { css } from '../styled-system/css';

const App: Component = () => {
  return (
    <div>
      <div class={css({ fontSize: '2xl', fontWeight: 'bold' })}>Hello 🐼!</div>
    </div>
  );
};

export default App;
