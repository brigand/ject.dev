import React from 'react';

// import * as monaco from 'monaco-editor/esm/vs/editor/editor.main.js';
import styled from '@emotion/styled';
import pt from 'prop-types';
import andromeda from '../theme/andromeda-monaco.json';
import { EventType } from '../EventType';
import useMonaco from '../hooks/useMonaco';

const Root = styled.div`
  height: 100%;
`;

const Inner = styled.div`
  height: 100%;
`;

let registeredTheme = false;

function extension(language) {
  switch (language) {
    case 'javascript':
      return 'jsx';
    case 'typescript':
      return 'ts';
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
  const monaco = useMonaco();

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
        // monaco.Uri.parse(`inmemory://src/index.${extension(props.language)}`),
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

    if (props.language === 'javascript' || props.language === 'typescript') {
      monaco.AutoTypings.create(ed, {});
    }

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
        label: 'Format (Prettier)',

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
                printWidth: 86,
                tabWidth: 2,
                singleQuote: true,
                trailingComma: 'all',
                bracketSpacing: true,
                arrowParens: 'always',
                proseWrap: 'always',
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
      return () => {
        editorRef.current = null;
      };
    }
  }, [monaco]);

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
