use actix_web::{http::StatusCode, HttpResponse};
use serde_json::{json, to_string};
use std::convert::TryInto;

pub enum ErrorMime {
    Json,
    Html,
    JavaScript,
    Css,
}

pub struct HttpError {
    /// Error title, e.g. <h1> text content
    pub title: String,
    /// Error message, e.g. <p> text content
    pub message: String,
    /// Http error status
    pub status: StatusCode,
    /// Error code, for use in code and bug reports
    pub code: String,
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
