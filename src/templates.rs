use std::{
    io::{self, Write},
    ops::Range,
};

use miette::{Diagnostic, SourceSpan};
use serde::Deserialize;
use thiserror::Error;

// https://users.rust-lang.org/t/what-is-the-best-way-to-iterate-over-lines-of-a-string-with-offset/120021
fn lines_with_offset(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.split('\n')
        .scan((0, ""), |(prev_end_offset, _), line| {
            let start_offset = *prev_end_offset;
            *prev_end_offset += '\n'.len_utf8() + line.len();
            Some((start_offset, line))
        })
        .map(|(start_offset, line)| match line.strip_prefix('\r') {
            Some(line) => (start_offset + '\r'.len_utf8(), line),
            None => (start_offset, line),
        })
        .map(|(start_offset, line)| (start_offset, line.strip_suffix('\r').unwrap_or(line)))
}

// This clone is free, have no shame in calling it
// I would really like to implement copy here, but that's not possible because of the `Range`s
#[derive(Clone, Debug)]
enum Order {
    TemplateSolution {
        infix: Range<usize>,
        solution: Range<usize>,
    },
    SolutionTemplate {
        solution: Range<usize>,
        infix: Range<usize>,
    },
    Template,
}

impl Order {
    fn solution(&self) -> Option<Range<usize>> {
        match self {
            Order::TemplateSolution { solution, .. } | Order::SolutionTemplate { solution, .. } => {
                Some(solution.clone())
            }
            Order::Template => None,
        }
    }

    /// Create the body that will fit between the prefix and suffix, handling indent correctly.
    fn body<'a>(
        &'_ self,
        full_template: &'a str,
        user_code: &'a str,
        indent: &'a str,
    ) -> impl Iterator<Item = &'a str> {
        let lines_with_indent =
            |s: &'a str, indent: &'a str| s.lines().flat_map(move |l| [indent, l, "\n"]);

        // The usage of std::iter::once("") here is to have each branch of the match be the same
        // type
        //
        // This ends up being something like
        // once(&str)
        //     .chain(lines_with_indent return value)
        //     .chain(&str)
        match self {
            Order::TemplateSolution { infix, solution: _ } => {
                std::iter::once(&full_template[infix.clone()])
                    .chain(lines_with_indent(user_code, indent))
                    .chain(std::iter::once(""))
            }
            Order::SolutionTemplate { solution: _, infix } => std::iter::once("")
                .chain(lines_with_indent(user_code, indent))
                .chain(std::iter::once(&full_template[infix.clone()])),
            Order::Template => std::iter::once("")
                .chain(lines_with_indent(user_code, indent))
                .chain(std::iter::once("")),
        }
    }
}

/// An iterator over string slices that can also be read from or converted into a [`String`].
pub struct StringIterator<'a, I: Iterator<Item = &'a str>> {
    peeked: Option<&'a str>,
    inner: I,
}

impl<'a, I: Iterator<Item = &'a str>> std::fmt::Debug for StringIterator<'a, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StringIterator")
            .field("peeked", &self.peeked)
            .field("inner", &std::any::type_name::<I>())
            .finish()
    }
}

impl<'a, I: Iterator<Item = &'a str>> StringIterator<'a, I> {
    fn new(inner: I) -> Self {
        Self {
            peeked: None,
            inner,
        }
    }

    fn peek_mut(&mut self) -> Option<&mut &'a str> {
        if let Some(ref mut peeked) = self.peeked {
            Some(peeked)
        } else {
            self.peeked = self.inner.next();
            self.peeked.as_mut()
        }
    }

    pub fn into_string(self) -> String {
        let mut out = String::new();
        for s in self {
            out.push_str(s);
        }
        out
    }
}

impl<'a, I: Iterator<Item = &'a str>> Iterator for StringIterator<'a, I> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(peeked) = self.peeked.take() {
            Some(peeked)
        } else {
            self.inner.next()
        }
    }
}

impl<'a, I: Iterator<Item = &'a str>> std::io::Read for StringIterator<'a, I> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let val = self.peek_mut();
        let bytes = if let Some(val) = val {
            // dbg!((buf.len(), val.len(), &val));
            match val.len().cmp(&buf.len()) {
                std::cmp::Ordering::Less => {
                    let len = val.len();
                    buf[..len].copy_from_slice(val.as_bytes());
                    self.next();
                    len + self.read(&mut buf[len..])?
                }
                std::cmp::Ordering::Equal => {
                    buf.copy_from_slice(&val.as_bytes()[..buf.len()]);
                    self.next(); // remove the peeked value
                    buf.len()
                }
                std::cmp::Ordering::Greater => {
                    buf.copy_from_slice(&val.as_bytes()[..buf.len()]);
                    *val = &val[buf.len()..];
                    buf.len()
                }
            }
        } else {
            0
        };
        Ok(bytes)
    }
}

