use anyhow::bail;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::io::Write;
use std::iter::Peekable;
use std::sync::Arc;
use std::sync::Mutex;
// use swc::config::Config;
// use swc::config::JscConfig;
// use swc::config::JscTarget;
// use swc::config::Options;
// use swc::config::SourceMapsConfig;
// use swc::config::TransformConfig;
use swc_common::errors::emitter::EmitterWriter;
use swc_common::errors::Handler;
use swc_common::source_map::Pos;
use swc_common::BytePos;
use swc_common::FileName;
use swc_common::SourceMap;
use swc_common::Span;
use swc_common::Spanned;
use swc_common::SyntaxContext;
use swc_ecma_ast::JSXAttrValue;
use swc_ecma_ast::JSXElement;
use swc_ecma_ast::JSXElementName;
use swc_ecma_ast::Lit;
use swc_ecma_parser::error::Error as EcmaParserError;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::EsConfig;
use swc_ecma_parser::Parser;
use swc_ecma_parser::StringInput;
use swc_ecma_parser::Syntax;
use swc_ecma_parser::TsConfig;
use swc_ecma_visit::VisitAll;
use swc_ecma_visit::VisitAllWith;

// Ref: https://sourcemaps.info/spec.html
struct Vql(i32);

impl Display for Vql {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    ReplaceSpan(Span),
    Remove,
}

#[derive(Debug)]
struct VisitJsx<'a> {
    code: &'a str,
    ops: Vec<(Action, Span)>,
}

fn span_len(span: Span) -> usize {
    (span.hi().0 as usize) - (span.lo().0 as usize)
}

trait ToPos {
    fn to_byte_pos(self) -> BytePos;
}

impl ToPos for BytePos {
    fn to_byte_pos(self) -> BytePos {
        self
    }
}
impl ToPos for i32 {
    fn to_byte_pos(self) -> BytePos {
        BytePos::from_u32(
            u32::try_from(self).expect("ToPos for i32 expected number to be positive"),
        )
    }
}
impl ToPos for u32 {
    fn to_byte_pos(self) -> BytePos {
        BytePos::from_u32(self)
    }
}
impl ToPos for usize {
    fn to_byte_pos(self) -> BytePos {
        BytePos::from_usize(self)
    }
}

fn make_span(lo: impl ToPos, hi: impl ToPos) -> Span {
    Span::new(lo.to_byte_pos(), hi.to_byte_pos(), Default::default())
}

// struct Consumer<'a> {
//     code: &'a str,
//     consumed: &'a mut Vec<Span>,
// }

// impl Consumer<'_> {
//     fn write_consume(&mut self, span: Span, out: &mut String) {
//         let mut offset = 0;
//         let start = span.lo().to_usize();
//         let end = span.hi().to_usize();
//         consumed.sort();

//         let omit = self.consumed.iter().copied().filter(|con| con.contains(span) || con.lo() > span.lo() && con.lo() < span.hi() || con.hi() > span.lo()  && con.hi() < span.hi()).collect();
// for c in  {
//     if c.contains(span) {
//         return;
//     } else if span.lo() >

// }
//     }
// }

impl<'a> VisitJsx<'a> {
    fn slice(&self, span: Span) -> &str {
        &self.code[span.lo().0 as usize..span.hi().0 as usize]
    }

    // fn write_and_consume(out: &mut String, code: &str, span: Span, consumed: &mut Vec<Span>) {
    //     &self.code[..]
    // }

    fn sort_by_spans(&mut self) {
        self.ops.sort_by_key(|x| x.1.hi);
        self.ops.sort_by_key(|x| x.1.lo);
    }
    fn apply(mut self) -> String {
        self.sort_by_spans();

        let range = make_span(0, self.code.len());
        dbg!(&self.ops);
        let mut ops = self.ops.into_iter().peekable();
        let mut out = String::with_capacity(self.code.len());
        let mut consumed = BytePos::from_u32(0);
        Self::apply_inner(&mut out, self.code, &mut consumed, range, &mut ops);
        out
    }

