const objectInspect = require('object-inspect');

const postMessage = window.parent.postMessage.bind(window.parent);

const normalMethods = ['log', 'info', 'error', 'warn'];

let { INJECT_DOMAIN_MAIN, INJECT_DOMAIN_FRAME } = process.env;
if (location.hostname === `${INJECT_DOMAIN_FRAME}.local`) {
  INJECT_DOMAIN_MAIN += '.local';
}

const getTargetOrigin = () => {
  const url = new URL(window.location);
  url.hostname = INJECT_DOMAIN_MAIN;
  return new URL(url).origin;
};

const targetOrigin = getTargetOrigin();

normalMethods.forEach((method) => {
  const original = console[method].bind(console);
  console[method] = (...args) => {
    postMessage(
      {
        type: 'console',
        method,
        args: args.map((arg) =>
          objectInspect(arg, { customInspect: false, maxStringLength: 1024 * 32 }),
        ),
      },
      targetOrigin,
    );
    original(...args);
  };
});