#[cfg(feature = "tokio")]
impl<'a, I: Iterator<Item = &'a str> + Unpin> tokio::io::AsyncRead for StringIterator<'a, I> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        // SAFETY: We don't de-initialise data in this buffer, as we don't construct any type that
        // has a `Drop` implementation.
        let write_to = unsafe { buf.unfilled_mut() };
        // SAFETY: We don't look at the bytes in the buffer, we just want to write to them, so we
        // can assume init and then write to them, resulting in them containing safe values.
        //
        // Really want: https://doc.rust-lang.org/std/primitive.slice.html#method.assume_init_ref
        let write_to = unsafe { &mut *(write_to as *mut [std::mem::MaybeUninit<u8>] as *mut [u8]) };
        let bytes = match std::io::Read::read(self.get_mut(), write_to) {
            Ok(bytes) => bytes,
            Err(e) => return std::task::Poll::Ready(Err(e)),
        };

        buf.advance(bytes);

        std::task::Poll::Ready(Ok(()))
    }
}

/// A structure which contains a template for users to write code.  Using specific comments, we
/// create regions that will be removed, used as the actual template, or populated with a user's
/// code.
///
/// ```
/// # use bedrock::templates::Template;
/// // This string is the example below
/// let t: Template = include_str!("../examples/reverse-template.rs")
///     .to_string()
///     .try_into()?;
///
/// assert_eq!(t.template(), "fn reverse(line: &str) -> String {\n    // Your solution here\n}");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// <details>
/// <summary>Example code used in functions</summary>
///
/// ```no_run
#[doc = include_str!("../examples/reverse-template.rs")]
/// ```
///
/// </details>
// All ranges are byte indicies into the `full` array.
#[derive(Debug, Clone)]
pub struct Template {
    full: String,
    /// The lines of code that come before the first template or solution
    prefix: Range<usize>,
    /// The lines of code that come after the last template or solution
    suffix: Range<usize>,
    order: Order,
    /// Range for the template, will be replace when running the code
    // Holding onto this since we parse the data already and it might be useful at a later point
    // in time
    _template_range: Range<usize>,
    /// String that contains the template for the competitor.  Needs to be a string to remove the
    /// comment prefix as necessary.
    template: String,
    indent: Range<usize>,
}

impl Template {
    /// Construct an empty template, with no solution or template.  Calling [`Template::template`]
    /// will result in the empty string.
    ///
    /// ```
    /// # use bedrock::templates::Template;
    /// # let user_code = "This is a random test string!";
    /// let template = Template::empty();
    /// assert_eq!(template.full(), "");
    /// assert_eq!(template.template(), "");
    /// assert_eq!(template.has_solution(), false);
    /// assert_eq!(template.populated(user_code).into_string().trim(), user_code);
    /// ```
    pub const fn empty() -> Self {
        Self {
            full: String::new(),
            prefix: 0..0,
            suffix: 0..0,
            order: Order::Template,
            _template_range: 0..0,
            template: String::new(),
            indent: 0..0,
        }
    }

