use crate::term::Term;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char as nom_char, i64 as nom_i64, multispace0},
    combinator::{eof, map, opt},
    error::ParseError as NomParseError,
    multi::many0,
    sequence::delimited,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError;

pub fn parse(input: &str) -> Result<Term, ParseError> {
    (expr, eof)
        .parse(input)
        .map(|(_, (term, _))| term)
        .map_err(|_| ParseError)
}

fn expr(input: &str) -> IResult<&str, Term> {
    map(
        (
            term,
            opt((
                whitespace(nom_char('?')),
                expr,
                whitespace(nom_char(':')),
                expr,
            )),
        ),
        |(condition, rest)| match rest {
            Some((_, consequent, _, alternative)) => Term::Condition {
                condition: Box::new(condition),
                consequent: Box::new(consequent),
                alternative: Box::new(alternative),
            },
            None => condition,
        },
    )
    .parse(input)
}

fn term(input: &str) -> IResult<&str, Term> {
    map(
        (factor, many0((whitespace(nom_char('+')), term))),
        |(term, terms)| {
            terms.iter().fold(term, |left, (_, right)| Term::Addition {
                left: Box::new(left),
                right: Box::new(right.clone()),
            })
        },
    )
    .parse(input)
}

fn factor(input: &str) -> IResult<&str, Term> {
    alt((
        number,
        boolean,
        delimited(whitespace(nom_char('(')), expr, whitespace(nom_char(')'))),
    ))
    .parse(input)
}

fn number(input: &str) -> IResult<&str, Term> {
    whitespace(map(nom_i64, |value| Term::Number { value })).parse(input)
}

fn boolean(input: &str) -> IResult<&str, Term> {
    alt((
        whitespace(map(tag("true"), |_| Term::True)),
        whitespace(map(tag("false"), |_| Term::False)),
    ))
    .parse(input)
}

fn whitespace<'a, O, E: NomParseError<&'a str>, F>(
    inner: F,
) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use crate::{parser::parse, term::Term};

    #[test]
    fn test_boolean() {
        assert_eq!(parse("true").unwrap(), Term::True);
        assert_eq!(parse("false").unwrap(), Term::False);
    }

    #[test]
    fn test_number() {
        assert_eq!(parse("1").unwrap(), Term::Number { value: 1 });
        assert_eq!(parse("123").unwrap(), Term::Number { value: 123 });
    }

    #[test]
    fn test_addition() {
        assert_eq!(
            parse("1 + 2").unwrap(),
            Term::Addition {
                left: Box::new(Term::Number { value: 1 }),
                right: Box::new(Term::Number { value: 2 }),
            },
        );
        assert_eq!(
            parse("true + 1").unwrap(),
            Term::Addition {
                left: Box::new(Term::True),
                right: Box::new(Term::Number { value: 1 }),
            },
        );
        assert_eq!(
            parse("1 + true").unwrap(),
            Term::Addition {
                left: Box::new(Term::Number { value: 1 }),
                right: Box::new(Term::True),
            },
        );
    }

    #[test]
    fn test_condition() {
        assert_eq!(
            parse("true ? 1 : 2").unwrap(),
            Term::Condition {
                condition: Box::new(Term::True),
                consequent: Box::new(Term::Number { value: 1 }),
                alternative: Box::new(Term::Number { value: 2 }),
            },
        );
        assert_eq!(
            parse("true ? 1 : false").unwrap(),
            Term::Condition {
                condition: Box::new(Term::True),
                consequent: Box::new(Term::Number { value: 1 }),
                alternative: Box::new(Term::False),
            },
        );
    }

    #[test]
    fn test_parens() {
        assert_eq!(
            parse("(1 + 2) + 3").unwrap(),
            Term::Addition {
                left: Box::new(Term::Addition {
                    left: Box::new(Term::Number { value: 1 }),
                    right: Box::new(Term::Number { value: 2 }),
                }),
                right: Box::new(Term::Number { value: 3 }),
            },
        );
        assert_eq!(
            parse("1 + (2 + 3)").unwrap(),
            Term::Addition {
                left: Box::new(Term::Number { value: 1 }),
                right: Box::new(Term::Addition {
                    left: Box::new(Term::Number { value: 2 }),
                    right: Box::new(Term::Number { value: 3 }),
                }),
            },
        );
        assert_eq!(
            parse("(1 + true) ? 2 : 3").unwrap(),
            Term::Condition {
                condition: Box::new(Term::Addition {
                    left: Box::new(Term::Number { value: 1 }),
                    right: Box::new(Term::True),
                }),
                consequent: Box::new(Term::Number { value: 2 }),
                alternative: Box::new(Term::Number { value: 3 }),
            },
        );
        assert_eq!(
            parse("1 + (true ? 2 : 3)").unwrap(),
            Term::Addition {
                left: Box::new(Term::Number { value: 1 }),
                right: Box::new(Term::Condition {
                    condition: Box::new(Term::True),
                    consequent: Box::new(Term::Number { value: 2 }),
                    alternative: Box::new(Term::Number { value: 3 }),
                }),
            },
        );
        assert_eq!(
            parse("true ? (1 + 2) : (3 + 4)").unwrap(),
            Term::Condition {
                condition: Box::new(Term::True),
                consequent: Box::new(Term::Addition {
                    left: Box::new(Term::Number { value: 1 }),
                    right: Box::new(Term::Number { value: 2 }),
                }),
                alternative: Box::new(Term::Addition {
                    left: Box::new(Term::Number { value: 3 }),
                    right: Box::new(Term::Number { value: 4 }),
                }),
            },
        );
    }
}