    fn apply_inner(
        out: &mut String,
        code: &str,
        consumed: &mut BytePos,
        local_range: Span,
        ops: &mut Peekable<impl Iterator<Item = (Action, Span)>>,
    ) {
        let mut last_span = make_span(local_range.lo, local_range.lo);

        while let Some((_action, span)) = ops.peek() {
            if last_span.hi <= span.lo {
                let a = std::cmp::max(last_span.hi(), *consumed).to_usize();
                let b = std::cmp::max(span.lo(), *consumed).to_usize();

                out.push_str(&code[a..b]);
                *consumed = std::cmp::max(span.hi(), *consumed);
                last_span = *span;
            }

            if !local_range.contains(*span) {
                return;
            }

            let (action, span) = ops.next().unwrap();

            match action {
                Action::Replace(new_text) => {
                    out.push_str(&new_text);
                }
                Action::ReplaceSpan(src_span) => {
                    // Replace `span` with `src_span` after evaluating all other ops inside `span`
                    // This needs to recurse to potentially any depth
                    // Maybe we make another VisitJsx with the `span` subset of `self.code`
                    // and the ops inside `span`, with an extra span offset field for `span.0.lo()`

                    let mut last = make_span(src_span.lo, src_span.lo);
                    while let Some((_action2, span2)) = ops.peek() {
                        if span.contains(*span2) {
                            let local = make_span(last.lo, span2.hi);
                            Self::apply_inner(out, code, consumed, local, ops);
                            last = local;
                        } else {
                            break;
                        }
                    }
                }
                Action::Remove => {
                    // Do nothing
                }
            }
        }

        let tail_start = last_span.hi.to_usize();
        let tail_end = code.len();
        out.push_str(&code[tail_start..tail_end]);
    }
}

impl VisitAll for VisitJsx<'_> {
    fn visit_jsx_element(&mut self, node: &JSXElement, parent: &dyn swc_ecma_visit::Node) {
        let (name_span, is_html) = match &node.opening.name {
            JSXElementName::Ident(ident) => {
                let first_char = std::str::from_utf8(ident.sym.as_bytes())
                    .ok()
                    .and_then(|s| s.chars().next());

                let is_html = match first_char {
                    Some(ch) => ch.is_lowercase(),
                    None => true,
                };
                (ident.span(), is_html)
            }
            JSXElementName::JSXMemberExpr(mem) => (mem.span(), false),
            JSXElementName::JSXNamespacedName(ns) => (ns.span(), false),
        };

        let prefix_span = make_span(node.opening.span().lo, name_span.hi);

        let has_props = !node.opening.attrs.is_empty();
        let name_replace = format!(
            "React.createElement({q}{name}{q}, {{",
            q = if is_html { "\"" } else { "" },
            name = self.slice(name_span)
        );
        self.ops.push((Action::Replace(name_replace), prefix_span));

        let mut last_attr_span = None;
        for attr in &node.opening.attrs {
            use swc_ecma_ast::JSXAttrOrSpread;
            last_attr_span = Some(attr.span());
            match attr {
                JSXAttrOrSpread::JSXAttr(attr) => {
                    let span = match &attr.name {
                        swc_ecma_ast::JSXAttrName::Ident(ident) => ident.span(),
                        swc_ecma_ast::JSXAttrName::JSXNamespacedName(ns) => ns.span(),
                    };
                    let mut name_and_punct_span = attr.name.span();
                    name_and_punct_span.hi = attr.value.span().lo;
                    self.ops.push((
                        Action::Replace(format!("\"{}\": ", self.slice(span))),
                        name_and_punct_span,
                    ));

                    match &attr.value {
                        Some(JSXAttrValue::Lit(lit)) => {
                            self.ops
                                .push((Action::ReplaceSpan(lit.span()), attr.value.span()));
                        }
                        Some(JSXAttrValue::JSXExprContainer(jsx_expr)) => {
                            let value_with_curlies_span =
                                attr.span().trim_start(name_and_punct_span).expect("[inject] attr span expected to be prefixed by name_and_punct_span (for Span::trim_start)");

                            let expr_span = jsx_expr.expr.span();
                            self.ops
                                .push((Action::ReplaceSpan(expr_span), value_with_curlies_span));
                        }
                        Some(JSXAttrValue::JSXElement(el)) => {
                            let span = el.span();
                            self.ops
                                .push((Action::ReplaceSpan(span), attr.value.span()));
                        }
                        Some(JSXAttrValue::JSXFragment(frag)) => {
                            let span = frag.span();
                            self.ops
                                .push((Action::ReplaceSpan(span), attr.value.span()));
                        }
                        None => {
                            self.ops
                                .push((Action::Replace("true".to_owned()), attr.value.span()));
                        }
                    }
                }
                JSXAttrOrSpread::SpreadElement(spread) => {
                    let mut start = spread.span();
                    start.hi = start.lo;
                    self.ops.push((Action::Replace("...".to_owned()), start));
                    self.ops
                        .push((Action::ReplaceSpan(spread.expr.span()), spread.span()));
                }
            }
        }

        let closing_span = if let Some(closing) = &node.closing {
            closing.span
        } else if let Some(last_attr_span) = last_attr_span {
            make_span(last_attr_span.hi, node.opening.span().hi())
        } else {
            make_span(name_span.hi, node.opening.span().hi())
        };

        if node.opening.self_closing {
            let suffix = if has_props { " })" } else { "})" };
            self.ops
                .push((Action::Replace(suffix.to_owned()), closing_span));
        } else {
            let suffix = if has_props { " }, " } else { "}, " };
            self.ops.push((
                Action::Replace(suffix.to_owned()),
                node.opening.span().shrink_to_hi(),
            ));
        }
    }
}

