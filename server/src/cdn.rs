pub fn cdnjs_src(suffix: &str) -> String {
    let suffix = if suffix.starts_with("/") {
        &suffix[1..]
    } else {
        suffix
    };
    format!("https://cdnjs.cloudflare.com/ajax/libs/{}", suffix)
}

pub fn cdnjs_script(suffix: &str) -> String {
    format!(
        "<script src=\"{}\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>",
        cdnjs_src(suffix)
    )
}
