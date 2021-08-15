use crate::{
    cdn::cdnjs_script,
    db::Db,
    env::domain_frame,
    http::Host,
    http_error::{ErrorMime, HttpError},
    parser::{parse_html, HtmlPart},
    state::FileKind,
};
use actix_web::{get, web, HttpResponse};

async fn try_get_file(
    db: Db,
    session_id: &str,
    err_mime: ErrorMime,
    file_kind: FileKind,
) -> Result<(Db, String), HttpError> {
    let (db, _session) = db
        .get_session(session_id)
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;

    let res = db
        .get_file(session_id, file_kind.to_default_name())
        .await
        .map_err(|_err| HttpError::file_not_found(err_mime).with_mime(err_mime))?;

    Ok(res)
}

#[get("/session/{session_id}/page.js")]
pub async fn r_get_session_page_js(info: web::Path<String>) -> Result<HttpResponse, HttpError> {
    let err_mime = ErrorMime::JavaScript;
    let db = Db::open_env()
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;
    let session_id = info.0;
    let (_, code) = try_get_file(db, &session_id, err_mime, FileKind::JavaScript).await?;

    let code = match crate::compile_service::babel_compile(&code).await {
        Ok(code) => code,
        Err(crate::compile_service::CompileError::Compile { err_id, message }) => {
            if &err_id == "ject_compile::babel::compiler_error" {
                return Err(HttpError::js_compile_fail(message).with_mime(err_mime));
            } else {
                let message = format!(
                    "Unknown compiler err_id of {}.\nMessage: {}",
                    err_id, message
                );
                return Err(HttpError::js_compile_fail(message).with_mime(err_mime));
            }
        }
        Err(err) => {
            return Err(HttpError::js_compile_fail(err).with_mime(err_mime));
        }
    };

    Ok(HttpResponse::Ok()
        .header("content-type", "application/javascript; charset=utf-8")
        .body(code))

    // let session_id = info.0;
    // let err_mime = ErrorMime::JavaScript;
    // let code = try_get_file(&db, &session_id, err_mime, FileKind::JavaScript)?;

    // Ok(match compile(code) {
    //     Ok(js) => HttpResponse::Ok()
    //         .header("content-type", "application/javascript; charset=utf-8")
    //         .body(js),
    //     Err(err) => {
    //         println!("compile error: {:?}", err);
    //         println!("compile error root cause: {:?}", err.source());
    //         HttpError::js_compile_fail(err).to_response(err_mime)
    //     }
    // })
}

#[get("/session/{session_id}/page.js.raw")]
pub async fn r_get_session_page_js_raw(info: web::Path<String>) -> Result<HttpResponse, HttpError> {
    let err_mime = ErrorMime::JavaScript;
    let db = Db::open_env()
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;
    let session_id = info.0;
    let (_, code) = try_get_file(db, &session_id, err_mime, FileKind::JavaScript).await?;

    Ok(HttpResponse::Ok()
        .header("content-type", "application/javascript; charset=utf-8")
        .body(code))
}

#[get("/session/{session_id}/page.css")]
pub async fn r_get_session_page_css(info: web::Path<String>) -> Result<HttpResponse, HttpError> {
    let err_mime = ErrorMime::Css;
    let db = Db::open_env()
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;
    let session_id = info.0;
    let (_, code) = try_get_file(db, &session_id, err_mime, FileKind::Css).await?;

    Ok(HttpResponse::Ok()
        .header("content-type", "text/css; charset=utf-8")
        .body(code))
}

#[get("/session/{session_id}/page")]
pub async fn r_get_session_page_html(
    info: web::Path<String>,
    host: Host,
) -> Result<HttpResponse, HttpError> {
    let err_mime = ErrorMime::Html;
    let domain_frame = domain_frame();
    if !host.matches(&domain_frame) {
        return Err(HttpError::invalid_host(&domain_frame).with_mime(err_mime));
    }

    let session_id = info.0;
    let db = Db::open_env()
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;
    let (_, html) = try_get_file(db, &session_id, err_mime, FileKind::Html).await?;

    let parts = match parse_html(&html) {
        Ok(parts) => parts,
        Err(err) => return Err(HttpError::invalid_html(err).with_mime(err_mime)),
    };

    let page_url = |suffix: &str| format!("/api/session/{}/page{}", session_id, suffix);
    let public_path = |path: &str| format!("/dist/{}", path);
    let public_script = |path: &str| format!("<script src=\"{}\"></script>", public_path(path));

    // TODO: perform searches like https://api.cdnjs.com/libraries?search=jquery&limit=1 to allow arbitrary cdnjs deps
    let html = parts
        .into_iter()
        .try_fold(String::with_capacity(html.len()), |mut out, part| {
            match part {
                HtmlPart::Literal(literal) => out.push_str(literal),
                HtmlPart::IncludePath(path) => match &path[..] {
                    &["console"] => out.push_str(&public_script("console.bundle.js")),
                    &["editors", "js"] | &["editors", "js", "url"] => {
                        out.push_str(&page_url(".js"))
                    }
                    &["editors", "js", "raw"] | &["editors", "js", "raw", "url"] => {
                        out.push_str(&page_url(".js.raw"))
                    }
                    &["editors", "css"]
                    | &["editors", "css", "url"]
                    | &["editors", "css", "raw"]
                    | &["editors", "css", "url", "raw"] => out.push_str(&page_url(".css")),
                    &["deps", "react"] => {
                        out.push_str(&cdnjs_script("react/17.0.2/umd/react.development.min.js"));
                        out.push_str(&cdnjs_script(
                            "react-dom/17.0.2/umd/react-dom.development.min.js",
                        ));
                    }
                    &["deps", "jquery"] => {
                        out.push_str(&cdnjs_script("jquery/3.6.0/jquery.min.js"));
                    }
                    &["editors", other, ..] => {
                        anyhow::bail!("Unexpected second segment in inject(urls.{})", other)
                    }
                    &[other, ..] => anyhow::bail!("Unexpected command: inject!({}, …)", other),
                    &[] => anyhow::bail!("Unexpected empty inject!()"),
                },
            }

            Ok(out)
        });

    match html {
        Ok(html) => Ok(HttpResponse::Ok()
            // Based on jsfiddle's result frame http response
            .header("content-type", "text/html; charset=utf-8")
            .header("cache-control", "max-age=0, private, must-revalidate")
            .header("referrer-policy", "strict-origin-when-cross-origin")
            // Other maybe useful headers from that response:
            // x-frame-options: ALLOWALL
            // x-xss-protection: 0
            // x-content-type-options: nosniff
            // x-download-options: noopen
            // x-permitted-cross-domain-policies: none
            // set-cookie: csrftoken={long string}; path=/
            // vary: Origin
            // X-Firefox-Spdy: h2
            .body(html)),
        Err(err) => Err(HttpError::generate_html_fail(err).with_mime(err_mime)),
    }
}
