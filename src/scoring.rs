use crate::Config;

pub trait Scorable {
    fn score(&self) -> i64;
}

impl Scorable for Config {
    fn score(&self) -> i64 {}
}
