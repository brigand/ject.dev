import React from 'react';
import { render } from 'react-dom';
import Editor from './comp/Editor';
import 'normalize.css/normalize.css';
import './index.css';

import QuadSplit from './comp/QuadSplit';

function App() {
  return (
    <QuadSplit>
      <div>html</div>
      <div>
        <Editor />
      </div>
      <div>css</div>
      <div>output</div>
    </QuadSplit>
  );
}

render(<App />, document.getElementById('root'));
