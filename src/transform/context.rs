//! # Dynamic context for a transformation

use crate::item::{Node, Sequence};
use crate::output::OutputDefinition;
use crate::transform::booleans::*;
use crate::transform::construct::*;
use crate::transform::controlflow::*;
use crate::transform::datetime::*;
use crate::transform::functions::*;
use crate::transform::grouping::*;
use crate::transform::logic::*;
use crate::transform::navigate::*;
use crate::transform::numbers::*;
use crate::transform::strings::*;
use crate::transform::template::{apply_imports, apply_templates, next_match, Template};
use crate::transform::variables::{declare_variable, reference_variable};
use crate::transform::Transform;
use crate::xdmerror::Error;
use crate::{ErrorKind, Item, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;
use url::Url;

/// The transformation context. This is the dynamic context, plus some parts of the static context.
/// Contexts are immutable, but frequently are cloned to provide a new context.
#[derive(Clone, Debug)]
pub struct Context<N: Node> {
    pub(crate) cur: Sequence<N>, // The current context
    pub(crate) i: usize,         // The index to the item that is the current context item
    pub(crate) depth: usize,     // Depth of evaluation
    pub(crate) rd: Option<N>,    // Result document
    // There is no distinction between built-in and user-defined templates
    // Built-in templates have no priority and no document order
    pub(crate) templates: Vec<Rc<Template<N>>>,
    pub(crate) current_templates: Vec<Rc<Template<N>>>,
    // Variables, with scoping
    pub(crate) vars: HashMap<String, Vec<Sequence<N>>>,
    // Grouping
    pub(crate) current_grouping_key: Option<Value>,
    pub(crate) current_group: Sequence<N>,
    // Output control
    pub(crate) od: OutputDefinition,
    pub(crate) base_url: Option<Url>,
}

impl<N: Node> Context<N> {
    pub fn new() -> Self {
        Context {
            cur: Sequence::new(),
            i: 0,
            depth: 0,
            rd: None,
            templates: vec![],
            current_templates: vec![],
            vars: HashMap::new(),
            current_grouping_key: None,
            current_group: Sequence::new(),
            od: OutputDefinition::new(),
            base_url: None,
        }
    }
    pub fn context(&mut self, s: Sequence<N>, i: usize) {
        self.cur = s;
        self.i = i;
    }
    pub fn result_document(&mut self, rd: N) {
        self.rd = Some(rd);
    }
    pub(crate) fn var_push(&mut self, name: String, value: Sequence<N>) {
        match self.vars.get_mut(name.as_str()) {
            Some(u) => {
                // If the variable already has a value, then this is a new, inner scope
                u.push(value);
            }
            None => {
                // Otherwise this is the first scope for the variable
                self.vars.insert(name, vec![value]);
            }
        }
    }
    #[allow(dead_code)]
    fn var_pop(&mut self, name: String) {
        self.vars.get_mut(name.as_str()).map(|u| u.pop());
    }

    #[allow(dead_code)]
    fn baseurl(&self) -> Option<Url> {
        self.base_url.clone()
    }
    #[allow(dead_code)]
    fn set_baseurl(&mut self, url: Url) {
        self.base_url = Some(url);
    }

    /// Evaluate finds a template matching the current item and evaluates the body of the template,
    /// returning the resulting [Sequence]
    pub fn evaluate(&self) -> Result<Sequence<N>, Error> {
        if self.cur.is_empty() {
            Ok(Sequence::new())
        } else {
            self.cur.get(self.i).map_or_else(
                || {
                    Err(Error::new(
                        ErrorKind::DynamicAbsent,
                        String::from("bad index into current sequence"),
                    ))
                },
                |i| {
                    // There may be 0, 1, or more matching templates.
                    // If there are more than one with the same priority and import level,
                    // then take the one with the higher document order.
                    let templates = self.find_templates(i)?;
                    match templates.len() {
                        0 => Err(Error::new(
                            ErrorKind::DynamicAbsent,
                            String::from("no matching template"),
                        )),
                        1 => self.dispatch(&templates[0].body),
                        _ => {
                            if templates[0].priority == templates[1].priority
                                && templates[0].import.len() == templates[1].import.len()
                            {
                                let mut candidates: Vec<Rc<Template<N>>> = templates
                                    .iter()
                                    .take_while(|t| {
                                        t.priority == templates[0].priority
                                            && t.import.len() == templates[0].import.len()
                                    })
                                    .cloned()
                                    .collect();
                                candidates.sort_unstable_by(|a, b| {
                                    a.document_order.map_or(Ordering::Greater, |v| {
                                        b.document_order.map_or(Ordering::Less, |u| v.cmp(&u))
                                    })
                                });
                                self.dispatch(&candidates.last().unwrap().body)
                            } else {
                                self.dispatch(&templates[0].body)
                            }
                        }
                    }
                },
            )
        }
    }

    /// Find a template with a matching [Pattern]
    pub fn find_templates(&self, i: &Rc<Item<N>>) -> Result<Vec<Rc<Template<N>>>, Error> {
        let mut candidates = self.templates.iter().try_fold(vec![], |mut cand, t| {
            let e = t.pattern.matches(self, i);
            if e {
                cand.push(t.clone())
            }
            Ok(cand)
        })?;
        if candidates.len() != 0 {
            // Find the template(s) with the lowest priority.

            candidates.sort_unstable_by(|a, b| (*a).cmp(&*b));
            Ok(candidates)
        } else {
            Err(Error::new(
                ErrorKind::Unknown,
                String::from("no matching template"),
            ))
        }
    }

    /// Interpret the given [Transform] object
    pub fn dispatch(&self, t: &Transform<N>) -> Result<Sequence<N>, Error> {
        match t {
            Transform::Root => root(self),
            Transform::ContextItem => context(self),
            Transform::Compose(v) => compose(self, v),
            Transform::Step(nm) => step(self, nm),
            Transform::Filter(t) => filter(self, t),
            Transform::Empty => empty(self),
            Transform::Literal(v) => literal(self, v),
            Transform::LiteralElement(qn, t) => literal_element(self, qn, t),
            Transform::LiteralAttribute(qn, t) => literal_attribute(self, qn, t),
            Transform::SetAttribute(qn, v) => set_attribute(self, qn, v),
            Transform::SequenceItems(v) => make_sequence(self, v),
            Transform::Copy(f, t) => copy(self, f, t),
            Transform::DeepCopy(d) => deep_copy(self, d),
            Transform::Or(v) => tr_or(self, v),
            Transform::And(v) => tr_and(self, v),
            Transform::Union(b) => union(self, b),
            Transform::GeneralComparison(o, l, r) => general_comparison(self, o, l, r),
            Transform::ValueComparison(o, l, r) => value_comparison(self, o, l, r),
            Transform::Concat(v) => tr_concat(self, v),
            Transform::Range(s, e) => tr_range(self, s, e),
            Transform::Arithmetic(v) => arithmetic(self, v),
            Transform::Loop(v, b) => tr_loop(self, v, b),
            Transform::Switch(c, o) => switch(self, c, o),
            Transform::ForEach(g, s, b) => for_each(self, g, s, b),
            Transform::ApplyTemplates(s) => apply_templates(self, s),
            Transform::ApplyImports => apply_imports(self),
            Transform::NextMatch => next_match(self),
            Transform::VariableDeclaration(n, v, f) => declare_variable(self, n.clone(), v, f),
            Transform::VariableReference(n) => reference_variable(self, n),
            Transform::Position => position(self),
            Transform::Last => last(self),
            Transform::Count(s) => tr_count(self, s),
            Transform::LocalName(s) => local_name(self, s),
            Transform::Name(s) => name(self, s),
            Transform::String(s) => string(self, s),
            Transform::StartsWith(s, t) => starts_with(self, s, t),
            Transform::EndsWith(s, t) => ends_with(self, s, t),
            Transform::Contains(s, t) => contains(self, s, t),
            Transform::Substring(s, t, l) => substring(self, s, t, l),
            Transform::SubstringBefore(s, t) => substring_before(self, s, t),
            Transform::SubstringAfter(s, t) => substring_after(self, s, t),
            Transform::NormalizeSpace(s) => normalize_space(self, s),
            Transform::Translate(s, m, t) => translate(self, s, m, t),
            Transform::Boolean(b) => boolean(self, b),
            Transform::Not(b) => not(self, b),
            Transform::True => tr_true(self),
            Transform::False => tr_false(self),
            Transform::Number(n) => number(self, n),
            Transform::Sum(s) => sum(self, s),
            Transform::Floor(n) => floor(self, n),
            Transform::Ceiling(n) => ceiling(self, n),
            Transform::Round(n, p) => round(self, n, p),
            Transform::CurrentGroup => current_group(self),
            Transform::CurrentGroupingKey => current_grouping_key(self),
            Transform::CurrentDateTime => current_date_time(self),
            Transform::CurrentDate => current_date(self),
            Transform::CurrentTime => current_time(self),
            Transform::FormatDateTime(t, p, l, c, q) => format_date_time(self, t, p, l, c, q),
            Transform::FormatDate(t, p, l, c, q) => format_date(self, t, p, l, c, q),
            Transform::FormatTime(t, p, l, c, q) => format_time(self, t, p, l, c, q),
            Transform::UserDefined(_qn, a, b) => user_defined(self, a, b),
            Transform::Error(k, m) => tr_error(self, k, m),
            Transform::NotImplemented(s) => not_implemented(self, s),
            _ => Err(Error::new(
                ErrorKind::NotImplemented,
                "not implemented".to_string(),
            )),
        }
    }
}

impl<N: Node> From<Sequence<N>> for Context<N> {
    fn from(value: Sequence<N>) -> Self {
        Context {
            cur: value,
            i: 0,
            depth: 0,
            rd: None,
            templates: vec![],
            current_templates: vec![],
            vars: HashMap::new(),
            current_grouping_key: None,
            current_group: Sequence::new(),
            od: OutputDefinition::new(),
            base_url: None,
        }
    }
}

/// Builder for a [Context]
pub struct ContextBuilder<N: Node>(Context<N>);

impl<N: Node> ContextBuilder<N> {
    pub fn new() -> Self {
        ContextBuilder(Context::new())
    }
    pub fn current(mut self, s: Sequence<N>) -> Self {
        self.0.cur = s;
        self
    }
    pub fn index(mut self, i: usize) -> Self {
        self.0.i = i;
        self
    }
    pub fn depth(mut self, d: usize) -> Self {
        self.0.depth = d;
        self
    }
    pub fn variable(mut self, n: String, v: Sequence<N>) -> Self {
        self.0.var_push(n, v);
        self
    }
    pub fn variables(mut self, v: HashMap<String, Vec<Sequence<N>>>) -> Self {
        self.0.vars = v;
        self
    }
    pub fn result_document(mut self, rd: N) -> Self {
        self.0.rd = Some(rd);
        self
    }
    pub fn template(mut self, t: Template<N>) -> Self {
        self.0.templates.push(Rc::new(t));
        self
    }
    pub fn template_all(mut self, v: Vec<Template<N>>) -> Self {
        for t in v {
            self.0.templates.push(Rc::new(t))
        }
        self
    }
    pub fn current_templates(mut self, c: Vec<Rc<Template<N>>>) -> Self {
        self.0.current_templates = c;
        self
    }
    pub fn current_group(mut self, c: Sequence<N>) -> Self {
        self.0.current_group = c;
        self
    }
    pub fn current_grouping_key(mut self, k: Value) -> Self {
        self.0.current_grouping_key = Some(k);
        self
    }
    pub fn output_definition(mut self, od: OutputDefinition) -> Self {
        self.0.od = od;
        self
    }
    pub fn base_url(mut self, b: Url) -> Self {
        self.0.base_url = Some(b);
        self
    }
    pub fn build(self) -> Context<N> {
        self.0
    }
}

impl<N: Node> From<&Context<N>> for ContextBuilder<N> {
    fn from(c: &Context<N>) -> Self {
        ContextBuilder(c.clone())
    }
}
