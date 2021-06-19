mod api;
mod state;

use crate::state::State;
use actix_web::{
    client::{self, SendRequestError},
    get, App, HttpRequest, HttpResponse, HttpServer, Responder,
};

#[get("/")]
async fn r_index() -> impl Responder {
    // Appears to work without these, despite html-webpack-plugin including them
    let _removed = r#"
    <script defer src="/dist/editor.worker.bundle.js"></script>
    <script defer src="/dist/json.worker.bundle.js"></script>
    <script defer src="/dist/css.worker.bundle.js"></script>
    <script defer src="/dist/html.worker.bundle.js"></script>
    <script defer src="/dist/ts.worker.bundle.js"></script>
    "#;

    let html = r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8" />
        <title>Inject - brigand.me</title>
        <script>
            // Used by global-init.js
            window.PUBLIC_PATH = '/dist/';
        </script>

        <script defer src="/dist/app.bundle.js"></script>
    </head>

    <body>
        <div id="root"></div>
    </body>
</html>"#;
    HttpResponse::Ok()
        .header("content-type", "text/html")
        .body(html)
}

#[get("/dist/{tail:.*}")]
async fn r_dist(req: HttpRequest) -> Result<HttpResponse, SendRequestError> {
    let path = req.match_info().get("tail").unwrap_or("");
    let client = client::Client::default();
    let url = format!("http://localhost:1800/{}", path);
    let remote = client.get(&url).send().await?;

    let mut res = HttpResponse::build(remote.status());
    for (key, value) in remote.headers() {
        res.header(key, value.clone());
    }

    Ok(res.streaming(remote))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state: State = State::default();

    let bind = "0.0.0.0:1950";
    println!("Starting server on {}", bind);

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(r_index)
            .service(r_dist)
            .service(api::service())
    })
    .bind(bind)?
    .run()
    .await
}
