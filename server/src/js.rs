use anyhow::bail;
use std::fmt;
use std::fmt::Display;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use swc::config::Config;
use swc::config::JscConfig;
use swc::config::JscTarget;
use swc::config::Options;
use swc::config::SourceMapsConfig;
use swc::config::TransformConfig;
use swc_common::errors::emitter::EmitterWriter;
use swc_common::errors::Handler;
use swc_common::FileName;
use swc_common::SourceMap;
use swc_common::Span;
use swc_common::Spanned;
use swc_ecma_ast::JSXElement;
use swc_ecma_ast::JSXElementName;
use swc_ecma_parser::error::Error as EcmaParserError;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::EsConfig;
use swc_ecma_parser::Parser;
use swc_ecma_parser::StringInput;
use swc_ecma_parser::Syntax;
use swc_ecma_parser::TsConfig;
use swc_ecma_visit::VisitAll;

struct Vql(i32);

impl Display for Vql {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;

        static ALPHA: [char; 64] = [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'O', 'N', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
            'i', 'j', 'k', 'l', 'm', 'o', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
            'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/',
        ];

        let negative = self.0 < 0;
        let sign_bit = if negative { 1 } else { 0 };
        let n = if negative { self.0 * -1 } else { self.0 } as u32;
        let mut e = n >> 19 & 0b11111;
        let mut d = n >> 14 & 0b11111;
        let mut c = n >> 9 & 0b11111;
        let mut b = n >> 4 & 0b11111;
        let mut a = ((n & 0b1111) << 1) | sign_bit;

        let mut carry = |x: &mut u32, y| {
            if y > 0 {
                *x |= 0b10000;
            }
        };
        carry(&mut d, e);
        carry(&mut c, d);
        carry(&mut b, c);
        carry(&mut a, b);

        let ch = |n| ALPHA[n as usize];

        if e > 0 {
            write!(f, "{}{}{}{}{}", ch(a), ch(b), ch(c), ch(d), ch(e),)
        } else if d > 0 {
            write!(f, "{}{}{}{}", ch(a), ch(b), ch(c), ch(d))
        } else if c > 0 {
            write!(f, "{}{}{}", ch(a), ch(b), ch(c))
        } else if b > 0 {
            write!(f, "{}{}", ch(a), ch(b))
        } else {
            write!(f, "{}", ch(a))
        }
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

fn make_handler(write: MemWrite) -> Handler {
    Handler::with_emitter(
        false,
        false,
        Box::new(EmitterWriter::new(Box::new(write), None, false, true)),
    )
}

#[derive(Debug, Clone)]
enum Action {
    Replace(String),
    Remove,
}

#[derive(Debug, Default)]
struct VisitJsx {
    ops: Vec<(Action, Span)>,
}

// Ref: https://sourcemaps.info/spec.html

impl VisitAll for VisitJsx {
    fn visit_jsx_element(&mut self, node: &JSXElement, parent: &dyn swc_ecma_visit::Node) {
        match node.opening.name {
            JSXElementName::Ident(ident) => {}
            JSXElementName::JSXMemberExpr(member) => {}
            JSXElementName::JSXNamespacedName(namespaced) => {}
        }
        if let Some(closing) = &node.closing {
            self.ops.push((Action::Remove, closing.span));
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JsError {
    #[error("A syntax error was encountered when parsing th file")]
    Parse {
        non_std_error_source: EcmaParserError,
    },
}

pub struct Position {
    pub line: usize,
    pub char: usize,
}
struct CodeSpan {
    pub start: Position,
    pub end: Position,
}

fn span_str(code: &str, spanned: &impl Spanned) -> Option<CodeSpan> {
    let span = spanned.span();
    let lo = span.lo().0 as usize;
    let hi = span.hi().0 as usize;

    let mut start = None;
    let mut end = None;
    let mut line = 1;
    for (i, ch) in code.char_indices() {
        if start.is_none() {
            if i >= lo {
                start = Some(Position { line, char: i });
            }
        } else if i >= hi {
            end = Some(Position { line, char: i });
            break;
        }

        if ch == '\n' {
            line += 1;
        }
    }

    if let (Some(start), Some(end)) = (start, end) {
        Some(CodeSpan { start, end })
    } else {
        None
    }
}

impl JsError {
    pub fn from_parse(err: EcmaParserError) -> Self {
        Self::Parse {
            non_std_error_source: err,
        }
    }
}

pub fn compile_minimal(js: String) -> Result<String, JsError> {
    let cm = Arc::<SourceMap>::default();
    let write = MemWrite::default();
    let handler = make_handler(write.clone());

    let fm = cm.new_source_file(FileName::Custom("your-code.mjs".to_owned()), js);

    let lexer = Lexer::new(
        Syntax::Es(EsConfig {
            jsx: true,
            optional_chaining: true,
            nullish_coalescing: true,
            num_sep: true,
            ..Default::default()
        }),
        JscTarget::Es2020,
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    let errors = parser.take_errors();
    if !errors.is_empty() {
        for e in errors {
            e.into_diagnostic(&handler).emit();
        }
    }

    let module = parser
        .parse_module()
        .map_err(|mut e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parser module");
    module.body
}

pub fn compile(js: String) -> anyhow::Result<String> {
    let cm = Arc::<SourceMap>::default();
    let write = MemWrite::default();
    let handler = make_handler(write.clone());
    let c = swc::Compiler::new(cm.clone(), Arc::new(handler));

    let fm = cm.new_source_file(FileName::Custom("your-code.mjs".to_owned()), js);

    let options = Options {
        config: Config {
            source_maps: Some(SourceMapsConfig::Str("inline".into())),
            // module: Some(swc::config::ModuleConfig::Es6),
            jsc: JscConfig {
                // syntax: Some(Syntax::Typescript(TsConfig {
                //     tsx: true,
                //     dynamic_import: true,
                //     ..Default::default()
                // })),
                syntax: Some(Syntax::Es(EsConfig {
                    jsx: true,
                    // optional_chaining: true,
                    // nullish_coalescing: true,
                    // num_sep: true,
                    ..Default::default()
                })),
                target: Some(JscTarget::Es2020),
                transform: Some(TransformConfig {
                    react: Default::default(),
                    ..Default::default()
                }),
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
