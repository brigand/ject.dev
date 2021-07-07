use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use swc_common::errors::emitter::Emitter;
use swc_common::errors::ColorConfig;
use swc_common::errors::Handler;
use swc_common::FileName;
use swc_common::SourceMap;
use swc_ecma_parser::EsConfig;
use swc_ecma_parser::Syntax;

use swc_common::{
    chain,
    comments::{Comment, Comments},
    errors::Handler,
    input::StringInput,
    BytePos, FileName, Globals, SourceFile, SourceMap, Spanned, GLOBALS,
};
use swc_ecma_ast::Program;
use swc_ecma_codegen::{self, Emitter, Node};
use swc_ecma_loader::resolvers::{lru::CachingResolver, node::NodeResolver, tsc::TsConfigResolver};
use swc_ecma_parser::{lexer::Lexer, Parser, Syntax};
use swc_ecma_transforms::{
    helpers::{self, Helpers},
    modules::path::NodeImportResolver,
    pass::noop,
};
// use swc_ecma_visit::FoldWith;

struct VoidEmit;

impl Emitter for VoidEmit {
    fn emit(&mut self, _db: &swc_common::errors::DiagnosticBuilder<'_>) {}
}

pub struct Compiler {
    /// swc uses rustc's span interning.
    ///
    /// The `Globals` struct contains span interner.
    globals: Globals,
    /// CodeMap
    pub cm: Arc<SourceMap>,
    pub handler: Arc<Handler>,
    comments: SwcComments,
}

#[derive(Debug, Serialize)]
pub struct TransformOutput {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<String>,
}

impl Compiler {
    /// Runs `op` in current compiler's context.
    ///
    /// Note: Other methods of `Compiler` already uses this internally.
    pub fn run<R, F>(&self, op: F) -> R
    where
        F: FnOnce() -> R,
    {
        GLOBALS.set(&self.globals, || op())
    }

    /// `custom_after_pass` is applied after swc transforms are applied.
    pub fn process_js_with_custom_pass<P>(
        &self,
        fm: Arc<SourceFile>,
        opts: &Options,
        custom_after_pass: P,
    ) -> Result<TransformOutput, Error>
    where
        P: swc_ecma_visit::Fold,
    {
        self.run(|| -> Result<_, Error> {
            let config = self.run(|| self.config_for_file(opts, &fm.name))?;
            let config = match config {
                Some(v) => v,
                None => {
                    bail!("cannot process file because it's ignored by .swcrc")
                }
            };
            let config = BuiltConfig {
                pass: chain!(config.pass, custom_after_pass),
                syntax: config.syntax,
                target: config.target,
                minify: config.minify,
                external_helpers: config.external_helpers,
                source_maps: config.source_maps,
                input_source_map: config.input_source_map,
                is_module: config.is_module,
            };
            let orig = self.get_orig_src_map(&fm, &opts.config.input_source_map)?;
            let program = self.parse_js(
                fm.clone(),
                config.target,
                config.syntax,
                config.is_module,
                true,
            )?;

            self.process_js_inner(program, orig.as_ref(), config)
        })
        .context("failed to process js file")
    }

    pub fn process_js_file(
        &self,
        fm: Arc<SourceFile>,
        opts: &Options,
    ) -> Result<TransformOutput, Error> {
        self.process_js_with_custom_pass(fm, opts, noop())
    }

    /// This method parses a javascript / typescript file
    pub fn parse_js(
        &self,
        fm: Arc<SourceFile>,
        target: JscTarget,
        syntax: Syntax,
        is_module: bool,
        parse_comments: bool,
    ) -> Result<Program, Error> {
        self.run(|| {
            let lexer = Lexer::new(
                syntax,
                target,
                StringInput::from(&*fm),
                if parse_comments {
                    Some(&self.comments)
                } else {
                    None
                },
            );
            let mut parser = Parser::new_from(lexer);
            let mut error = false;
            let program = if is_module {
                let m = parser.parse_module();

                for e in parser.take_errors() {
                    e.into_diagnostic(&self.handler).emit();
                    error = true;
                }

                m.map_err(|e| {
                    e.into_diagnostic(&self.handler).emit();
                    Error::msg("failed to parse module")
                })
                .map(Program::Module)?
            } else {
                let s = parser.parse_script();

                for e in parser.take_errors() {
                    e.into_diagnostic(&self.handler).emit();
                    error = true;
                }

                s.map_err(|e| {
                    e.into_diagnostic(&self.handler).emit();
                    Error::msg("failed to parse module")
                })
                .map(Program::Script)?
            };

            if error {
                bail!(
                    "failed to parse module: error was recoverable, but proceeding would result \
                     in wrong codegen"
                )
            }

            Ok(program)
        })
    }
}

pub fn compile(js: String) -> anyhow::Result<String> {
    let cm = Arc::<SourceMap>::default();
    let handler = Handler::with_emitter(false, false, Box::new(VoidEmit));
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

    let code = c.process_js_file(fm, &options)?.code;

    Ok(code)
}
