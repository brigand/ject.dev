const babelPluginSkypack = ({ types: t }) => {
  return {
    name: 'skypack',
    visitor: {
      ImportDeclaration(path) {
        const value = path.node.source.value;
        // Matches e.g. 'http' but not 'http://...'
        if (/^[\w\d_-]+(?!:)/.test(value)) {
          path.node.source.value = `https://cdn.skypack.dev/${value}`;
        }
      },
    },
  };
};

module.exports = babelPluginSkypack;
