use std::collections::HashMap;
use xrust::item::{Node, NodeType};
use xrust::item_node_tests;
use xrust::item_value_tests;
use xrust::pattern_tests;
use xrust::qname::QualifiedName;
use xrust::transform::context::{Context, ContextBuilder, StaticContext, StaticContextBuilder};
use xrust::transform_tests;
use xrust::trees::intmuttree::Document;
use xrust::trees::intmuttree::{NodeBuilder, RNode};
use xrust::xdmerror::{Error, ErrorKind};
use xrust::xpath_tests;
use xrust::xslt_tests;

type F = Box<dyn FnMut(&str) -> Result<(), Error>>;

fn make_empty_doc() -> RNode {
    NodeBuilder::new(NodeType::Document).build()
}

fn make_doc(n: QualifiedName, v: Value) -> RNode {
    let mut d = NodeBuilder::new(NodeType::Document).build();
    let mut child = NodeBuilder::new(NodeType::Element).name(n).build();
    d.push(child.clone()).expect("unable to append child");
    child
        .push(NodeBuilder::new(NodeType::Text).value(Rc::new(v)).build())
        .expect("unable to append child");
    d
}

fn make_sd_raw() -> RNode {
    let r = Document::try_from((
        "<a id='a1'><b id='b1'><a id='a2'><b id='b2'/><b id='b3'/></a><a id='a3'><b id='b4'/><b id='b5'/></a></b><b id='b6'><a id='a4'><b id='b7'/><b id='b8'/></a><a id='a5'><b id='b9'/><b id='b10'/></a></b></a>",
        None,None ))
        .expect("failed to parse XML");
    r.content[0].clone()
}
fn make_sd() -> Item<RNode> {
    let r = make_sd_raw();
    //let e = r.clone();
    //let mut d = NodeBuilder::new(NodeType::Document).build();
    //d.push(e).expect("unable to append node");
    //Item::Node(d)
    Item::Node(r)
}

fn make_from_str(s: &str) -> Result<RNode, Error> {
    Ok(Document::try_from((s, None, None))?.content[0].clone())
}

fn make_from_str_with_ns(s: &str) -> Result<(RNode, Vec<HashMap<String, String>>), Error> {
    let mut ns = HashMap::new();
    ns.insert(
        String::from("xsl"),
        String::from("http://www.w3.org/1999/XSL/Transform"),
    );
    Ok((
        Document::try_from((s, None, None))?.content[0].clone(),
        vec![ns],
    ))
}

item_value_tests!(RNode);
item_node_tests!(make_empty_doc, make_doc, make_sd_raw);
pattern_tests!(RNode, make_empty_doc, make_sd);
transform_tests!(RNode, make_empty_doc, make_doc);
xpath_tests!(RNode, make_empty_doc, make_sd);
xslt_tests!(make_from_str, make_empty_doc, make_from_str_with_ns);
