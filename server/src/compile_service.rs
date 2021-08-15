use actix_web::client::Client;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonError {
    err_id: String,
    message: String,
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Failed to POST to /api/babel on the compiler service")]
    CompileHttp { source: actix_web::Error },

    #[error("Failed to POST to /api/babel on the compiler service")]
    Compile { err_id: String, message: String },
}

impl CompileError {
    fn compile_http(source: impl Into<actix_web::Error>) -> Self {
        Self::CompileHttp {
            source: source.into(),
        }
    }
}

pub async fn babel_compile(code: &str) -> Result<String, CompileError> {
    let client = Client::default();
    let mut res = client
        .post("http://localhost:1951/api/babel")
        .send_json(&serde_json::json!({
            "code": code.to_owned()
        }))
        .await
        .map_err(CompileError::compile_http)?;
    if res.status().is_success() {
        let bytes = res.body().await.map_err(CompileError::compile_http)?;

        Ok(String::from_utf8_lossy(bytes.as_ref()).to_string())
    } else {
        let body: JsonError = res.json().await.map_err(CompileError::compile_http)?;
        Err(CompileError::Compile {
            err_id: body.err_id,
            message: body.message,
        })
    }
}
