# Development

Prerequisites:

- node/npm >=14
- rustc/cargo (latest nightly release)

Add the following two entries to /etc/hosts:

```
127.0.0.1 enject.org.local ject.link.local
```

Start webpack's server in one terminal tab/pane:

```sh
$ npm install
$ npm run dev
```

In another terminal session start the API server (will take a bit for the first
compile):

```sh
$ cargo run

# Wait for this line:
Starting server on 0.0.0.0:1950

# Alternative (auto-restart)
$ cargo install cargo-watch
$ cargo watch -c -x run -w server/
```

Then open your browser to http://enject.org.local:1950/

<kbd>ctrl-c</kbd> and `cargo run` after updating a `.rs` file
