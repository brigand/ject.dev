use std::sync::Arc;
use swc::config::Config;
use swc::config::JscConfig;
use swc::config::JscTarget;
use swc::config::SourceMapsConfig;
use swc::config::TransformConfig;
use swc_common::FileName;
use swc_common::errors::ColorConfig;
use swc_common::errors::Handler;
use swc_common::SourceMap;
use swc_ecma_parser::EsConfig;
use swc_ecma_parser::Syntax;
use std::path::Path;
use swc::config::Options;
use swc_common::errors::emitter::Emitter;

struct VoidEmit;

impl Emitter for VoidEmit {
    fn emit(&mut self, _db: &swc_common::errors::DiagnosticBuilder<'_>) {

    }
}

pub fn compile(js: String) -> anyhow::Result<String> {
    let cm = Arc::<SourceMap>::default();
    let handler = Handler::with_emitter(false,false,  Box::new(VoidEmit));
    let c = swc::Compiler::new(cm.clone(), Arc::new(handler));

    let fm = cm.new_source_file(FileName::Custom("your-code.js".to_owned()), js);

    let options = Options {
        config: Config {
            source_maps: Some(SourceMapsConfig::Str("inline".into())),
            jsc: JscConfig {
                syntax: Some(Syntax::Es(EsConfig{
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

    let code = c.process_js_file(
        fm,&options
    )?.code;

    Ok(code)
}
