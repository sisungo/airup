//! A parser based on `rust-peg`.

peg::parser! {
    grammar ace() for str {
        // ==- Commons -==
        rule _()
            = quiet!{comment()}
            / quiet!{[x if x.is_ascii_whitespace()]}

        // ==- Comments -==
        rule comment()
            = quiet! {"#" [^ '\n']* "\n"}
            / expected!("comment")

        // ==- Text Literals -==
        rule ascii_escape() -> char
            = "\\" esc:[x if ['n', 'r', 't', '\\', '0'].contains(&x)] { map_ascii_escape(esc) }

        rule quote_escape() -> char
            = "\\" esc:[x if ['\'', '"'].contains(&x)] { esc }

        rule text_literal_escape() -> char
            = x:(ascii_escape() / quote_escape()) { x }

        rule strong_string_literal() -> String
            = "\""  s:(text_literal_escape() / [^ '\n' | '\r' | '\\' | '\"'])* "\"" { s.into_iter().collect() }

        rule weak_string_literal() -> &'input str
            = s:$([^ '$' | '"'] [^ '\n' | '\r' | '\\' | '\"' | ' ']*) { s }

        rule string_literal() -> String
            = quiet! {x:strong_string_literal() { x }}
            / quiet! {x:weak_string_literal() { x.into() }}
            / expected!("string literal")

        // ==- Variables -==
        rule ident() -> &'input str
            = s:$([^ x if (x.is_ascii_punctuation() && x != '_' && x != '-') || x.is_whitespace() ]*) { s }

        rule variable() -> &'input str
            = "${" s:ident() "}" { s }

        // ==- Expressions -==
        rule expr() -> String
            = v:variable() {? std::env::var(v).or(Err("environment not found")) }
            / s:string_literal() { s }

        pub rule command() -> Command
            = _* module:expr() _+ args:(expr() ** _) _* {
                Command { module, args }
            }
    }
}

fn map_ascii_escape(x: char) -> char {
    match x {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '\\' => '\\',
        '0' => '\0',
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Command {
    pub module: String,
    pub args: Vec<String>,
}
impl Command {
    /// Parses a command.
    pub fn parse(s: &str) -> Result<Self, anyhow::Error> {
        Ok(ace::command(s)?)
    }

    /// Wraps a `sudo`-pattern command.
    pub fn wrap<T>(mut self, f: impl FnOnce(Self) -> T) -> Option<T> {
        if self.args.is_empty() {
            return None;
        }
        let module = self.args.remove(0);
        Some(f(Self {
            module,
            args: self.args,
        }))
    }
}

#[cfg(test)]
#[test]
fn tests() {
    unsafe {
        std::env::set_var("TEST_ENV", "It works!");
    }

    assert_eq!(
        Command::parse("echo \"Hello, world!\"").unwrap(),
        Command {
            module: "echo".into(),
            args: vec!["Hello, world!".into()],
        }
    );
    assert_eq!(
        Command::parse("echo ${TEST_ENV}").unwrap(),
        Command {
            module: "echo".into(),
            args: vec!["It works!".into()],
        }
    );
    assert_eq!(
        Command::parse("echo -n Hello, world!").unwrap(),
        Command {
            module: "echo".into(),
            args: vec!["-n".into(), "Hello,".into(), "world!".into()],
        }
    );

    Command::parse("echo \"Hello, world!").unwrap_err();
    Command::parse("echo ${__ENV_NON_EXISTENT__}").unwrap_err();
}
