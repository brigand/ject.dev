const path = require('path');
const { default: Parcel } = require('@parcel/core');

(async () => {
  let bundler = new Parcel({
    entries: [
      path.resolve('src/index.html'),
      path.resolve(
        './node_modules/monaco-editor/esm/vs/language/json/json.worker.js',
      ),
      path.resolve(
        './node_modules/monaco-editor/esm/vs/language/html/html.worker.js',
      ),
      path.resolve('./node_modules/monaco-editor/esm/vs/language/css/css.worker.js'),
      path.resolve(
        './node_modules/monaco-editor/esm/vs/language/typescript/ts.worker.js',
      ),
      path.resolve('./node_modules/monaco-editor/esm/vs/editor/editor.worker.js'),
    ],
    defaultConfig: require.resolve('@parcel/config-default'),
    defaultTargetOptions: {
      engines: {
        browsers: ['last 1 Chrome version'],
        node: '10',
      },
    },
    mode: 'development',
  });

  await bundler.run();
})();
