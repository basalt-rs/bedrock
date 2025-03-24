use evalexpr::{context_map, eval_with_context, DefaultNumericTypes, HashMapContext, Value};

use crate::Config;

pub struct EvaluationContext {
    pub num_completions: u32,
    pub num_attempts: u32,
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
    #[error("This kind of competition cannot produce a numeric score")]
    Unscorable,
    #[error("The requested problem does not exist")]
    NoSuchProblem,
}

pub trait Scorable {
    fn score(&self, problem_idx: usize, ctx: EvaluationContext) -> Result<f64, ScoreError>;
}

impl Scorable for Config {
    fn score(&self, problem_idx: usize, ctx: EvaluationContext) -> Result<f64, ScoreError> {
        match &self.game {
            crate::Game::Points(settings) => {
                let points = self
                    .packet
                    .problems
                    .get(problem_idx)
                    .ok_or(ScoreError::NoSuchProblem)?
                    .points
                    .unwrap_or(settings.question_point_value);
                let teams = self.accounts.competitors.len() as i32;
                let inner_ctx: HashMapContext<DefaultNumericTypes> = context_map! {
                    "p" => int points,
                    "points" => int points,
                    "c" => int ctx.num_completions,
                    "completed" => int ctx.num_completions,
                    "t" => int teams,
                    "teams" => int teams,
                    "a" => int ctx.num_attempts,
                    "attempts" => int ctx.num_attempts,
                }
                .map_err(|e| ScoreError::ContextInitialization(e.to_string()))?;

                match eval_with_context(&settings.score, &inner_ctx)
                    .map_err(|e| ScoreError::Evaluation(e.to_string()))?
                {
                    evalexpr::Value::Int(value) => Ok(value as f64),
                    evalexpr::Value::Float(value) => Ok(value),
                    ev => Err(ScoreError::ValueError(type_name(ev))),
                }
            }
            _ => Err(ScoreError::Unscorable),
        }
    }
}