    /// Get the full file that is used for this template
    ///
    /// ```
    /// # use bedrock::templates::Template;
    /// let raw_template = include_str!("../examples/reverse-template.rs");
    /// let t: Template = raw_template.to_string().try_into()?;
    /// assert_eq!(t.full(), raw_template);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn full(&self) -> &str {
        &self.full
    }

    /// Get whether this template has a host-defined solution
    pub fn has_solution(&self) -> bool {
        self.order.solution().is_some()
    }

    /// Get just the solution of the code, without the indent.
    ///
    /// From the example above, we'd get:
    /// ```ignore
    /// fn reverse(line: &str) -> String {
    ///     line.chars().rev().collect()
    /// }
    /// ```
    pub fn solution(&self) -> Option<StringIterator<'_, impl Iterator<Item = &str>>> {
        let solution = self.order.solution()?;

        let iter = self.full[solution.clone()]
            .lines()
            .flat_map(|l| [l.strip_prefix(self.indent()).unwrap(), "\n"]);

        Some(StringIterator::new(iter))
    }

    /// Get the user template that has been extracted from this template, without indent or leading
    /// comment.  Comment is extracted from the line containing `BASALT_TEMPLATE_START`
    ///
    /// From the example above, we'd get:
    /// ```ignore
    /// fn reverse(line: &str) -> String {
    ///     // Your solution here
    /// }
    /// ```
    pub fn template(&self) -> &str {
        &self.template
    }

    /// Populate the template using user's code.  This will replace the "template" section with the
    /// users code and removes the solution section.
    pub fn populated<'a>(
        &'a self,
        user_code: &'a str,
    ) -> StringIterator<'a, impl Iterator<Item = &'a str>> {
        let body = self.order.body(&self.full, user_code, self.indent());
        StringIterator::new(
            std::iter::once(&self.full[self.prefix.clone()])
                .chain(body)
                .chain(std::iter::once(&self.full[self.suffix.clone()]))
                .peekable(),
        )
    }

    /// Get the string used as the indent.  This is determined by looking at the line which
    /// contains the `BASALT_TEMPLATE_START` command.
    pub fn indent(&self) -> &str {
        &self.full[self.indent.clone()]
    }

    /// Directly writes the populated solution to a writer.  For more details of what populated
    /// means, see [`Template::populated`].
    ///
    /// Returns the total number of bytes written.
    ///
    /// Also see the [`Template::write_populated_async`] function if using `tokio`.
    pub fn write_populated<W>(&self, user_code: &str, mut w: W) -> io::Result<usize>
    where
        W: Write,
    {
        let mut populated = self.populated(user_code);
        let bytes = std::io::copy(&mut populated, &mut w)?;
        Ok(usize::try_from(bytes).expect("u64 <= usize on all intended platforms"))
    }

    /// Directly writes the populated solution to a writer.  For more details of what populated
    /// means, see [`Template::populated`].
    ///
    /// Returns the total number of bytes written.
    ///
    /// Also see the [`Template::write_populated`] function if not using `tokio`.
    #[cfg(feature = "tokio")]
    pub async fn write_populated_async<W>(&self, user_code: &str, mut w: W) -> io::Result<usize>
    where
        W: tokio::io::AsyncWrite + Unpin,
    {
        let mut populated = self.populated(user_code);
        let bytes = tokio::io::copy(&mut populated, &mut w).await?;
        Ok(usize::try_from(bytes).expect("u64 <= usize on all intended platforms"))
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum TemplateParseError {
    #[error("Missing template section")]
    MissingTemplate,
    #[error("Duplicate template section definition")]
    DuplicateTemplateSection {
        #[label("first definition here")]
        first: SourceSpan,
        #[label("second definition here")]
        second: SourceSpan,
    },
}

pub const SOLUTION_START: &str = "BASALT_SOLUTION_START";
pub const SOLUTION_END: &str = "BASALT_SOLUTION_END";
pub const TEMPLATE_START: &str = "BASALT_TEMPLATE_START";
pub const TEMPLATE_END: &str = "BASALT_TEMPLATE_END";

// NOTE: The choice for `TryFrom<String>`, rather than `FromStr` is to make the caller need to
// allocate the string, rather than having us do it.  This is done in an effort to remove
// unnecessary allocations.  Take for example the following code:
// ```rust
// let s: String = some_read_string_function();
// let t: Template = s.parse()?; // This would need to clone the string
// let t: Template = s.try_into()?; // This takes ownership of the string
// ```
//
// If the caller wants to clone the string, they can use `String::clone`:
// ```rust
// let t: Template = s.clone().try_into()?;
// ```
impl TryFrom<String> for Template {
    type Error = TemplateParseError;

    fn try_from(full: String) -> Result<Self, Self::Error> {
        let mut solution: Option<Range<usize>> = None;
        let mut template_range: Option<Range<usize>> = None;
        let mut template: Option<String> = None;

        #[derive(Debug, PartialEq, Eq)]
        enum State {
            None,
            Solution,
            Template,
        }

        let mut state = State::None;
        let mut start = 0;

        let mut comment = "";
        let mut indent = 0..0;
        for (offset, line) in lines_with_offset(&full) {
            if line.ends_with(SOLUTION_START) {
                assert_eq!(start, 0);
                start = offset;
                state = State::Solution;
            } else if line.ends_with(SOLUTION_END) {
                solution = Some(start..offset + line.len());
                start = 0;
                state = State::None;
            } else if line.ends_with(TEMPLATE_START) {
                comment = line.strip_suffix(TEMPLATE_START).unwrap();
                if indent == (0..0) {
                    indent = offset
                        ..offset
                            + line
                                .find(|c: char| !c.is_ascii_whitespace())
                                .unwrap_or_default();
                }
                assert_eq!(start, 0);
                start = offset;
                template = Some(String::new());
                state = State::Template;
            } else if line.ends_with(TEMPLATE_END) {
                template_range = Some(start..offset + line.len());
                start = 0;
                state = State::None;
            } else if state == State::Template {
                let x = template.as_mut().expect(
                    "if we are in this state, then we have already set template to be Some",
                );
                // TODO: better error here
                let line = line.strip_prefix(comment).unwrap();
                if !line.is_empty() {
                    if !x.is_empty() {
                        x.push('\n');
                    }
                    x.push_str(line);
                }
            }
        }

        let (Some(template_range), Some(template)) = (template_range, template) else {
            return Err(TemplateParseError::MissingTemplate);
        };

        let (prefix_end, suffix_start, template_first) = match (&template_range, &solution) {
            (t, None) => (t.start, t.end, true),
            (t, Some(s)) if t.start < s.start => (t.start, s.end, true),
            (t, Some(s)) => (s.start, t.end, false),
        };

        let full_len = full.len();
        Ok(Template {
            full,
            prefix: 0..prefix_end,
            suffix: suffix_start..full_len,
            order: if let Some(solution) = solution {
                if template_first {
                    Order::TemplateSolution {
                        infix: template_range.end..solution.start,
                        solution,
                    }
                } else {
                    Order::SolutionTemplate {
                        infix: solution.end..template_range.start,
                        solution,
                    }
                }
            } else {
                Order::Template
            },
            _template_range: template_range,
            template,
            indent,
        })
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from(s).map_err(serde::de::Error::custom)
    }
}
