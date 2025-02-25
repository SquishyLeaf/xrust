// Support functions for smite tests

use std::collections::HashMap;
use std::rc::Rc;

use xrust::item::{Item, Node};
use xrust::parser::xml::{parse as xmlparse, parse_with_ns};
use xrust::qname::QualifiedName;
use xrust::trees::smite::{Node as SmiteNode, RNode};
use xrust::value::Value;
use xrust::xdmerror::Error;

#[allow(dead_code)]
pub fn make_empty_doc() -> RNode {
    Rc::new(SmiteNode::new())
}

#[allow(dead_code)]
pub fn make_doc(n: QualifiedName, v: Value) -> RNode {
    let mut d = Rc::new(SmiteNode::new());
    let mut child = d.new_element(n).expect("unable to create element");
    d.push(child.clone()).expect("unable to add element node");
    child
        .push(
            child
                .new_text(Rc::new(v))
                .expect("unable to create text node"),
        )
        .expect("unable to add text node");
    d
}

#[allow(dead_code)]
pub fn make_sd_raw() -> RNode {
    let doc = Rc::new(SmiteNode::new());
    xmlparse(doc.clone(),
             "<a id='a1'><b id='b1'><a id='a2'><b id='b2'/><b id='b3'/></a><a id='a3'><b id='b4'/><b id='b5'/></a></b><b id='b6'><a id='a4'><b id='b7'/><b id='b8'/></a><a id='a5'><b id='b9'/><b id='b10'/></a></b></a>",
             None).expect("unable to parse XML");
    doc
}
#[allow(dead_code)]
pub fn make_sd_cooked() -> Result<RNode, Error> {
    Ok(make_sd_raw())
}
#[allow(dead_code)]
pub fn make_sd() -> Item<RNode> {
    Item::Node(make_sd_raw())
}

#[allow(dead_code)]
pub fn make_from_str(s: &str) -> Result<RNode, Error> {
    let doc = Rc::new(SmiteNode::new());
    xmlparse(doc.clone(), s, None)?;
    Ok(doc)
}

#[allow(dead_code)]
pub fn make_from_str_with_ns(s: &str) -> Result<(RNode, Vec<HashMap<String, String>>), Error> {
    let doc = Rc::new(SmiteNode::new());
    let r = parse_with_ns(doc.clone(), s, None)?;
    Ok(r)
}
