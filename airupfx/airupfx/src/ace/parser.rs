//! A very simple parser.

use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Command {
    pub module: String,
    pub args: Vec<String>,
}
impl<T> From<T> for Command
where
    T: Iterator<Item = String>,
{
    fn from(mut value: T) -> Self {
        let module = value.next().unwrap_or_else(|| "noop".into());
        let args = value.collect();

        Self { module, args }
    }
}
impl FromStr for Command {
    type Err = super::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(shlex::split(s)
            .ok_or(super::Error::ParseError)?
            .into_iter()
            .into())
    }
}
