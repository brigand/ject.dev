mod api;
mod cdn;
mod db;
mod env;
mod http_error;
mod ids;
// mod js;
mod parser;
mod state;

use std::sync::Arc;

use actix_web::{
    client::{self, SendRequestError},
    get,
    // middleware::Logger,
    App,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder,
};

use crate::db::Db;

#[get("/")]
async fn r_index() -> impl Responder {
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
    let client = client::Client::default();

    // Send a request to the webpack server
    let path = req.match_info().get("tail").unwrap_or("");
    let url = format!("http://localhost:1800/{}", path);

    // It actually seems to be about twice as slow if we enable compression, so disabled currently.
    // let wp_req = client.get(&url);
    // if let Some(accept_encoding) = req.headers().get("accept-encoding") {
    //     wp_req = wp_req.set_header("accept-encoding", accept_encoding.clone());
    // }
    // let wp_res = wp_req.send().await?;

    let wp_res = client.get(&url).send().await?;

    let mut res = HttpResponse::build(wp_res.status());
    for (key, value) in wp_res.headers() {
        if key.as_str().eq_ignore_ascii_case("transfer-encoding") {
            continue;
        }

        res.header(key, value.clone());
    }
    Ok(res.streaming(wp_res))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    Db::open_env().await?.create_tables().await?;
    println!("Created tables");

    let bind = "0.0.0.0:1950";
    println!("Starting server on {}", bind);

    HttpServer::new(move || {
        // let logger = Logger::default().exclude("/dist/");
        App::new()
            // .wrap(logger)
            .service(r_index)
            .service(r_dist)
            .service(api::service())
    })
    .bind(bind)?
    .run()
    .await?;

    Ok(())
}
