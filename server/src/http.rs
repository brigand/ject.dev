use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::future::{ready, Ready};

fn get_header(req: &HttpRequest, header: &str) -> Option<String> {
    req.headers()
        .get(header)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.to_owned())
}

#[derive(Debug, Clone)]
pub struct Host {
    normal: Option<String>,
    forwarded: Option<String>,
}

impl Host {
    pub fn normal(&self) -> Option<&str> {
        self.normal.as_deref()
    }

    pub fn forwarded(&self) -> Option<&str> {
        self.forwarded.as_deref()
    }

    pub fn matches(&self, domain: &str) -> bool {
        let host = self.normal().and_then(|h| h.split(':').next());
        match host {
            Some(host) => host.eq_ignore_ascii_case(domain),
            None => return false,
        }
    }
}

impl FromRequest for Host {
    type Error = ();

    type Future = Ready<Result<Self, Self::Error>>;

    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // println!("Headers: {:?}", req.headers());
        ready(Ok(Self {
            normal: get_header(req, "host"),
            forwarded: get_header(req, "x-forwarded-for"),
        }))
    }
}