#[cfg(test)]
mod visit_test {
    use super::compile_minimal;

    //     #[test]
    //     fn first() {
    //         let code = r#"
    // console.log('Hello, world!')
    // function App() {
    //     const [count, setCount] = React.useState(0)
    //     console.log({ count })
    //     return <button type="button" onClick={() => setCount(c => c + 1)}>Count: {count}</button>;
    // }

    // ReactDOM.render(<App />, root)
    // "#;

    //         let out = compile_minimal(code.to_owned()).unwrap();
    //     }

    #[test]
    fn simple_jsx() {
        let code = r#"const el = <aaaa />;"#;

        let out = compile_minimal(code.to_owned()).unwrap();
        assert_eq!(out, r#"const el = React.createElement("aaaa", {});"#);
    }

    #[test]
    fn nested_jsx_in_prop() {
        let code = r#"const el = <aaaa bbbb={<cccc />} />;"#;

        let out = compile_minimal(code.to_owned()).unwrap();
        assert_eq!(
            out,
            r#"const el = React.createElement("aaaa", { "bbbb": React.createElement("cccc", {}) });"#
        );
    }

    #[test]
    fn nested_jsx_in_prop_2() {
        let code = r#"const el = <aaaa bbbb={<cccc dddd={<eeee />} />} />;"#;

        let out = compile_minimal(code.to_owned()).unwrap();
        assert_eq!(
            out,
            r#"const el = React.createElement("aaaa", { "bbbb": React.createElement("cccc", {"dddd": React.createElement("eeee", {}) }) });"#
        );
    }

    // #[test]
    // fn nested_jsx_in_prop_3() {
    //     let code = r#"const el = <aaaa bbbb={<cccc dddd={<eeee ffff={<gggg />} />} />} />;"#;
    //     let out = compile_minimal(code.to_owned()).unwrap();
    // }
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

    let fm = cm.new_source_file(FileName::Custom("your-code.mjs".to_owned()), js.clone());

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
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parse module");

    let mut visitor = VisitJsx {
        code: &js,
        ops: vec![],
    };
    for item in &module.body {
        item.visit_all_with(&module, &mut visitor);
    }

    // println!("Visitor: {:?}", visitor);
    let out = visitor.apply();
    println!("Out: {}", out);

    Ok(out)
}

// pub fn compile(js: String) -> anyhow::Result<String> {
//     let cm = Arc::<SourceMap>::default();
//     let write = MemWrite::default();
//     let handler = make_handler(write.clone());
//     let c = swc::Compiler::new(cm.clone(), Arc::new(handler));

//     let fm = cm.new_source_file(FileName::Custom("your-code.mjs".to_owned()), js);

//     let options = Options {
//         config: Config {
//             source_maps: Some(SourceMapsConfig::Str("inline".into())),
//             // module: Some(swc::config::ModuleConfig::Es6),
//             jsc: JscConfig {
//                 // syntax: Some(Syntax::Typescript(TsConfig {
//                 //     tsx: true,
//                 //     dynamic_import: true,
//                 //     ..Default::default()
//                 // })),
//                 syntax: Some(Syntax::Es(EsConfig {
//                     jsx: true,
//                     // optional_chaining: true,
//                     // nullish_coalescing: true,
//                     // num_sep: true,
//                     ..Default::default()
//                 })),
//                 target: Some(JscTarget::Es2020),
//                 transform: Some(TransformConfig {
//                     react: Default::default(),
//                     ..Default::default()
//                 }),
//                 ..Default::default()
//             },
//             ..Default::default()
//         },
//         ..Default::default()
//     };

//     match c.process_js_file(fm, &options) {
//         Ok(output) => Ok(output.code),
//         Err(err) => {
//             let buf = write.take_buf();
//             let s = String::from_utf8_lossy(&buf);
//             bail!("[compile js] {}: {}", err, s);
//         }
//     }
// }
