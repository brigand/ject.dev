mod api;
mod cdn;
mod compile_service;
mod db;
mod env;
mod http;
mod http_error;
mod ids;
// mod js;
mod parser;
mod state;

use actix_files as fs;

use actix_web::{
    client::{
        SendRequestError, {self},
    },
    get,
    middleware::Logger,
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use env::is_production;
use fs::NamedFile;
use ov::*;

use crate::db::Db;

async fn r_index() -> impl Responder {
    let html = r##"<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8" />
        <title>ject.dev</title>
        <script>
            // Used by global-init.js
            window.PUBLIC_PATH = '/dist/';
        </script>
        <script async defer src="/dist/app.bundle.js"></script>

        <link rel="apple-touch-icon" sizes="180x180" href="/dist/apple-touch-icon.png">
        <link rel="icon" type="image/png" sizes="32x32" href="/dist/favicon-32x32.png">
        <link rel="icon" type="image/png" sizes="16x16" href="/dist/favicon-16x16.png">
        <link rel="manifest" href="/dist/site.webmanifest">
        <link rel="mask-icon" href="/dist/safari-pinned-tab.svg" color="#ffe66d">
        <link rel="stylesheet" href="/dist/app.css">
        <meta name="msapplication-TileColor" content="#ffe66d">
        <meta name="theme-color" content="#ffe66d">
    </head>

    <body>
        <div id="root"></div>
    </body>
</html>"##;
    HttpResponse::Ok()
        .header("content-type", "text/html")
        .body(html)
}

#[get("/favicon.ico")]
async fn r_favicon(_req: HttpRequest) -> Result<impl Responder, SendRequestError> {
    let file = if is_production() {
        NamedFile::open("dist/favicon.ico")?
    } else {
        NamedFile::open("public/favicon.ico")?
    };
    Ok(file)
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

    // let domain_main = env::domain_main();
    // let domain_frame = env::domain_frame();

    let bind = if env::is_production() {
        "127.0.0.1:1950"
    } else {
        "0.0.0.0:1950"
    };

    let server = HttpServer::new(move || {
        let logger = Logger::default().exclude("/dist/");
        App::new()
            .wrap(logger)
            .route("/", actix_web::web::get().to(r_index))
            .route("/new/{templateName}", actix_web::web::get().to(r_index))
            .service(r_favicon)
            .over(|app| {
                if env::is_production() {
                    app.service(fs::Files::new("/dist", "./dist"))
                } else {
                    app.service(r_dist)
                }
            })
            .service(api::service())
    });

    println!("Starting server on {}", bind);
    server.bind(bind)?.run().await?;

    Ok(())
}
