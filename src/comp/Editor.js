import React from 'react';
import * as monaco from 'monaco-editor/esm/vs/editor/editor.main.js';
import styled from '@emotion/styled';
import andromeda from '../theme/andromeda-monaco.json';

const Root = styled.div`
  height: 100%;
`;

const Inner = styled.div`
  height: 100%;
`;

// let promise = null;
// let monaco = null;

self.MonacoEnvironment = {
  getWorkerUrl: function (moduleId, label) {
    if (label === 'json') {
      return './json.worker.bundle.js';
    }
    if (label === 'css' || label === 'scss' || label === 'less') {
      return './css.worker.bundle.js';
    }
    if (label === 'html' || label === 'handlebars' || label === 'razor') {
      return './html.worker.bundle.js';
    }
    if (label === 'typescript' || label === 'javascript') {
      return './ts.worker.bundle.js';
    }
    return './editor.worker.bundle.js';
  },
};

let registeredTheme = false;

function Editor(props) {
  const editorRef = React.useRef();
  const [editor, setEditor] = React.useState(null);

  const init = () => {
    if (!registeredTheme) {
      monaco.editor.defineTheme('andromeda', andromeda);
      registeredTheme = true;
    }

    monaco.editor.create(editorRef.current, {
      value: "function hello() {\n\talert('Hello world!');\n}",
      language: 'javascript',
      theme: 'andromeda',
      fontSize: '16px',
      scrollBeyondLastLine: false,
      minimap: {
        enabled: false,
      },
    });
  };

  React.useEffect(() => {
    if (monaco) {
      init();
    } else {
      if (!promise) {
        // promise = import('monaco-editor/esm/vs/editor/editor.main.js').then((mod) => {
        // promise = import('monaco-editor/dev/vs/editor/editor.main.js').then((mod) => {
        // monaco = { ...mod };
        // });
      }

      promise.then(init);
    }
  }, []);

  return (
    <Root>
      <Inner ref={editorRef} />
    </Root>
  );
}

export default Editor;
