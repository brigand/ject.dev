import React from 'react';
import * as monaco from 'monaco-editor/esm/vs/editor/editor.main.js';

let promise = null;
// let monaco = null;

self.MonacoEnvironment = {
  getWorkerUrl: function (moduleId, label) {
    if (label === 'json') {
      return './json.worker.js';
    }
    if (label === 'css' || label === 'scss' || label === 'less') {
      return './css.worker.js';
    }
    if (label === 'html' || label === 'handlebars' || label === 'razor') {
      return './html.worker.js';
    }
    if (label === 'typescript' || label === 'javascript') {
      return './ts.worker.js';
    }
    return './editor.worker.js';
  },
};

function Editor(props) {
  const editorRef = React.useRef();
  const [editor, setEditor] = React.useState(null);

  const init = () => {
    monaco.editor.create(editorRef.current, {
      value: "function hello() {\n\talert('Hello world!');\n}",
      language: 'javascript',
    });
  };

  React.useEffect(() => {
    if (monaco) {
      init();
    } else {
      if (!promise) {
        promise = import('monaco-editor/esm/vs/editor/editor.main.js').then((mod) => {
          // promise = import('monaco-editor/dev/vs/editor/editor.main.js').then((mod) => {
          // monaco = { ...mod };
        });
      }

      promise.then(init);
    }
  }, []);

  return (
    <div>
      <div ref={editorRef} />
    </div>
  );
}

export default Editor;
