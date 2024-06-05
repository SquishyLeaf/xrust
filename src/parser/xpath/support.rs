//! Supporting functions.

use crate::item::Node;
use crate::parser::{ParseError, ParseInput};
use crate::transform::{NameTest, NodeTest, Transform, WildcardOrName};

pub(crate) fn get_nt_localname(nt: &NodeTest) -> String {
    match nt {
        NodeTest::Name(NameTest {
            name: Some(WildcardOrName::Name(localpart)),
            ns: None,
            prefix: None,
        }) => localpart.to_string(),
        _ => String::from("invalid qname"),
    }
}

/// Return zero or more digits from the input stream. Be careful not to consume non-digit input.
pub(crate) fn digit0<N: Node>(
) -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, String), ParseError> {
    move |(input, state)| {
        match input.find(|c| !(c >= '0' && c <= '9')) {
            Some(0) => Err(ParseError::Combinator),
            Some(pos) => {
                //let result = (&mut input).take(pos).collect::<String>();
                Ok(((&input[pos..], state), input[..pos].to_string()))
            }
            None => {
                if input.is_empty() {
                    Err(ParseError::Combinator)
                } else {
                    Ok((("", state), input.to_string()))
                }
            }
        }
    }
}
/// Return one or more digits from the input stream.
pub(crate) fn digit1<N: Node>(
) -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, String), ParseError> {
    move |(input, state)| {
        if input.starts_with(|c| (c >= '0' && c <= '9')) {
            match input.find(|c| !(c >= '0' && c <= '9')) {
                Some(0) => Ok(((&input[1..], state), input[..1].to_string())),
                Some(pos) => Ok(((&input[pos..], state), input[..pos].to_string())),
                None => Ok((("", state), input.to_string())),
            }
        } else {
            Err(ParseError::Combinator)
        }
    }
}

/// Return the next character if it is not from the given set
pub(crate) fn none_of<N: Node>(
    s: &str,
) -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, char), ParseError> + '_ {
    move |(input, state)| {
        if input.is_empty() {
            Err(ParseError::Combinator)
        } else {
            let a = input.chars().next().unwrap();
            match s.find(|b| a == b) {
                Some(_) => Err(ParseError::Combinator),
                None => Ok(((&input[1..], state), a)),
            }
        }
    }
}

pub(crate) fn noop<N: Node>(
) -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, Transform<N>), ParseError> {
    move |_| Err(ParseError::Combinator)
}
