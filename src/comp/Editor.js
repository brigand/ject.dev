import React from 'react';

// import * as monaco from 'monaco-editor/esm/vs/editor/editor.main.js';
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
import styled from '@emotion/styled';
import pt from 'prop-types';
import andromeda from '../theme/andromeda-monaco.json';
import { EventType } from '../EventType';

const Root = styled.div`
  height: 100%;
`;

const Inner = styled.div`
  height: 100%;
`;

// const join = (a, b) => a.replace(/[/]$/, '') + '/' + b.replace(/^\.?\// < '');
// const pub = (path) => join(__webpack_public_path__, path);

// https://github.com/microsoft/monaco-editor/issues/264#issuecomment-289911286
monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
  target: monaco.languages.typescript.ScriptTarget.ES2020,
  allowNonTsExtensions: true,
  moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
  module: monaco.languages.typescript.ModuleKind.ES2020,
  noEmit: true,
  typeRoots: ['node_modules/@types'],
  jsx: true,
  jsxFactory: 'React.createElement',
  allowJs: true,
});

monaco.languages.typescript.typescriptDefaults.addExtraLib(
  require('!raw-loader!@types/react/index.d.ts').default,
  'node_modules/@types/react/index.d.ts',
);

monaco.languages.typescript.typescriptDefaults.addExtraLib(
  `
declare var root: HTMLDivElement;
  `.trim(),
  'node_modules/@types/inject-client/index.d.ts',
);

monaco.languages.typescript.typescriptDefaults.addExtraLib(
  require('!raw-loader!@types/react-dom/index.d.ts').default,
  'node_modules/@types/react-dom/index.d.ts',
);

// monaco.languages.typescript.typescriptDefaults.addExtraLib(
//   `declare var React: require('react');
//   declare var ReactDOM: require('react-dom');`,
//   'node_modules/@types/inject-global/index.d.ts',
// );

monaco.languages.typescript.typescriptDefaults.setDiagnosticsOptions({
  noSemanticValidation: false,
  noSyntaxValidation: false,
});

let registeredTheme = false;

function extension(language) {
  switch (language) {
    case 'javascript':
      return 'jsx';
    case 'typescript':
      return 'tsx';
    case 'css':
      return 'css';
    case 'html':
      return 'html';
    case 'json':
      return 'json';
    default:
      return 'txt';
  }
}

function getPrettierOpts(language) {
  switch (language) {
    case 'javascript':
      return ['babel', () => import('prettier/esm/parser-babel.mjs')];
    case 'typescript':
      return ['typescript', () => import('prettier/esm/parser-typescript.mjs')];
    case 'css':
      return ['css', () => import('prettier/esm/parser-postcss.mjs')];
    case 'html':
      return ['html', () => import('prettier/esm/parser-html.mjs')];
    case 'json':
      break;
  }

  return [];
}

function Editor(props) {
  const containerRef = React.useRef();
  const editorRef = React.useRef();

  const init = () => {
    if (!registeredTheme) {
      monaco.editor.defineTheme('andromeda', andromeda);
      registeredTheme = true;
    }

    const ed = monaco.editor.create(containerRef.current, {
      // language: props.language,
      model: monaco.editor.createModel(
        props.value.contents,
        props.language,
        monaco.Uri.parse(`file:///your-code.${extension(props.language)}`),
      ),
      theme: 'andromeda',
      fontSize: 16,
      scrollBeyondLastLine: false,
      minimap: {
        enabled: false,
      },
      cursorBlinking: 'solid',
      cursorSurroundingLines: 10,
      formatOnPaste: true,
      padding: {
        top: 8,
        bottom: 8,
      },
    });

    // Ref: https://microsoft.github.io/monaco-editor/playground.html#interacting-with-the-editor-adding-an-action-to-an-editor-instance
    // Ref: https://microsoft.github.io/monaco-editor/api/enums/monaco.keycode.html
    ed.addAction({
      id: 'ject-save',

      // A label of the action that will be presented to the user.
      label: 'Save',

      // An optional array of keybindings for the action.
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KEY_S],
      contextMenuGroupId: 'custom',
      contextMenuOrder: 10,
      run: function () {
        props.events.save.emit();
      },
    });

    const [parser, loadPlugin] = getPrettierOpts(props.language);
    if (parser) {
      ed.addAction({
        id: 'ject-format',

        // A label of the action that will be presented to the user.
        label: 'Format (prettier)',

        // An optional array of keybindings for the action.
        keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KEY_D],
        contextMenuGroupId: 'custom',
        contextMenuOrder: 10,
        run: function () {
          const value = ed.getModel().getValue();
          Promise.all([import('prettier/esm/standalone.mjs'), loadPlugin()]).then(
            ([prettier, plugin]) => {
              const outCode = prettier.default.format(value, {
                plugins: [plugin.default],
                parser,
              });
              ed.getModel().setValue(outCode);
            },
          );
        },
      });
    }

    ed.addAction({
      id: 'inject-run',
      label: 'Run',
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
      contextMenuGroupId: 'custom',
      contextMenuOrder: 10.1,
      run: function () {
        props.events.run.emit();
      },
    });

    ed.onDidChangeModelContent(() => {
      const value = ed.getModel().getValue();
      props.onChange(value);
    });

    editorRef.current = ed;
  };

  props.events.resize.use(() => {
    if (editorRef.current) {
      editorRef.current.layout();
    }
  });

  React.useEffect(() => {
    if (monaco) {
      init();
    }
  }, []);

  return (
    <Root>
      <Inner ref={containerRef} />
    </Root>
  );
}

Editor.propTypes = {
  onChange: pt.func,
  language: pt.string.isRequired,
  value: pt.shape({
    contents: pt.string.isRequired,
    version: pt.number.isRequired,
  }).isRequired,
  events: pt.shape({
    resize: pt.instanceOf(EventType).isRequired,
    save: pt.instanceOf(EventType).isRequired,
    run: pt.instanceOf(EventType).isRequired,
  }),
};

export default Editor;
