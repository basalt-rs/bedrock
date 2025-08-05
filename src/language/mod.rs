mod language_set;
use std::str::FromStr;

pub use language_set::LanguageSet;

use indoc::indoc;
use phf::{phf_map, phf_ordered_map};
use serde::{Deserialize, Serialize};
use strum::VariantNames;

struct LanguageVersion {
    build: Option<&'static str>,
    run: &'static str,
    install_command: Option<&'static str>,
    init_command: Option<&'static str>,
}

struct Builtin {
    builtin: BuiltInLanguage,
    source_file: &'static str,
    syntax: Syntax,
    versions: phf::OrderedMap<&'static str, LanguageVersion>,
    template: &'static str,
}

// TODO: enforce minimum version count of 1 at compile time
static BUILTINS: phf::Map<&'static str, Builtin> = phf_map! {
    "python3" => Builtin {
        builtin: BuiltInLanguage::Python3,
        source_file: "solution.py",
        syntax: Syntax::Python,
        versions: phf_ordered_map! {
            "latest" => LanguageVersion {
                build: None,
                run: "python3 ./solution.py",
                install_command: Some("dnf install python3 -y"),
                init_command: None,
            }
        },
        template: indoc!(r#"
        num = int(input())
        print(num)
        "#),
    },
    "java" => Builtin {
        builtin: BuiltInLanguage::Java,
        source_file: "Solution.java",
        syntax: Syntax::Java,
        versions: phf_ordered_map! { // `java[c]` is fine since we only allow one language at a time
            "8" => LanguageVersion {
                build: Some("javac Solution.java"),
                run: "java Solution",
                install_command: Some("dnf install java-1.8.0-openjdk-devel -y"),
                init_command: None,
            },
            "11" => LanguageVersion {
                build: Some("javac Solution.java"),
                run: "java Solution",
                install_command: Some("dnf install java-11-openjdk-devel -y"),
                init_command: None,
            },
            "21" => LanguageVersion {
                build: Some("javac Solution.java"),
                run: "java Solution",
                install_command: Some("dnf install java-21-openjdk-devel -y"),
                init_command: None,
            },
        },
        template: indoc!(r#"
        import java.util.Scanner;

        public class Solution {
            public static void main(String[] args) {
                Scanner scanner = new Scanner(System.in);

                int num = scanner.nextInt();
                System.out.println(num);
            }
        }
        "#),
    },
    "javascript" => Builtin {
        builtin: BuiltInLanguage::JavaScript,
        source_file: "solution.js",
        syntax: Syntax::Javascript,
        versions: phf_ordered_map! {
            "latest" => LanguageVersion {
                build: None,
                run: "deno run -A solution.js",
                install_command: Some(indoc!(r#"
                deno_install_path=$(mktemp)
                curl -o "$deno_install_path" -fsSL https://deno.land/x/install/install.sh
                sh "$deno_install_path" -y
                "#)),
                init_command: None,
            }
        },
        template: indoc!(r#"
        const num = parseInt(prompt(''));
        console.log(num);
        "#),
    },
    "rust" => Builtin {
        builtin: BuiltInLanguage::Rust,
        source_file: "solution.rs",
        syntax: Syntax::Rust,
        versions: phf_ordered_map! {
            "latest" => LanguageVersion {
                build: Some("rustc -o solution solution.rs"),
                run: "./solution",
                install_command: Some("dnf install rust -y"),
                init_command: None,
            }
        },
        template: indoc!(r#"
        fn main() {
            let num: i32 = std::io::stdin()
                .lines()
                .next()
                .unwrap()
                .unwrap()
                .parse()
                .unwrap();
            println!("{}", num);
        }
        "#),
    },
};

#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, VariantNames,
)]
#[strum(serialize_all = "lowercase")]
pub enum BuiltInLanguage {
    Python3,
    Java,
    JavaScript,
    Rust,
}

impl BuiltInLanguage {
    pub fn has_version(self, version: &Version) -> Result<(), Vec<&str>> {
        let bil = &BUILTINS[self.as_str()];
        match version {
            Version::Latest => Ok(()),
            Version::Specific(v) => {
                if bil.versions.contains_key(v) {
                    Ok(())
                } else {
                    Err(bil.versions.keys().copied().collect())
                }
            }
        }
    }

    pub fn joined_variants() -> String {
        BuiltInLanguage::VARIANTS
            .iter()
            .map(|s| format!("'{}'", s))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Python3 => "python3",
            Self::Java => "java",
            Self::JavaScript => "javascript",
            Self::Rust => "rust",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Python3 => "Python3",
            Self::Java => "Java",
            Self::JavaScript => "JavaScript",
            Self::Rust => "Rust",
        }
    }

    pub fn source_file(self) -> &'static str {
        BUILTINS[self.as_str()].source_file
    }

    pub fn build_command(self, version: &Version) -> Option<&str> {
        let bil = &BUILTINS[self.as_str()];
        match version {
            Version::Latest => bil.versions.values().last()?.build,
            Version::Specific(v) => bil.versions[v].build,
        }
    }

    pub fn run_command(self, version: &Version) -> &str {
        let bil = &BUILTINS[self.as_str()];
        match version {
            Version::Latest => {
                bil.versions
                    .values()
                    .last()
                    .expect("all language must have at least one version")
                    .run
            }
            Version::Specific(v) => bil.versions[v].run,
        }
    }

    pub fn install_command(self, version: &Version) -> Option<&str> {
        let bil = &BUILTINS[self.as_str()];
        match version {
            Version::Latest => {
                bil.versions
                    .values()
                    .last()
                    .expect("all language must have at least one version")
                    .install_command
            }
            Version::Specific(v) => bil.versions[v].install_command,
        }
    }

    pub fn init_command(self, version: &Version) -> Option<&str> {
        let bil = &BUILTINS[self.as_str()];
        match version {
            Version::Latest => {
                bil.versions
                    .values()
                    .last()
                    .expect("all language must have at least one version")
                    .init_command
            }
            Version::Specific(v) => bil.versions[v].init_command,
        }
    }

    pub fn syntax(self) -> Syntax {
        BUILTINS[self.as_str()].syntax
    }

    pub fn template(self) -> &'static str {
        BUILTINS[self.as_str()].template
    }
}

