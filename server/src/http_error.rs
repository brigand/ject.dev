use std::fmt::Display;

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

    pub fn js_compile_fail(error: impl Display) -> Self {
        Self {
            title: "JS Parse/Compile Failed",
            message: format!("nReason:\n{}", error),
            code: "js_compile_fail",
            status: StatusCode::OK,
        }
    }
}

impl<T, M, C> HttpError<T, M, C>
where
    T: AsRef<str>,
    M: AsRef<str>,
    C: AsRef<str>,
{
    pub fn to_response(&self, mime: ErrorMime) -> HttpResponse {
        let title = self.title.as_ref();
        let message = self.message.as_ref();
        let code = self.code.as_ref();

        let get_json = || {
            to_string(&json!({
                "title": title.to_string(),
                "message": message.to_string(),
                "code": code.to_string(),
            }))
            .unwrap()
            .replace("</", "\\x3c/")
        };

        fn to_css_string(s: &str) -> String {
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for ch in s.chars() {
                // Ref: http://www.asciitable.com/
                // Ref: https://stackoverflow.com/a/9063069/1074592
                let end_of_control = 33;
                let del = 127;

                let num = u32::from(ch);
                if ch == ' ' {
                    out.push(ch);
                } else if num < end_of_control || ch == '"' || num == del {
                    use std::fmt::Write;
                    write!(&mut out, "\\{:06X}", num).unwrap();
                } else {
                    out.push(ch);
                }
            }

            out.push('"');

            out
        }

        let colors = indoc::indoc!(
            r#"
            :root {
                --fg: #d5ced9;
                --bg: #23262e;
                --red: #ee5d43;
                --purple: #c74ded;
                --yellow: #ffe66d;
                --font: Arial, sans;
            }
            "#
        );

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
                            <style>
                                {colors}
                                html {{
                                    font-family: var(--font);
                                    background: var(--bg);
                                    color: var(--fg);
                                }}
                                h1 {{ color: var(--red)}}
                                h3 span {{ color: var(--purple); }}
                                p {{ color: var(--yellow); white-space: pre-wrap; }}
                            </style>
                        </head>

                        <body>
                            <h1>{title}</h1>
                            <h3>Code: <span>{code}</span></h3>
                            <p>{message}</p>
                        </body>
                    </html>
                    "#,
                    title = html_escape::encode_text(title),
                    message = html_escape::encode_text(message),
                    code = html_escape::encode_text(code),
                    colors = colors
                ),
                ErrorMime::JavaScript => format!(
                    "{pre}{json};\nlet css = {css_json};\n{code}\n{close}",
                    json = get_json(),
                    pre = "{ let error_json = ",
                    close = "}",
                    code = indoc::indoc!(r#"
                        const h = (tag, {style, ...props}, ...children) => {
                            const element = Object.assign(document.createElement(tag), props);
                            Object.assign(element.style, style);
                            for (const child of children) {
                                if (!child || child === true) continue;
                                if (child instanceof Node) {
                                    element.appendChild(child);
                                } else {
                                    const text = document.createTextNode(String(child));
                                    element.appendChild(text);
                                }
                            }

                            return element;
                        };

                        document.head.appendChild(
                            h('style', {}, css),
                        );
                        document.body.appendChild(
                            h('aside', {},
                                h('h1', { style: { color: 'var(--red)' } }, error_json.title),
                                h('h3', {},
                                    'Code: ',
                                    h('span', { style: { color: 'var(--purple)' } }, error_json.code),
                                ),
                                h('p', { style: { color: 'var(--yellow)', whiteSpace: 'pre-wrap' } }, error_json.message),
                            ),
                        );
                        "#),
                        css_json = serde_json::to_string(colors).unwrap(),
                    ),
                ErrorMime::Css => indoc::formatdoc!(r#"
                    {colors}

                    html {{
                        background: var(--bg);
                    }}

                    html::before {{
                        content: {title};
                        display: block;
                        font-size: 2em;
                        margin: 1rem;
                        color: var(--red);
                        font-family: var(--font);
                    }}

                    body::before {{
                        content: {body};
                        display: block;
                        white-space: pre-wrap;
                        margin: 1rem;
                        color: var(--yellow);
                        font-family: var(--font);
                    }}
                "#,
                title = to_css_string(title),
                body = to_css_string(&format!("Code: {}\n\n{}", code, message)),
                colors = colors)
            })
    }
}
