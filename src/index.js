import React from 'react';
import { render } from 'react-dom';
import Editor from './comp/Editor';
import { EventType } from './EventType';
import 'normalize.css/normalize.css';
import './index.css';

import QuadSplit from './comp/QuadSplit';

function App() {
  const [resize] = React.useState(() => new EventType());

  return (
    <QuadSplit resize={resize}>
      <div>html</div>
      <>
        <Editor resize={resize} />
      </>
      <div>css</div>
      <div>output</div>
    </QuadSplit>
  );
}

render(<App />, document.getElementById('root'));
