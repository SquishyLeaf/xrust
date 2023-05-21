mod intsubset;
mod extsubset;
mod elementdecl;
mod attlistdecl;
mod pedecl;
mod gedecl;
mod enumerated;
mod notation;
mod misc;

use crate::parser::{ParseInput, ParseResult};
use crate::parser::combinators::alt::alt2;
use crate::parser::combinators::delimited::delimited;
use crate::parser::combinators::map::map;
use crate::parser::combinators::opt::opt;
use crate::parser::combinators::tag::tag;
use crate::parser::combinators::take::{take_until, take_while};
use crate::parser::combinators::tuple::{tuple3, tuple5, tuple8};
use crate::parser::combinators::whitespace::{whitespace0, whitespace1};
use crate::parser::common::{is_pubid_char, is_pubid_charwithapos};
use crate::parser::xml::qname::name;
use crate::parser::xml::dtd::intsubset::intsubset;

pub(crate) fn doctypedecl() -> impl Fn(ParseInput) -> ParseResult<()> {
    move |input| match tuple8(
        tag("<!DOCTYPE"),
        whitespace1(),
        name(),
        whitespace1(),
        opt(externalid()),
        whitespace0(),
        opt(delimited(tag("["), intsubset(), tag("]"))),
        tag(">"),
    )(input)
    {
        Ok((d, (_, _, _n, _, _e, _, _inss, _))) => Ok((d, ())),
        Err(err) => Err(err),
    }
}


fn externalid() -> impl Fn(ParseInput) -> ParseResult<(String, Option<String>)> {
    alt2(
        map(
            tuple3(
                tag("SYSTEM"),
                whitespace0(),
                alt2(
                    delimited(tag("'"), take_until("'"), tag("'")),
                    delimited(tag("\""), take_until("\""), tag("\"")),
                ), //SystemLiteral
            ),
            |(_, _, sid)| (sid, None),
        ),
        map(
            tuple5(
                tag("PUBLIC"),
                whitespace0(),
                alt2(
                    delimited(tag("'"), take_while(|c| !is_pubid_char(&c)), tag("'")),
                    delimited(
                        tag("\""),
                        take_while(|c| !is_pubid_charwithapos(&c)),
                        tag("\""),
                    ),
                ), //PubidLiteral TODO validate chars here (PubidChar from spec).
                whitespace1(),
                alt2(
                    delimited(tag("'"), take_until("'"), tag("'")),
                    delimited(tag("\""), take_until("\""), tag("\"")),
                ), //SystemLiteral
            ),
            |(_, _, pid, _, sid)| (sid, Some(pid)),
        ),
    )
}
