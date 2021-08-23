import { baseCss } from './default';

export const react = {
  files: [
    {
      kind: 'JavaScript',
      contents: `function App() {
  const [state, setState] = React.useState(null);
}

ReactDOM.render(<App />, document.getElementById('root'));`,
    },
    {
      kind: 'Html',
      contents: `<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <link rel="stylesheet" href="inject!(editors.css.raw)" />

    inject!(console)
    inject!(deps.react)
  </head>

  <body>
    <div id="root">

    </div>

    <script type="module" src="inject!(editors.js)"></script>
  </body>
</html>
`,
    },
    {
      kind: 'Css',
      contents: `${baseCss}

#root {
}`,
    },
  ],
};
