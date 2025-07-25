use std::{
    hash::{DefaultHasher, Hasher},
    io::Read,
    path::PathBuf,
    time::Duration,
};

use integrations::Integrations;
use language::LanguageSet;
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode};
use packet::Packet;
use roi::RawOrImport;
use serde::{Deserialize, Serialize};
use typst::foundations::{Array, Dict, IntoValue, Str, Value};

mod custom_serde;
pub mod integrations;
pub mod language;
pub mod packet;
pub mod render;
pub mod roi;
pub mod scoring;

mod util;

#[cfg(test)]
mod tests;

pub(crate) fn default_false() -> bool {
    false
}

pub(crate) fn default_true() -> bool {
    false
}

pub(crate) fn default_port() -> u16 {
    8517
}

pub(crate) fn default_time_limit() -> Duration {
    Duration::from_secs(60 * 75)
}

pub(crate) fn default_points() -> i32 {
    10
}

/// Authentication details for a specific user (competitor or host)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub name: String,
    pub display_name: Option<String>,
    pub password: String,
}

impl User {
    pub fn display_name(&self) -> &str {
        self.display_name.as_ref().unwrap_or(&self.name)
    }
}

/// Set of users that are either hosts or competitors
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[serde(deny_unknown_fields)]
pub struct Accounts {
    /// Hosts in charge of managing the competition
    pub hosts: Vec<User>,
    /// Competitors participating in the competition
    pub competitors: Vec<User>,
}

impl Accounts {
    pub fn to_value(&self) -> (Value, Value) {
        let hosts = self
            .hosts
            .iter()
            .map(|h| {
                [
                    (Str::from("username"), Str::from(&*h.name).into_value()),
                    (Str::from("password"), Str::from(&*h.password).into_value()),
                ]
                .into_iter()
                .collect::<Dict>()
                .into_value()
            })
            .collect::<Array>();
        let competitors = self
            .competitors
            .iter()
            .map(|c| {
                [
                    (Str::from("username"), Str::from(&*c.name).into_value()),
                    (Str::from("password"), Str::from(&*c.password).into_value()),
                ]
                .into_iter()
                .collect::<Dict>()
                .into_value()
            })
            .collect::<Array>();

        (hosts.into_value(), competitors.into_value())
    }
}

/// Configuration for setting up the docker container and starting the server
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    /// Specifies what commands are to be run when building the container to ensure dependencies
    /// are installed.
    pub install: Option<RawOrImport<String, roi::Raw>>,
    /// Specifies commands to run before running basalt-server so that dependencies are enabled
    /// properly.
    pub init: Option<RawOrImport<String, roi::Raw>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct PointsSettings {
    /// An expression used to evaluate how many points a competitor should get upon
    /// completion of a problem. This expression has access to variables that can allow for
    /// very flexible scoring mechanisms. By default however, all points will simply be
    /// awarded upon successful submission and evaluation.
    ///
    /// Variables:
    ///
    /// `p` or `points`: number of points available for the problem
    ///
    /// `t` or `teams`: number of teams in the competition
    ///
    /// `c` or `completed`: number of teams that have already completed the problem
    ///
    /// `a` or `attempts`: number of attempts that a team has made to solve the problem
    ///
    /// Example:
    /// ```toml
    /// # Decrease the points by 2 each time someone completes it.
    /// score = "p - 2*c"
    /// ```
    pub score: String,
    #[serde(default = "default_points")]
    pub question_point_value: i32,
    #[serde(default = "default_time_limit")]
    pub time_limit: Duration,
}

