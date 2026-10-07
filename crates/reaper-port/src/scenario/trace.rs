use std::fmt;

/// What a scenario run did, line by line, for people to read and tests to compare.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Trace {
    lines: Vec<String>,
}

impl Trace {
    /// Adds a line.
    pub fn push(&mut self, line: String) {
        self.lines.push(line);
    }

    /// The lines so far.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.lines {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}
