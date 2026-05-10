use crate::utils::markdown;

pub fn render_summary_or_excerpt(summary: Option<&str>, content_md: &str) -> Option<String> {
    if let Some(s) = summary {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let rendered = markdown::render(content_md);
    let excerpt: String = strip_html_tags(&rendered)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(180)
        .collect();
    if excerpt.is_empty() {
        None
    } else {
        Some(excerpt)
    }
}

pub fn render_html(content_md: &str) -> String {
    // Admin-authored Markdown is trusted in M2. Revisit sanitization before adding multi-author roles.
    markdown::render(content_md)
}

fn strip_html_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}
