//! Functions that produce strings.

use crate::item::Node;
use crate::parser::combinators::list::separated_list1;
use crate::parser::combinators::map::map;
use crate::parser::combinators::tag::tag;
use crate::parser::combinators::tuple::tuple3;
use crate::parser::combinators::whitespace::xpwhitespace;
use crate::parser::xpath::numbers::range_expr;
use crate::parser::{ParseError, ParseInput};
use crate::transform::Transform;

// StringConcatExpr ::= RangeExpr ( '||' RangeExpr)*
pub(crate) fn stringconcat_expr<'a, N: Node + 'a>(
) -> Box<dyn Fn(ParseInput<N>) -> Result<(ParseInput<N>, Transform<N>), ParseError> + 'a> {
    Box::new(map(
        separated_list1(
            map(tuple3(xpwhitespace(), tag("||"), xpwhitespace()), |_| ()),
            range_expr::<N>(),
        ),
        |mut v| {
            if v.len() == 1 {
                v.pop().unwrap()
            } else {
                Transform::Concat(v)
            }
        },
    ))
}
