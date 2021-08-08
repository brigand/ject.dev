const babel = require('@babel/core');

const makeOptions = ({ react = true }) => {
  return {
    filename: 'page.js',
    babelrc: false,
    presets: [
      react && [
        '@babel/preset-react',
        {
          runtime: 'classic',
          development: true,
          useSpread: true,
        },
      ],
    ].filter(Boolean),
  };
};

/**
 * @returns {Promise<{ code: string, map: string }>}
 */
const transformInternalRaw = (inputCode, options) => {
  return new Promise((resolve, reject) => {
    babel.transform(inputCode, options, (err, data) =>
      err ? reject(err) : resolve({ code: data.code, map: data.map }),
    );
  });
};

/**
 * @returns {Promise<{ code: string, map: string }>}
 */
const transformInternal = async (inputCode, options) => {
  const { code, map } = await transformInternalRaw(inputCode, options);
  return { code: replaceCwd(code), map };
};

const replaceCwd = (code) => {
  const cwd = process.cwd();
  let result = code;

  while (result.includes(cwd)) {
    result = result.replace(cwd, '/ject');
  }
  return result;
};

/**
 * Transform some code, with output suitable for evergreen browsers.
 *
 * @param {string} code - The JS code (with JSX allowed)
 * @returns {Promise<{ code: string, map: string }>}
 */
const defaultCompile = async (code) => {
  return transformInternal(code, makeOptions({ react: true }));
};

exports.defaultCompile = defaultCompile;
