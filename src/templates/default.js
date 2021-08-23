export const baseCss = `html {
  font-family: Arial, sans;
  background: #23262e;
  color: #d5ced9;
}`;

export const default$ = {
  files: [
    {
      kind: 'JavaScript',
      contents: `// JavaScript\n`,
    },
    {
      kind: 'Html',
      contents: `<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <link rel="stylesheet" href="inject!(editors.css.raw)" />

    inject!(console)
    <!-- inject!(deps.react) -->
    <!-- inject!(deps.jquery) -->
  </head>

  <body>
    <div id="root">

    </div>

    <!-- Note: Remove ".raw" to enable JSX support -->
    <script src="inject!(editors.js.raw)"></script>
    <!-- <script type="module" src="inject!(editors.js.raw)"></script> -->
  </body>
</html>
`,
    },
    {
      kind: 'Css',
      contents: baseCss,
    },
  ],
};
