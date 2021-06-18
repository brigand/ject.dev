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
      <>
        <Editor
          resize={resize}
          language="html"
          onChange={(value) => {
            console.log(`HTML Changed to:`, value);
          }}
          value={{
            current: `<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />

    <link rel="stylesheet" href="inject!(panels.css.url)" />
  </head>

  <body>
    <div id="root"></div>
    <script src="inject!(panels.js.url)"></script>
  </body>
</html>
`,
            version: 1,
          }}
        />
      </>
      <>
        <Editor
          resize={resize}
          language="javascript"
          onChange={(value) => {
            console.log(`Changed to:`, value);
          }}
          value={{
            current: `// This is an example
const f = () => location.href.toLowerCase();
console.log(f());
const div = <div className={css.foo}>Hello, world!</div>;
`,
            version: 1,
          }}
        />
      </>
      <div>css</div>
      <div>output</div>
    </QuadSplit>
  );
}

render(<App />, document.getElementById('root'));
