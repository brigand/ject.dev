use anyhow::Context;
use anyhow::Result;
use rocksdb::{Options, DB};

pub fn open_rocksdb_env() -> Result<DB> {
    let data_dir = match std::env::var("IJ_DATA_DIR") {
        Ok(v) if !v.is_empty() => v,
        _ => "ij_data_dir".into(),
    };

    open_rocksdb_path(&data_dir)
}

fn open_rocksdb_path(path: &str) -> Result<DB> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.increase_parallelism(3);
    opts.create_missing_column_families(true);
    DB::open(&opts, path).or_else(|err| {
        Err(err).context(format!(
            "[inject] Failed to open database with path {} relative to {:?}",
            path,
            std::env::current_dir().expect("[inject] must be able to get CWD")
        ))
    })
}
