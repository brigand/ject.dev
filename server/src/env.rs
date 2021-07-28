use anyhow::Result;
use rusqlite::Connection;

pub fn is_production() -> bool {
    !std::env::var("JECT_IS_PROD").unwrap_or_default().is_empty()
}

pub fn domain_main() -> String {
    if let Some(domain) = std::env::var("JECT_DOMAIN_MAIN").ok() {
        domain
    } else if is_production() {
        "ject.dev".to_owned()
    } else {
        "ject.dev.local".to_owned()
    }
}

pub fn domain_frame() -> String {
    if let Some(domain) = std::env::var("JECT_DOMAIN_FRAME").ok() {
        domain
    } else if is_production() {
        "ject.link".to_owned()
    } else {
        "ject.link.local".to_owned()
    }
}

pub fn open_sqlite_env() -> Result<Connection, rusqlite::Error> {
    let mut path = match std::env::var("JECT_DB") {
        Ok(v) if !v.is_empty() => v,
        _ => "ject.db3".into(),
    };

    if !path.ends_with(".db3") {
        path.push_str(".db3");
    }

    Connection::open(path)
}

// fn open_rocksdb_path(path: &str) -> Result<DB> {
//     let mut opts = Options::default();
//     opts.create_if_missing(true);
//     opts.increase_parallelism(3);
//     opts.create_missing_column_families(true);
//     DB::open(&opts, path).or_else(|err| {
//         Err(err).context(format!(
//             "[inject] Failed to open database with path {} relative to {:?}",
//             path,
//             std::env::current_dir().expect("[inject] must be able to get CWD")
//         ))
//     })
// }
