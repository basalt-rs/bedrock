const CONFIG_STR: &str = include_str!("./template.toml");

#[test]
fn deser() {
    let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
    config.validate().expect("Template should be valid");
}

#[test]
fn correct_templates() {
    let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
    assert_eq!(
        config.template("python3", 0),
        Some("print(''.join(reversed(input())))")
    );
    assert_eq!(
        config.template("haskell", 0),
        Some(indoc::indoc!(
            r#"
            main = do
              line <- getLine
              putStrLn $ reverse line
            "#
        ))
    );
}

#[test]
fn correct_overrides() {
    let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
    assert_eq!(
        config.template("python3", 1),
        Some(indoc::indoc!(
            r#"
            x = float(input())
            print(x)
            "#
        ))
    );
}

#[test]
fn correct_missing() {
    let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
    assert_eq!(config.template("haskell", 1), None);
}

#[test]
fn correct_custom() {
    let config = bedrock::Config::from_str(CONFIG_STR, Some("template.toml")).unwrap();
    assert_eq!(
        config.template("ocaml", 0),
        Some(indoc::indoc!(
            r#"
            open Format

            let () = 
                let inp = read_int () in 
                printf "%d\n" inp
            ;;
            "#
        ))
    );
    assert_eq!(
        config.template("ocaml", 1),
        Some(indoc::indoc!(
            r#"
            open Format

            let () = 
                let inp = read_int () in 
                printf "%d\n" inp
            ;;
            "#
        ))
    );
}
