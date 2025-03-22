use bedrock::{scoring::*, Config};
const EXAMPLE_ONE_CONTENT: &str = include_str!("../examples/one.toml");
const EXAMPLE_TWO_CONTENT: &str = include_str!("../examples/two.toml");

#[test]
fn basic_scoring() {
    let config = Config::from_str(EXAMPLE_ONE_CONTENT, Some("Cargo.toml")).unwrap();
    assert_eq!(
        config
            .score(0, EvaluationContext { num_completions: 0 })
            .unwrap(),
        10.0
    );
}

#[test]
fn more_complicated_scoring() {
    let config = Config::from_str(EXAMPLE_TWO_CONTENT, Some("Cargo.toml")).unwrap();
    assert_eq!(
        config
            .score(0, EvaluationContext { num_completions: 0 })
            .unwrap(),
        20.0
    );
    assert_eq!(
        config
            .score(0, EvaluationContext { num_completions: 1 })
            .unwrap(),
        18.0
    );
    assert_eq!(
        config
            .score(1, EvaluationContext { num_completions: 0 })
            .unwrap(),
        40.0
    );
    assert_eq!(
        config
            .score(1, EvaluationContext { num_completions: 1 })
            .unwrap(),
        38.0
    );
}
