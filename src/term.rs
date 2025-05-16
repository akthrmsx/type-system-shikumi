#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    True,
    False,
    Condition {
        condition: Box<Term>,
        consequent: Box<Term>,
        alternative: Box<Term>,
    },
    Number {
        value: i64,
    },
    Addition {
        left: Box<Term>,
        right: Box<Term>,
    },
}
