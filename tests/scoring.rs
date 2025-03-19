use bedrock::scoring::*;
#[test]
fn basic_scoring() {
    assert_eq!(
        EvaluationContext {
            scoring_formula: "p - 2*c".into(),
            default_points: 10,
            num_teams: 5,
            num_completions: 0,
            question_points: None,
        }
        .score()
        .unwrap(),
        10.0
    );
    assert_eq!(
        EvaluationContext {
            scoring_formula: "p - 2*c".into(),
            default_points: 10,
            num_teams: 5,
            num_completions: 1,
            question_points: None,
        }
        .score()
        .unwrap(),
        8.0
    );
}