impl Default for PointsSettings {
    fn default() -> Self {
        Self {
            score: "p".into(),
            question_point_value: default_points(),
            time_limit: default_time_limit(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[serde(untagged)]
pub enum RaceMode {
    #[default]
    Sprint,
    Endurance,
    Relay,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct RaceSettings {
    pub race: RaceMode,
    pub arcade: bool,
    pub time_limit: Option<Duration>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(untagged)]
pub enum Game {
    Points(PointsSettings),
    Race(RaceSettings),
}

impl Default for Game {
    fn default() -> Self {
        Self::Points(PointsSettings::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct FileCopy {
    /// Source file to copy
    ///
    /// Relative to the directory in which the server is running
    pub from: PathBuf,
    /// Destination of the file
    ///
    /// Relative to the directory in which the test is run
    pub to: PathBuf,
}

/// Mirrors the `CommandConfig` type in [leucite](https://basalt-rs.github.io/erudite/erudite/struct.CommandConfig.html)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[serde(deny_unknown_fields, untagged)]
pub enum CommandConfig<T> {
    #[default]
    Neither,
    Both(T),
    Compile {
        compile: T,
    },
    Run {
        run: T,
    },
    Each {
        compile: T,
        run: T,
    },
}

impl<T> CommandConfig<T> {
    pub fn compile(&self) -> Option<&T> {
        match self {
            CommandConfig::Neither => None,
            CommandConfig::Both(t) => Some(t),
            CommandConfig::Compile { compile } => Some(compile),
            CommandConfig::Run { .. } => None,
            CommandConfig::Each { compile, .. } => Some(compile),
        }
    }

    pub fn run(&self) -> Option<&T> {
        match self {
            CommandConfig::Neither => None,
            CommandConfig::Both(t) => Some(t),
            CommandConfig::Compile { .. } => None,
            CommandConfig::Run { run } => Some(run),
            CommandConfig::Each { run, .. } => Some(run),
        }
    }
}

/// Configuration for the test runner
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
pub struct TestRunner {
    /// The amount of time that a test may run before it is cancelled by the test runner and marked
    /// as failure
    ///
    /// Measured in milliseconds
    ///
    /// [Default: 10 seconds]
    #[serde(rename = "timeout_ms")] // renamed so unit is obvious
    #[serde(
        with = "custom_serde::duration",
        default = "TestRunner::default_timeout"
    )]
    pub timeout: Duration,
    /// Whether the test runner should trim the output of a test before comparing with the
    /// expected output
    ///
    /// If this is true, the output of `hello world    ` matches the expected output of ` hello
    /// world`
    ///
    /// [Default: true]
    #[serde(default = "TestRunner::default_trim_output")]
    pub trim_output: bool,
    /// Files to copy into the test directory
    #[serde(default)]
    pub copy_files: Vec<FileCopy>,
    /// Amount of memory that may be used by the process, measured in MiB
    #[serde(default)]
    pub max_memory: CommandConfig<u64>,
    /// Maximum size of files that may be created by the tests, measured in MiB
    #[serde(default)]
    pub max_file_size: CommandConfig<u64>,
}

impl TestRunner {
    fn default_timeout() -> Duration {
        Duration::from_secs(10)
    }

    fn default_trim_output() -> bool {
        true
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self {
            timeout: Self::default_timeout(),
            trim_output: Self::default_trim_output(),
            copy_files: Default::default(),
            max_memory: CommandConfig::Neither,
            max_file_size: CommandConfig::Neither,
        }
    }
}

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum ConfigReadError {
    /// The Config file was unable to be read due to an IO error
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),
    /// The data being deserialised was formatted incorrectly
    #[error("{}", .0.to_string())] // needed to use the miette error instead of thiserror
    #[diagnostic(transparent)]
    MalformedData(miette::Error),
}

impl ConfigReadError {
    fn malformed<S>(source: S, value: toml_edit::de::Error) -> Self
    where
        S: SourceCode + 'static,
    {
        let labels = if let Some(span) = value.span() {
            vec![LabeledSpan::new_with_span(Some("here".into()), span)]
        } else {
            Vec::new()
        };
        Self::MalformedData(
            miette::miette! {
                labels = labels,
                "{}", value.message()
            }
            .with_source_code(source),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Configuration for setting up the docker container and starting the server
    pub setup: Option<RawOrImport<Setup>>,
    /// Port on which the server will be hosted
    #[serde(default = "default_port")]
    pub port: u16,
    /// Whether this competition should host the web client
    #[serde(default = "default_true")]
    pub web_client: bool,
    /// Indicates the mode by which the competition is held.
    ///
    /// In points, each team must attempt to score the most points possible
    #[serde(default)]
    pub game: Game,
    #[serde(default)]
    /// Contains information for all things related to programmability and
    /// external integrations in Basalt.
    pub integrations: Integrations,
    /// Maximum number of attempts that a user is allowed to make for a given problem
    pub max_submissions: Option<std::num::NonZero<u32>>,
    /// List of languages available for the server
    pub languages: RawOrImport<LanguageSet>,
    /// Accounts that will be granted access to the server
    pub accounts: RawOrImport<Accounts>,
    /// The packet for this competition
    pub packet: RawOrImport<Packet>,
    /// Configuration for the test runner
    #[serde(default)]
    pub test_runner: RawOrImport<TestRunner>,
}

impl std::hash::Hash for Config {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.setup.hash(state);
        // skip port
        // self.setup.hash(port);
        // self.events.hash(state);
        self.web_client.hash(state);
        self.languages.hash(state);
        self.accounts.hash(state);
        self.packet.hash(state);
        self.test_runner.hash(state);
    }
}

impl Config {
    /// Read config from a string
    ///
    /// - `file_name` provided for better miette errors
    pub fn from_str(
        content: impl AsRef<str>,
        file_name: Option<impl AsRef<str>>,
    ) -> Result<Self, ConfigReadError> {
        let content = content.as_ref();
        let config: Self = toml_edit::de::from_str(content).map_err(|e| {
            if let Some(file_name) = file_name {
                ConfigReadError::malformed(
                    NamedSource::new(file_name, content.to_string()).with_language("TOML"),
                    e,
                )
            } else {
                ConfigReadError::malformed(content.to_string(), e)
            }
        })?;
        Ok(config)
    }

    /// Read config from a file
    ///
    /// - `file_name` provided for better miette errors
    pub fn read<R>(
        reader: &mut R,
        file_name: Option<impl AsRef<str>>,
    ) -> Result<Self, ConfigReadError>
    where
        R: Read,
    {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        Self::from_str(&buf, file_name)
    }

    /// Read config from a file asynchronously
    ///
    /// - `file_name` provided for better miette errors
    #[cfg(feature = "tokio")]
    pub async fn read_async<R>(
        reader: &mut R,
        file_name: Option<impl AsRef<str>>,
    ) -> Result<Self, ConfigReadError>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        use tokio::io::AsyncReadExt;
        let mut buf = String::new();
        reader.read_to_string(&mut buf).await?;
        Self::from_str(&buf, file_name)
    }

    /// Generate a hash string for this config
    ///
    /// ```
    /// # use bedrock::Config;
    /// # let config = Config::default();
    /// let hash = format!("Your hash is: {}", config.hash());
    /// ```
    pub fn hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(self, &mut hasher);
        let mut hash = hasher.finish();
        const N: u64 = 36;
        const ALPHABET: [u8; N as usize] = *b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut out = String::with_capacity(14);
        loop {
            let n = (hash % N) as usize;
            hash /= N;
            out.push(ALPHABET[n] as char);
            if hash == 0 {
                break;
            }
        }
        out
    }

    /// Render the competition information to a PDF, either using a provided template (written in
    /// [typst](https://typst.app/)) or the default template
    ///
    /// # Template
    ///
    /// The template currently provides several variables that contain information about the
    /// competition.
    ///
    /// - `#title`: `str` - the title of the competition
    /// - `#preamble`: `content` - rendered markdown of the competition
    /// - `#problems`: `array<Dict>` - array of problems in the packet
    pub fn render_pdf(&self, template: Option<String>) -> std::io::Result<Vec<u8>> {
        let template = if let Some(template) = template {
            template
        } else {
            #[cfg(feature = "dev")]
            {
                std::fs::read_to_string("./data/template.typ").unwrap()
            }
            #[cfg(not(feature = "dev"))]
            {
                include_str!("../data/template.typ").into()
            }
        };

        let mut world = render::typst::TypstWrapperWorld::new(template);

        let mut errs = Vec::new();
        let mut problems = Array::with_capacity(self.packet.problems.len());
        for p in &self.packet.problems {
            match p.as_value(&world) {
                Ok(v) => problems.push(v),
                Err(err) => errs.push(err),
            }
        }

        world
            .library
            .global
            .scope_mut()
            .define("problems", problems);

        world
            .library
            .global
            .scope_mut()
            .define("title", self.packet.title.as_str());

        let preamble = self
            .packet
            .preamble
            .as_deref()
            .map(|s| s.content(&world))
            .transpose()?;
        world
            .library
            .global
            .scope_mut()
            .define("preamble", preamble);

        let document = typst::compile(&world)
            .output
            .expect("Error compiling typst");
        typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
            .map_err(|e| std::io::Error::other(format!("{:?}", e)))
    }

    /// Note: In the current implementation of `typst-pdf`, this just renders to a vector and then
    /// writes that to the `writer`.
    pub fn write_pdf<W>(&self, writer: &mut W, template: Option<String>) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        // XXX: I would really love it if typst offered an API that did not have to create a vec
        // just to render the PDF
        let vec = self.render_pdf(template)?;
        writer.write_all(&vec)
    }

    /// Render competition logins to PDF, either using a provided template (written in
    /// [typst](https://typst.app/)) or the default template
    ///
    /// # Template
    ///
    /// The template currently provides several variables that contain information about the
    /// competition.
    ///
    /// - `#title`: `str` - the title of the competition
    /// - `#preamble`: `content` - rendered markdown of the competition
    /// - `#problems`: `array<Dict>` - array of problems in the packet
    pub fn render_login_pdf(&self, template: Option<String>) -> std::io::Result<Vec<u8>> {
        let template = if let Some(template) = template {
            template
        } else {
            #[cfg(feature = "dev")]
            {
                std::fs::read_to_string("./data/login-template.typ").unwrap()
            }
            #[cfg(not(feature = "dev"))]
            {
                include_str!("../data/login-template.typ").into()
            }
        };

        let mut world = render::typst::TypstWrapperWorld::new(template);

        let mut errs = Vec::new();
        let mut problems = Array::with_capacity(self.packet.problems.len());
        for p in &self.packet.problems {
            match p.as_value(&world) {
                Ok(v) => problems.push(v),
                Err(err) => errs.push(err),
            }
        }

        world
            .library
            .global
            .scope_mut()
            .define("problems", problems);

        world
            .library
            .global
            .scope_mut()
            .define("title", self.packet.title.as_str());

        let preamble = self
            .packet
            .preamble
            .as_deref()
            .map(|s| s.content(&world))
            .transpose()?;
        world
            .library
            .global
            .scope_mut()
            .define("preamble", preamble);

        let (hosts, competitors) = self.accounts.to_value();
        world.library.global.scope_mut().define("hosts", hosts);
        world
            .library
            .global
            .scope_mut()
            .define("competitors", competitors);

        let document = typst::compile(&world)
            .output
            .expect("Error compiling typst");
        typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
            .map_err(|e| std::io::Error::other(format!("{:?}", e)))
    }

    /// Note: In the current implementation of `typst-pdf`, this just renders to a vector and then
    /// writes that to the `writer`.
    pub fn write_login_pdf<W>(
        &self,
        writer: &mut W,
        template: Option<String>,
    ) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        // XXX: I would really love it if typst offered an API that did not have to create a vec
        // just to render the PDF
        let vec = self.render_login_pdf(template)?;
        writer.write_all(&vec)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            setup: None,
            port: default_port(),
            web_client: true,
            integrations: Default::default(),
            game: Default::default(),
            max_submissions: None,
            languages: Default::default(),
            accounts: Default::default(),
            packet: Default::default(),
            test_runner: Default::default(),
        }
    }
}
