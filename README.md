# Development

Prerequisites:

- node/npm >=14
- rustc/cargo (latest nightly release)

Add the following two entries to /etc/hosts:

```
127.0.0.1 ject.dev.local ject.link.local
```

Start webpack's server in one terminal tab/pane:

```sh
$ npm install
$ npm run dev
```

In another terminal session start the API server (will take a bit for the first
compile):

```sh
$ export RUST_BACKTRACE=1
$ cargo run

# Wait for this line:
Starting server on 0.0.0.0:1950

# Alternative (auto-restart)
$ cargo install cargo-watch
$ cargo watch -c -w server/ -x run

# Watch and run tests
$ RUST_BACKTRACE=1 cargo watch -c -w server/ -x test
```

Then open your browser to http://ject.dev.local:1950/

<kbd>ctrl-c</kbd> and `cargo run` after updating a `.rs` file

# Deployment

Setup entries in ~/.ssh/config for `ject` and `ject-root`.

Run `cargo xtask provision` which will setup the server using `ject-root` (see
./xtask/src/main.rs->fn provision).

Ensure DNS is pointing at the server for both domain names, and then run certbot over
ssh (`ssh ject-root`) with: `sudo certbot --nginx` which asks 4 questions:

- contact email: letsencrypt-ject@brigand.me
- accept TOS: Y
- give email to EFF: N
- domains: ject.dev,ject.link

Edit `/etc/nginx/sites-enabled/default` searching for
`server_name ject.dev ject.link`, and replace the `location /` block with the
following:

```nginx
location / {
  proxy_set_header Host $host;
  proxy_set_header X-Real-IP $remote_addr;
  proxy_pass http://localhost:1950;
}
```

Then notify nginx of the changed config with `systemctl reload nginx`.
