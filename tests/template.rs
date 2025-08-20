use std::collections::HashMap;

use bedrock::{
    language::{BuiltInLanguage, Language, LanguageSet, Syntax, Version},
    packet::{Packet, Problem},
    ConfigValidationError,
};

const CONFIG_STR: &str = include_str!("./template.toml");

// #[test]
// fn deser() {
//     let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
//     config.validate().expect("Template should be valid");
// }
//
// #[test]
// fn deser_invalid1() {
//     let config = bedrock::Config {
//         setup: None,
//         port: 8517,
//         web_client: false,
//         game: Default::default(),
//         integrations: Default::default(),
//         max_submissions: Default::default(),
//         languages: LanguageSet::from_iter([
//             Language::BuiltIn {
//                 language: BuiltInLanguage::Python3,
//                 version: Version::Latest,
//             },
//             Language::Custom {
//                 raw_name: "haskell".into(),
//                 name: "Haskell".into(),
//                 build: None,
//                 run: "foo".into(),
//                 source_file: "solution.hs".into(),
//                 syntax: Syntax::Haskell,
//                 template: None,
//             },
//         ])
//         .into(),
//         templates: HashMap::from_iter([
//             ("haskell".into(), "foo".into()),
//             ("foobar".into(), "bazqux".into()),
//         ])
//         .into(),
//         accounts: Default::default(),
//         packet: Default::default(),
//         test_runner: Default::default(),
//     };
//     assert_eq!(
//         config.validate(),
//         Err(ConfigValidationError::UnknownTemplateLanguage {
//             language: "foobar".into(),
//             problem: None
//         })
//     );
// }
//
// #[test]
// fn deser_invalid2() {
//     let config = bedrock::Config {
//         setup: None,
//         port: 8517,
//         web_client: false,
//         game: Default::default(),
//         integrations: Default::default(),
//         max_submissions: Default::default(),
//         languages: LanguageSet::from_iter([
//             Language::BuiltIn {
//                 language: BuiltInLanguage::Python3,
//                 version: Version::Latest,
//             },
//             Language::Custom {
//                 raw_name: "haskell".into(),
//                 name: "Haskell".into(),
//                 build: None,
//                 run: "foo".into(),
//                 source_file: "solution.hs".into(),
//                 syntax: Syntax::Haskell,
//                 template: None,
//             },
//         ])
//         .into(),
//         templates: Default::default(),
//         accounts: Default::default(),
//         packet: Packet {
//             title: Default::default(),
//             preamble: Default::default(),
//             problems: vec![Problem {
//                 languages: None,
//                 title: "Some Title".into(),
//                 description: None,
//                 tests: vec![],
//                 points: None,
//                 templates: HashMap::from_iter([
//                     ("haskell".into(), "foo".into()),
//                     ("invalid".into(), "bazqux".into()),
//                 ])
//                 .into(),
//             }
//             .into()],
//         }
//         .into(),
//         test_runner: Default::default(),
//     };
//     assert_eq!(
//         config.validate(),
//         Err(ConfigValidationError::UnknownTemplateLanguage {
//             language: "invalid".into(),
//             problem: Some("Some Title".into())
//         })
//     );
// }
//
// #[test]
// fn correct_templates() {
//     let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
//     assert_eq!(
//         config.template("python3", 0),
//         Some("print(''.join(reversed(input())))")
//     );
//     assert_eq!(
//         config.template("haskell", 0),
//         Some(indoc::indoc!(
//             r#"
//             main = do
//               line <- getLine
//               putStrLn $ reverse line
//             "#
//         ))
//     );
// }
//
// #[test]
// fn correct_overrides() {
//     let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
//     assert_eq!(
//         config.template("python3", 1),
//         Some(indoc::indoc!(
//             r#"
//             x = float(input())
//             print(x)
//             "#
//         ))
//     );
// }
//
// #[test]
// fn correct_missing() {
//     let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
//     assert_eq!(config.template("haskell", 1), None);
// }
//
// #[test]
// fn correct_custom() {
//     let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
//     assert_eq!(
//         config.template("ocaml", 0),
//         Some(indoc::indoc!(
//             r#"
//             open Format
//
//             let () =
//                 let inp = read_int () in
//                 printf "%d\n" inp
//             ;;
//             "#
//         ))
//     );
//     assert_eq!(
//         config.template("ocaml", 1),
//         Some(indoc::indoc!(
//             r#"
//             open Format
//
//             let () =
//                 let inp = read_int () in
//                 printf "%d\n" inp
//             ;;
//             "#
//         ))
//     );
// }
