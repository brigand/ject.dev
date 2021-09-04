import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
import { AutoTypings } from 'monaco-editor-auto-typings';

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

export * from 'monaco-editor/esm/vs/editor/editor.api';
export { AutoTypings };
