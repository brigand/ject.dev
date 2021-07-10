const path = require('path');
const { inspect } = require('util');
const { DefinePlugin } = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = defineConfig((env, argv) => ({
  mode: argv.mode || 'development',
  entry: {
    app: './src/index.js',
    'editor.worker': 'monaco-editor/esm/vs/editor/editor.worker.js',
    'json.worker': 'monaco-editor/esm/vs/language/json/json.worker',
    'css.worker': 'monaco-editor/esm/vs/language/css/css.worker',
    'html.worker': 'monaco-editor/esm/vs/language/html/html.worker',
    'ts.worker': 'monaco-editor/esm/vs/language/typescript/ts.worker',
  },
  output: {
    globalObject: 'self',
    filename: '[name].bundle.js',
    path: path.resolve(__dirname, 'dist'),
  },
  devServer: {
    // contentBase: './dist',
    compress: true,
    allowedHosts: ['localhost', 'enject.org.local'],
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: 'src/index.html',
    }),
    new DefinePlugin({
      'process.env': {
        NODE_ENV: JSON.stringify(argv.mode || 'development'),
        INJECT_DOMAIN_MAIN: JSON.stringify('enject.org'),
        INJECT_DOMAIN_FRAME: JSON.stringify('ject.link'),
      },
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader'],
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

function defineConfig(factory) {
  if (process.env.DEBUG_WEBPACK) {
    return (...args) => {
      console.log(`Webpack config arguments`, ...args);
      const result = factory(...args);
      console.log(inspect(result, { depth: 7 }));
      return result;
    };
  } else {
    return factory;
  }
}
