//! General productions for XPath expressions.

use crate::item::Node;
use crate::parser::combinators::alt::{alt2, alt5};
use crate::parser::combinators::map::map;
use crate::parser::{ParseError, ParseInput};
//use crate::parser::combinators::debug::inspect;
use crate::parser::combinators::delimited::delimited;
use crate::parser::combinators::tag::tag;
use crate::parser::xpath::context::context_item;
use crate::parser::xpath::expr_wrapper;
use crate::parser::xpath::functions::function_call;
use crate::parser::xpath::literals::literal;
use crate::parser::xpath::variables::variable_reference;
use crate::transform::Transform;

// PostfixExpr ::= PrimaryExpr (Predicate | ArgumentList | Lookup)*
// TODO: predicates, arg list, lookup
pub(crate) fn postfix_expr<'a, N: Node + 'a>(
) -> Box<dyn Fn(ParseInput<N>) -> Result<(ParseInput<N>, Transform<N>), ParseError> + 'a> {
    Box::new(primary_expr::<N>())
}

// PrimaryExpr ::= Literal | VarRef | ParenthesizedExpr | ContextItemExpr | FunctionCall | FunctionItemExpr | MapConstructor | ArrayConstructor | UnaryLookup
// TODO: finish this parser
fn primary_expr<'a, N: Node + 'a>(
) -> Box<dyn Fn(ParseInput<N>) -> Result<(ParseInput<N>, Transform<N>), ParseError> + 'a> {
    Box::new(alt5(
        literal::<N>(),
        parenthesized_expr::<N>(),
        function_call::<N>(),
        variable_reference::<N>(),
        context_item::<N>(),
    ))
}

// ParenthesizedExpr ::= '(' Expr? ')'
pub(crate) fn parenthesized_expr<'a, N: Node + 'a>(
) -> Box<dyn Fn(ParseInput<N>) -> Result<(ParseInput<N>, Transform<N>), ParseError> + 'a> {
    Box::new(alt2(
        parenthesized_expr_empty::<N>(),
        parenthesized_expr_nonempty::<N>(),
    ))
}
fn parenthesized_expr_empty<'a, N: Node + 'a>(
) -> Box<dyn Fn(ParseInput<N>) -> Result<(ParseInput<N>, Transform<N>), ParseError> + 'a> {
    Box::new(map(tag("()"), |_| Transform::Empty))
}
fn parenthesized_expr_nonempty<'a, N: Node + 'a>(
) -> Box<dyn Fn(ParseInput<N>) -> Result<(ParseInput<N>, Transform<N>), ParseError> + 'a> {
    Box::new(delimited(
        tag("("),
        map(expr_wrapper::<N>(true), |e| e),
        tag(")"),
    ))
}
