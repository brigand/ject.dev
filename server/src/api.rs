mod frame;
mod saved;
mod session;
mod util;

use actix_web::{get, web, HttpResponse, Responder, Scope};

#[cfg(debug_assert)]
const SESSION_LIMIT: u32 = 512;
#[cfg(not(debug_assert))]
const SESSION_LIMIT: u32 = 1024 * 8;

#[get("/health")]
async fn r_health() -> impl Responder {
    HttpResponse::Ok()
        .header("content-type", "text/plain; charset=utf-8")
        .body("Ok")
}

pub fn service() -> Scope {
    web::scope("/api")
        .service(r_health)
        .service(saved::r_get_saved)
        .service(saved::r_post_save)
        .service(session::r_post_session_new)
        .service(session::r_put_session)
        .service(frame::r_get_session_page_js)
        .service(frame::r_get_session_page_js_raw)
        .service(frame::r_get_session_page_css)
        .service(frame::r_get_session_page_html)
}
