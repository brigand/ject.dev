use anyhow::bail;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use swc::config::Config;
use swc::config::JscConfig;
use swc::config::JscTarget;
use swc::config::Options;
use swc::config::SourceMapsConfig;
use swc_common::errors::emitter::Emitter;
use swc_common::errors::emitter::EmitterWriter;
use swc_common::errors::Handler;
use swc_common::FileName;
use swc_common::SourceMap;
use swc_ecma_parser::EsConfig;
use swc_ecma_parser::Syntax;

struct VoidEmit;

impl Emitter for VoidEmit {
    fn emit(&mut self, db: &swc_common::errors::DiagnosticBuilder<'_>) {}

    fn should_show_explain(&self) -> bool {
        false
    }
}

#[derive(Debug, Default, Clone)]
struct MemWrite(Arc<Mutex<Vec<u8>>>);

impl Write for MemWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

impl MemWrite {
    fn take_buf(&self) -> Vec<u8> {
        std::mem::take(&mut *self.0.lock().unwrap())
    }
}

pub fn compile(js: String) -> anyhow::Result<String> {
    let cm = Arc::<SourceMap>::default();
    let write = MemWrite::default();
    let handler = Handler::with_emitter(
        false,
        false,
        Box::new(EmitterWriter::new(
            Box::new(write.clone()),
            None,
            false,
            true,
        )),
    );
    let c = swc::Compiler::new(cm.clone(), Arc::new(handler));

    let fm = cm.new_source_file(FileName::Custom("your-code.js".to_owned()), js);

    let options = Options {
        config: Config {
            source_maps: Some(SourceMapsConfig::Str("inline".into())),
            jsc: JscConfig {
                syntax: Some(Syntax::Es(EsConfig {
                    jsx: true,
                    optional_chaining: true,
                    nullish_coalescing: true,
                    num_sep: true,
                    ..Default::default()
                })),
                target: Some(JscTarget::Es2020),
                // transform: Some(TransformConfig { react:
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    match c.process_js_file(fm, &options) {
        Ok(output) => Ok(output.code),
        Err(err) => {
            let buf = write.take_buf();
            let s = String::from_utf8_lossy(&buf);
            bail!("[compile js] {}: {}", err, s);
        }
    }
}
