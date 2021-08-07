const objectInspect = require('object-inspect');

const postMessage = window.parent.postMessage.bind(window.parent);

const normalMethods = ['log', 'info', 'error', 'warn'];

let { JECT_DOMAIN_MAIN, JECT_DOMAIN_FRAME } = process.env;
if (location.hostname === `${JECT_DOMAIN_FRAME}.local`) {
  JECT_DOMAIN_MAIN += '.local';
}

const getTargetOrigin = () => {
  const url = new URL(window.location);
  url.hostname = JECT_DOMAIN_MAIN;
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
          typeof arg === 'string'
            ? arg.slice(0, 1024 * 16)
            : objectInspect(arg, {
                customInspect: false,
                maxStringLength: 1024 * 32,
              }),
        ),
      },
      targetOrigin,
    );
    original(...args);
  };
});

window.addEventListener('error', (event) => {
  console.error(event.error);
});
