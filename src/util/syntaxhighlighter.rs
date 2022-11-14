use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::html::append_highlighted_html_for_styled_line;
use syntect::html::IncludeBackground::No;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub fn html_highlight(text: &str, extension: &str) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps
        .find_syntax_by_extension(extension)
        .or_else(|| Option::from(ps.find_syntax_plain_text()))
        .unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["InspiredGitHub"]);

    let mut highlighted_content: String = String::from("");

    for line in LinesWithEndings::from(text) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        append_highlighted_html_for_styled_line(&ranges[..], No, &mut highlighted_content)
            .expect("Failed to append highlighted line!");
    }

    let mut highlighted_content2: String = String::from("");
    for line in highlighted_content.lines() {
        highlighted_content2 += &*format!("<code-line>{}</code-line>\n", line);
    }

    // Rewrite colours to ones that are compatible with water.css and both light/dark modes
    highlighted_content2 = highlighted_content2.replace("style=\"color:#323232;\"", "");
    highlighted_content2 =
        highlighted_content2.replace("style=\"color:#183691;\"", "style=\"color:blue;\"");

    highlighted_content2
}
