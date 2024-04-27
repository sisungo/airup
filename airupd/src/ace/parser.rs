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
        Ok(tokenize(s)
            .map_err(|_| super::Error::Parse)?
            .into_iter()
            .into())
    }
}

pub fn tokenize(s: &str) -> Result<Vec<String>, Error> {
    let mut tokens = Vec::with_capacity(s.len() / 4);
    let mut in_literal = false;
    let mut in_escape = false;
    let mut this = Vec::with_capacity(8);

    for b in s.bytes() {
        if in_literal {
            if in_escape {
                match b {
                    b'n' => this.push(b'\n'),
                    b'r' => this.push(b'\r'),
                    b'"' => this.push(b'"'),
                    b'\\' => this.push(b'\\'),
                    b'0' => this.push(b'\0'),
                    _ => this.push(b),
                };
                in_escape = false;
            } else {
                match b {
                    b'\\' => in_escape = true,
                    b'"' => {
                        in_literal = false;
                        tokens.push(
                            String::from_utf8(this.clone()).map_err(|_| Error::CorruptUnicode)?,
                        );
                        this.clear();
                    }
                    _ => this.push(b),
                }
            }
        } else if b"\" ".contains(&b) {
            if !this.is_empty() {
                tokens.push(String::from_utf8(this.clone()).map_err(|_| Error::CorruptUnicode)?);
                this.clear();
            }

            match b {
                b'"' => in_literal = true,
                b' ' => { /* skip */ }
                _ => unreachable!(),
            }
        } else {
            this.push(b);
        }
    }

    if in_literal {
        return Err(Error::IncompleteLiteral);
    }
    if !this.is_empty() {
        tokens.push(String::from_utf8(this.clone()).map_err(|_| Error::CorruptUnicode)?);
    }

    Ok(tokens)
}

pub enum Error {
    IncompleteLiteral,
    CorruptUnicode,
}
