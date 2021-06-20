#[derive(Debug, Clone)]
pub enum HtmlPart<'a> {
    Literal(&'a str),
    IncludePath(Vec<&'a str>),
}

static PRE_INJECT: &str = "inject!(";
static POST_INJECT: &str = ")";

pub fn parse_html(html_orig: &str) -> Result<Vec<HtmlPart>, anyhow::Error> {
    let mut parts = vec![];
    let mut wb = &html_orig[..];

    loop {
        if let Some(start) = wb.find(PRE_INJECT) {
            let (l, r) = wb.split_at(start);
            parts.push(HtmlPart::Literal(l));
            wb = r;

            let end = wb.find(POST_INJECT);
            let newline = wb.find('\n');

            match (end, newline) {
                (Some(end), Some(newline)) if newline < end => {
                    anyhow::bail!("inject! calls must be closed on the same line");
                }
                (None, _) => {
                    anyhow::bail!("inject! calls must be closed (on the same line)");
                }
                (Some(end), _) => {
                    let (l, r) = wb.split_at(end);
                    let contents = (&l[PRE_INJECT.len()..]).trim();
                    parts.push(HtmlPart::IncludePath(
                        contents.split('.').map(|segment| segment.trim()).collect(),
                    ));
                    wb = &r[POST_INJECT.len()..];
                }
            }
        } else {
            parts.push(HtmlPart::Literal(wb));
            break;
        }
    }

    Ok(parts)
}

#[cfg(test)]
mod tests {}
