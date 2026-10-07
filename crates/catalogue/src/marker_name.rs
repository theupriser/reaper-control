use crate::Directive;

/// A marker name read through the one grammar: whitespace-separated tokens,
/// our directives picked out, the rest plain text.
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerName {
    directives: Vec<Directive>,
    hidden: bool,
}

impl MarkerName {
    /// Never fails: anything that is not a directive is plain text.
    pub fn parse(name: &str) -> Self {
        let mut directives = Vec::new();
        let mut tokens = 0;
        let mut commands = 0;
        for token in name.split_whitespace() {
            tokens += 1;
            if let Some(directive) = Directive::parse(token) {
                directives.push(directive);
                commands += 1;
            } else if is_action(token) {
                commands += 1;
            }
        }
        Self {
            directives,
            hidden: tokens > 0 && tokens == commands,
        }
    }

    /// Our directives in order of appearance.
    pub fn directives(&self) -> &[Directive] {
        &self.directives
    }

    /// Only commands, no label: the timeline does not draw it.
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }
}

/// `!<digits>` is an SWS marker action; it is not ours but it is not a label either.
fn is_action(token: &str) -> bool {
    token
        .strip_prefix('!')
        .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
}
