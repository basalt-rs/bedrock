use evalexpr::{context_map, eval_with_context, DefaultNumericTypes, HashMapContext, Value};

pub struct EvaluationContext {
    pub scoring_formula: String,
    pub default_points: i32,
    pub num_teams: u32,
    pub num_completions: u32,
    pub question_points: Option<i32>,
}

fn type_name(v: Value) -> String {
    match v {
        Value::Empty => "Empty".into(),
        Value::Int(_) => "Int".into(),
        Value::Float(_) => "Float".into(),
        Value::Boolean(_) => "Boolean".into(),
        Value::String(_) => "String".into(),
        Value::Tuple(_) => "Tuple".into(),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ScoreError {
    #[error("Something went wrong initializing the evaluation context")]
    ContextInitialization(String),
    #[error("Something went wrong evaluating the score {0}")]
    Evaluation(String),
    #[error("Invalid value produced: expected Int | Float, got {0}")]
    ValueError(String),
}

impl EvaluationContext {
    pub fn score(&self) -> Result<f64, ScoreError> {
        let inner_ctx: HashMapContext<DefaultNumericTypes> = context_map! {
            "p" => int self.question_points.unwrap_or(self.default_points),
            "points" => int self.question_points.unwrap_or(self.default_points),
            "c" => int self.num_completions,
            "completed" => int self.num_completions,
            "t" => int self.num_teams,
            "teams" => int self.num_teams,
            "t" => int self.num_teams,
            "teams" => int self.num_teams,
        }
        .map_err(|e| ScoreError::ContextInitialization(e.to_string()))?;

        match eval_with_context(&self.scoring_formula, &inner_ctx)
            .map_err(|e| ScoreError::Evaluation(e.to_string()))?
        {
            evalexpr::Value::Int(value) => Ok(value as f64),
            evalexpr::Value::Float(value) => Ok(value),
            ev => Err(ScoreError::ValueError(type_name(ev))),
        }
    }
}