impl From<&str> for BuiltInLanguage {
    fn from(value: &str) -> Self {
        BUILTINS[value].builtin
    }
}

impl FromStr for BuiltInLanguage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BUILTINS.get(s).map(|b| b.builtin).ok_or(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Version {
    Latest,
    Specific(String),
}

// Mostly from <https://github.com/ajaxorg/ace/tree/master/src/mode>
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum Syntax {
    Ada,
    Basic,
    Batchfile,
    #[serde(alias = "c", alias = "cpp")]
    CCpp,
    Clojure,
    Cobol,
    Csharp,
    D,
    Dart,
    Ejs,
    Elixir,
    Elm,
    Erlang,
    Forth,
    Fortran,
    Fsharp,
    Golang,
    Haskell,
    Java,
    Javascript,
    Julia,
    Kotlin,
    Lisp,
    Lua,
    Mips,
    Nim,
    Nix,
    Ocaml,
    Odin,
    Pascal,
    Perl,
    Php,
    #[default]
    PlainText,
    Powershell,
    Prolog,
    Python,
    R,
    Ruby,
    Rust,
    Scala,
    Scheme,
    Sh,
    Typescript,
    Zig,
}

impl Syntax {
    pub fn from_string<E: serde::de::Error>(s: impl AsRef<str>) -> Result<Self, E> {
        Syntax::deserialize(serde::de::value::StrDeserializer::<E>::new(s.as_ref()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Language {
    BuiltIn {
        language: BuiltInLanguage,
        version: Version,
    },
    Custom {
        raw_name: String,
        name: String,
        build: Option<String>,
        run: String,
        source_file: String,
        syntax: Syntax,
        template: Option<String>,
    },
}

impl Language {
    pub fn raw_name(&self) -> &str {
        match self {
            Language::BuiltIn { language, .. } => language.as_str(),
            Language::Custom { raw_name, .. } => raw_name,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Language::BuiltIn { language, .. } => language.name(),
            Language::Custom { name, .. } => name,
        }
    }

    pub fn source_file(&self) -> &str {
        match self {
            Language::BuiltIn { language, .. } => language.source_file(),
            Language::Custom { source_file, .. } => source_file,
        }
    }

    pub fn build_command(&self) -> Option<&str> {
        match self {
            Language::BuiltIn { language, version } => language.build_command(version),
            Language::Custom { build, .. } => build.as_deref(),
        }
    }

    pub fn run_command(&self) -> &str {
        match self {
            Language::BuiltIn { language, version } => language.run_command(version),
            Language::Custom { run, .. } => run,
        }
    }

    pub fn install_command(&self) -> Option<&str> {
        match self {
            Language::BuiltIn { language, version } => language.install_command(version),
            Language::Custom { .. } => None,
        }
    }

    pub fn init_command(&self) -> Option<&str> {
        match self {
            Language::BuiltIn { language, version } => language.init_command(version),
            Language::Custom { .. } => None,
        }
    }

    pub fn syntax(&self) -> Syntax {
        match self {
            Language::BuiltIn { language, .. } => language.syntax(),
            Language::Custom { syntax, .. } => *syntax,
        }
    }

    pub fn template(&self) -> &str {
        match self {
            Language::BuiltIn { language, .. } => language.template(),
            Language::Custom { template, .. } => template.as_deref().unwrap_or(""),
        }
    }
}
