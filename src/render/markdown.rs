use std::str::FromStr;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use syntect::{html::ClassStyle, parsing::SyntaxSet, util::LinesWithEndings};

#[derive(thiserror::Error, Debug)]
pub enum RenderError {
    #[error("HTML tags are unsupported in Markdown")]
    UnsupportedHtml,
}

type RenderResult<T> = Result<T, RenderError>;

// `Options::ENABLE_TABLES | Options::ENABLE_SMART_PUNCTUATION | ... ` is not const
const CMARK_OPTIONS: Options = Options::from_bits_truncate(
    (1 << 1) // Options::ENABLE_TABLES
    | (1 << 5) // Options::ENABLE_SMART_PUNCTUATION
    | (1 << 3) // Options::ENABLE_STRIKETHROUGH
    | (1 << 10), // Options::ENABLE_MATH
);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[repr(transparent)]
#[serde(transparent)]
pub struct MarkdownRenderable(String);

impl AsRef<str> for MarkdownRenderable {
    fn as_ref(&self) -> &str {
        self.raw()
    }
}

impl From<String> for MarkdownRenderable {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for MarkdownRenderable {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl FromStr for MarkdownRenderable {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl MarkdownRenderable {
    pub fn from_raw(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub const fn raw(&self) -> &str {
        self.0.as_str()
    }

    /// Renders the given string into HTML
    ///
    /// This uses typst to fill in the maths blocks.
    pub fn html(&self) -> RenderResult<String> {
        let parser = Parser::new_ext(self.raw(), CMARK_OPTIONS);
        let mut current_code = None;
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let parser = parser.flat_map(|event| -> Box<dyn Iterator<Item = Event>> {
            match event {
                pulldown_cmark::Event::InlineMath(s) => {
                    let html =
                        latex2mathml::latex_to_mathml(&s, latex2mathml::DisplayStyle::Inline)
                            .unwrap();
                    Box::new(std::iter::once(Event::Html(html.into())))
                }
                pulldown_cmark::Event::DisplayMath(s) => {
                    let html = latex2mathml::latex_to_mathml(&s, latex2mathml::DisplayStyle::Block)
                        .unwrap();
                    Box::new(std::iter::once(Event::Html(html.into())))
                }
                pulldown_cmark::Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        CodeBlockKind::Indented => String::new(),
                        CodeBlockKind::Fenced(cow_str) => cow_str.to_string(),
                    };

                    let syntax = syntax_set
                        .find_syntax_by_name(&lang)
                        .or_else(|| syntax_set.find_syntax_by_extension(&lang))
                        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                    current_code = Some(syntect::html::ClassedHTMLGenerator::new_with_class_style(
                        syntax,
                        &syntax_set,
                        ClassStyle::Spaced,
                    ));
                    Box::new(std::iter::empty())
                }
                pulldown_cmark::Event::Text(t) => {
                    if let Some(ref mut code) = current_code {
                        for line in LinesWithEndings::from(&t) {
                            code.parse_html_for_line_which_includes_newline(line)
                                .unwrap();
                        }
                        Box::new(std::iter::empty())
                    } else {
                        Box::new(std::iter::once(Event::Text(t)))
                    }
                }
                pulldown_cmark::Event::End(TagEnd::CodeBlock) => {
                    let code = current_code.take().expect("Can't have end without start");
                    let out = code.finalize();
                    Box::new(std::iter::once(Event::Html(
                        format!("<pre>{}</pre>", out).into(),
                    )))
                }
                e => Box::new(std::iter::once(e)),
            }
        });
        let mut s = String::new();
        pulldown_cmark::html::push_html(&mut s, parser);
        Ok(s)
    }
}
