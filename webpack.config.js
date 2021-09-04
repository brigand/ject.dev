const path = require('path');
const { inspect } = require('util');
const { DefinePlugin } = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyPlugin = require('copy-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');
const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

const extra = {
  enableBundlerAnalyzer: !!process.env.MEASURE,
};

module.exports = defineConfig((env, argv) => ({
  ...(() => {
    console.log('ject webpack config args:', { env, argv });
  })(),
  mode: argv.mode || 'development',
  entry: {
    app: './src/index.js',
    console: './src/console.js',
    // 'editor.worker': 'monaco-editor/esm/vs/editor/editor.worker.js',
    // 'json.worker': 'monaco-editor/esm/vs/language/json/json.worker',
    // 'css.worker': 'monaco-editor/esm/vs/language/css/css.worker',
    // 'html.worker': 'monaco-editor/esm/vs/language/html/html.worker',
    // 'ts.worker': 'monaco-editor/esm/vs/language/typescript/ts.worker',
  },
  output: {
    globalObject: 'self',
    filename: '[name].bundle.js',
    chunkFilename: '[name].[contenthash].chunk.js',
    path: path.resolve(__dirname, 'dist'),
  },
  resolve: {
    alias: {
      path: require.resolve('path-browserify'),
    },
  },
  devServer: {
    // contentBase: './dist',
    compress: true,
    allowedHosts: ['localhost', 'ject.dev.local', 'ject.link.local'],
  },
  plugins: [
    new CleanWebpackPlugin(),
    new CopyPlugin({
      patterns: [{ from: 'public' }],
    }),
    new HtmlWebpackPlugin({
      template: 'src/index.html',
    }),
    new DefinePlugin({
      'process.env': {
        NODE_ENV: JSON.stringify(argv.mode || 'development'),
        JECT_DOMAIN_MAIN: JSON.stringify('ject.dev'),
        JECT_DOMAIN_FRAME: JSON.stringify('ject.link'),
      },
    }),
    extra.enableBundlerAnalyzer
      ? new BundleAnalyzerPlugin({
          analyzerMode: env.WEBPACK_SERVE ? 'server' : 'static',
          // analyzerMode: 'static',
          openAnalyzer: false,
        })
      : null,
    new MonacoWebpackPlugin({
      languages: [
        'css',
        'html',
        'typescript',
        //'javascript', 'json',
      ],
      features: [
        // 'accessibilityHelp',
        // 'anchorSelect',
        'bracketMatching',
        'caretOperations',
        // 'clipboard',
        'codeAction',
        'codelens',
        // 'colorPicker',
        'comment',
        'contextmenu',
        'coreCommands',
        // 'cursorUndo',
        // 'dnd',
        'documentSymbols',
        'find',
        // 'folding',
        // 'fontZoom',
        'format',
        'gotoError', // ?
        'gotoLine', // ?
        'gotoSymbol', // ?
        'hover',
        'iPadShowKeyboard', // ?
        'inPlaceReplace',
        'indentation',
        'inlayHints',
        'inlineCompletions',
        'inspectTokens',
        'linesOperations',
        // 'linkedEditing', // ?
        'links', // ?
        'multicursor',
        'parameterHints',
        'quickCommand', // ?
        'quickHelp', // ?
        'quickOutline', // ?
        // 'referenceSearch',
        'rename',
        'smartSelect', // ?
        'snippets',
        'suggest',
        // 'toggleHighContrast',
        // 'toggleTabFocusMode',
        'transpose',
        // 'unusualLineTerminators',
        'viewportSemanticTokens',
        'wordHighlighter',
        'wordOperations',
        'wordPartOperations',
      ],
    }),
    new MiniCssExtractPlugin({
      // [name].css is the default filename
      filename: '[name].css',
      chunkFilename: '[name].[contenthash].chunk.css',
    }),
  ].filter(Boolean),
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          argv.mode === 'production' ? MiniCssExtractPlugin.loader : 'style-loader',
          'css-loader',
        ],
      },
      {
        test: /\.ttf$/,
        use: ['file-loader'],
      },
      {
        test: /\.m?js$/,
        include: path.resolve('./src/'),
        exclude: /(node_modules|bower_components)/,
        use: {
          loader: 'swc-loader',
        },
      },
    ],
  },
}));

/**
 * @param {(env: { WEBPACK_SERVE?: boolean }, argv: { port?: number, progress?: boolean, config?: string[] }) => object} factory
 */
function defineConfig(factory) {
  if (process.env.DEBUG_WEBPACK) {
    return (...args) => {
      console.log(`Webpack config arguments`, ...args);
      const result = factory(...args);
      console.log(inspect(result, { depth: 7 }));
      return result;
    };
  } else {
    return (...args) => factory(...args);
  }
}
