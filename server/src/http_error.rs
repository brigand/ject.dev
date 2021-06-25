use actix_web::{http::StatusCode, HttpResponse};
use serde_json::{json, to_string};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorMime {
    Json,
    Html,
    JavaScript,
    Css,
}

pub struct HttpError<T, M, C> {
    /// Error title, e.g. <h1> text content
    pub title: T,
    /// Error message, e.g. <p> text content
    pub message: M,
    /// Http error status
    pub status: StatusCode,
    /// Error code, for use in code and bug reports
    pub code: C,
}

impl HttpError<&'static str, &'static str, &'static str> {
    pub fn session_not_found(mime: ErrorMime) -> Self {
        Self {
            title: "Unknown session",
            message: "Please try reloading the page",
            code: "inject_session_not_found",
            // Not sending error as it causes the asset to not be used
            // An error response on CSS/JS resources will cause it to display strangely
            status: if mime == ErrorMime::Html {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::OK
            },
        }
    }

    pub fn file_not_found(mime: ErrorMime) -> Self {
        Self {
            title: match mime {
                ErrorMime::Json => "No JSON file found",
                ErrorMime::Html => "No HTML file found",
                ErrorMime::JavaScript => "No JavaScript file found",
                ErrorMime::Css => "No CSS file found",
            },
            message: "The session appears to be missing an essential file",
            code: "inject_missing_file",
            // An error response on CSS/JS resources will cause it to display strangely
            status: if mime == ErrorMime::Html {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::OK
            },
        }
    }
}

impl HttpError<&'static str, String, &'static str> {
    pub fn invalid_html(error: impl Display) -> Self {
        Self {
            title: "Invalid HTML Provided",
            message: format!("Invalid HTML Provided\n\nReason:\n{}", error),
            code: "inject_invalid_html",
            status: StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    pub fn generate_html_fail(error: impl Display) -> Self {
        Self {
            title: "Unable to Generate HTML",
            message: format!(
                "Error encountered in HTML transform/generation.\n\nReason:\n{}",
                error
            ),
            code: "inject_failed_html_generation",
            status: StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

impl HttpError {
    pub fn to_response(&self, mime: ErrorMime) -> HttpResponse {
        let get_json = || {
            to_string(&json!({
                "title": &self.title,
                "message": &self.message,
                "code": &self.code,
            }))
            .unwrap()
            .replace("</", "\\x3c/")
        };

        HttpResponse::build(self.status)
            .header(
                "content-type",
                match mime {
                    ErrorMime::Json => "application/json; charset=UTF-8",
                    ErrorMime::Html => "text/html; charset=UTF-8",
                    ErrorMime::JavaScript => "application/javascript; charset=UTF-8",
                    ErrorMime::Css => "text/css; charset=UTF-8",
                },
            )
            .body(match mime {
                ErrorMime::Json => get_json(),
                ErrorMime::Html => indoc::formatdoc!(
                    r#"
                    <!DOCTYPE html>
                    <html>
                        <head>
                            <meta charset="utf-8" />
                        </head>

                        <body>
                            <h1>{title}</h1>
                            <h3>Code: {code}</h3>
                            <p>{message}</p>
                        </body>
                    </html>
                    "#,
                    title = html_escape::encode_text(&self.title),
                    message = html_escape::encode_text(&self.message),
                    code = html_escape::encode_text(&self.code)
                ),
                ErrorMime::JavaScript => format!(
                    "{pre}{json};\n {code}\n{close}",
                    json = get_json(),
                    pre = "{ let error_json = ",
                    close = "}",
                    code = indoc::indoc!(r#"
                        document.body.appendChild(Object.assign(document.createElement('h1'), { textContent: error_json.title }));
                        document.body.appendChild(Object.assign(document.createElement('h3'), { textContent: 'resource type: JavaScript, code: ' + error_json.code }));
                        document.body.appendChild(Object.assign(document.createElement('p'), { textContent: error_json.message }));
                        "#)),
                ErrorMime::Css => todo!(),
            })
    }
}
