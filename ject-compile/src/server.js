const express = require('express');
const babel = require('./babel');
const makePromiseRouter = require('express-promise-router');

const PORT = 1951;
const app = express();

const apiRouter = makePromiseRouter();
app.use('/api', apiRouter);

app.get('/', (req, res) => {
  console.log(`[ject-compile] index route hit by`, req.ip);

  res.status(404).json({
    errId: 'ject_compile::index',
    message: `ject-compile has no index route`,
  });
});

app.get('/health', (req, res) => {
  res.status(200).json({
    healthy: true,
  });
});

apiRouter.get('/', async (req, res) => {
  res.status(404).json({
    errId: 'ject_compile::api::index',
    message: `ject-compile has no index route at /api`,
  });
});

apiRouter.post('/babel', express.json(), express.text(), async (req, res) => {
  const bodyRaw = typeof req.body === 'string' ? { code: req.body } : req.body;
  if (typeof bodyRaw.code !== 'string') {
    res.status(422).json({
      errId: 'ject_compile::babel::bad_body',
      message: `expected body to be text/plain of the JS code, or JSON with .code being a string`,
    });
    return;
  }

  try {
    const output = await babel.defaultCompile(bodyRaw.code);
    res.setHeader('content-type', 'application/javascript');
    res.end(output.code);
  } catch (error) {
    console.error(
      `[ject-compile] babel failed on code (rendered in json):`,
      JSON.stringify(bodyRaw.code),
    );
    console.error(`[ject-compile] babel error for above code:`, error);
    res.status(422).json({
      errId: 'ject_compile::babel::compiler_error',
      message: String(error.message),
    });
  }
});

apiRouter.use((err, req, res, next) => {
  void next;
  res.status(500).json({
    errId: 'ject_compile::api::internal_server_error',
    message: String(err.message),
  });
});

const server = app
  .listen(PORT, () => {
    console.log(`[ject-compile] Listening on port ${PORT}`);
  })
  .on('error', (err) => {
    console.error(`[ject-compile] Failed to listen on port ${PORT}`, err);
    process.exit(1);
  });

// Ref: https://medium.com/@becintec/building-graceful-node-applications-in-docker-4d2cd4d5d392
const signals = {
  SIGHUP: 1,
  SIGINT: 2,
  SIGTERM: 15,
};
const shutdown = (signal, value) => {
  console.log('shutdown!');
  server.close(() => {
    console.log(`server stopped by ${signal} with value ${value}`);
    process.exit(128 + value);
  });
};
Object.keys(signals).forEach((signal) => {
  process.on(signal, () => {
    console.log(`process received a ${signal} signal`);
    shutdown(signal, signals[signal]);
  });
});
