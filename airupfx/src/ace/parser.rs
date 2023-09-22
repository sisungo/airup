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
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(tokenize(s).into_iter().into())
    }
}

pub fn tokenize(s: &str) -> Vec<String> {
    let mut result = Vec::with_capacity(4);
    let mut token_buf = String::new();

    // *NOTE*: This implementation of the tokenizer is a placeholder. It is not working as expected and
    // is to be replaced in further versions.
    for c in s.chars() {
        if c == ' ' {
            result.push(token_buf.clone());
            token_buf.clear();
        } else {
            token_buf.push(c);
        }
    }

    if !token_buf.is_empty() {
        result.push(token_buf);
    }

    result
}
