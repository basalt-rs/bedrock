use basalt_bedrock::Config;

/// Test that the hash does not change between executions of the application
#[test]
fn hash_consistent() {
    let a = Config {
        setup: None,
        port: 69,
        web_client: true,
        integrations: Default::default(),
        game: Default::default(),
        max_submissions: None,
        languages: Default::default(),
        accounts: Default::default(),
        packet: Default::default(),
        test_runner: Default::default(),
    };

    assert_eq!("1y15mwwp3sysd", a.hash());
}

#[test]
fn port_diff() {
    let a = Config {
        setup: None,
        port: 69,
        web_client: true,
        integrations: Default::default(),
        game: Default::default(),
        max_submissions: None,
        languages: Default::default(),
        accounts: Default::default(),
        packet: Default::default(),
        test_runner: Default::default(),
    };

    let b = Config {
        setup: None,
        port: 420,
        web_client: true,
        game: Default::default(),
        integrations: Default::default(),
        max_submissions: None,
        languages: Default::default(),
        accounts: Default::default(),
        packet: Default::default(),
        test_runner: Default::default(),
    };

    assert_eq!(dbg!(a.hash()), b.hash());
}

#[test]
fn whitespace_diff() {
    let a = Config::from_str(
        r#"
port = 80
accounts.hosts = []
accounts.competitors = []
[languages]
python3 = "latest"
java = "21"
ocaml = { build = "ocamlc -o out solution.ml", run = "./out", source_file = "solution.ml" }

# Specify information about the packet itself
[packet]
# import = "./packet.toml"
title = "Example Packet"
preamble = '''
...
'''
problems = []
"#,
        None::<&str>,
    )
    .unwrap();

    let b = Config::from_str(
        r#"
port = 80
accounts.hosts = []
accounts.competitors = []
# Specify information a
[packet]
# import = "./packet.toml"
title = "Example Packet"
preamble = '''
...
'''
problems = []

[languages]
python3 = "latest"
java = "21"
ocaml = { build = "ocamlc -o out solution.ml", run = "./out", source_file = "solution.ml" }




"#,
        None::<&str>,
    )
    .unwrap();

    assert_eq!(dbg!(a.hash()), b.hash());
}
