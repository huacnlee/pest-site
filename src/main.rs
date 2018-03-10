extern crate pest_meta;
extern crate pest_vm;
#[macro_use]
extern crate stdweb;

use pest_meta::parser::{self, Rule};
use pest_meta::validator;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web;
use stdweb::web::event::{ClickEvent, InputEvent};
use stdweb::web::html_element::TextAreaElement;

macro_rules! try_output {
    ( $result:expr ) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                let output = web::document().query_selector(".editor-output").unwrap().unwrap();
                let output: TextAreaElement = output.try_into().unwrap();
                output.set_value(&format!(
                    "parsing error\n\n{}",
                    error
                ));

                return;
            }
        }
    };
}

macro_rules! try_output_vec {
    ( $result:expr ) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                let output = web::document().query_selector(".editor-output").unwrap().unwrap();
                let output: TextAreaElement = output.try_into().unwrap();
                output.set_value(&format!(
                    "grammar error\n\n{}",
                    &error.into_iter()
                        .map(|error| format!("{}", error))
                        .collect::<Vec<_>>()
                        .join("\n\n")
                ));

                return;
            }
        }
    };
}

static mut NEEDS_RUN: bool = false;

fn listen_for_input() {
    let grammar = web::document().query_selector(".editor-grammar").unwrap().unwrap();
    let input = web::document().query_selector(".editor-input").unwrap().unwrap();

    grammar.add_event_listener(move |_: InputEvent| {
        unsafe { NEEDS_RUN = true; }
        wait_and_run();
    });
    input.add_event_listener(move |_: InputEvent| {
        unsafe { NEEDS_RUN = true; }
        wait_and_run();
    });
}

fn wait_and_run() {
    web::set_timeout(|| {
        if unsafe { NEEDS_RUN } {
            compile_grammar();
            unsafe { NEEDS_RUN = false; }
        }
    }, 800);
}

fn compile_grammar() {
    let grammar = web::document().query_selector(".editor-grammar").unwrap().unwrap();
    let grammar: TextAreaElement = grammar.try_into().unwrap();
    let grammar = grammar.value();

    let pairs = try_output!(parser::parse(Rule::grammar_rules, &grammar).map_err(|error| {
        error.renamed_rules(|rule| match *rule {
            Rule::grammar_rule => "rule".to_owned(),
            Rule::_push => "push".to_owned(),
            Rule::assignment_operator => "`=`".to_owned(),
            Rule::silent_modifier => "`_`".to_owned(),
            Rule::atomic_modifier => "`@`".to_owned(),
            Rule::compound_atomic_modifier => "`$`".to_owned(),
            Rule::non_atomic_modifier => "`!`".to_owned(),
            Rule::opening_brace => "`{`".to_owned(),
            Rule::closing_brace => "`}`".to_owned(),
            Rule::opening_paren => "`(`".to_owned(),
            Rule::positive_predicate_operator => "`&`".to_owned(),
            Rule::negative_predicate_operator => "`!`".to_owned(),
            Rule::sequence_operator => "`&`".to_owned(),
            Rule::choice_operator => "`|`".to_owned(),
            Rule::optional_operator => "`?`".to_owned(),
            Rule::repeat_operator => "`*`".to_owned(),
            Rule::repeat_once_operator => "`+`".to_owned(),
            Rule::comma => "`,`".to_owned(),
            Rule::closing_paren => "`)`".to_owned(),
            Rule::quote => "`\"`".to_owned(),
            Rule::insensitive_string => "`^`".to_owned(),
            Rule::range_operator => "`..`".to_owned(),
            Rule::single_quote => "`'`".to_owned(),
            other_rule => format!("{:?}", other_rule)
        })
    }));
    let defaults = try_output_vec!(validator::validate_pairs(pairs.clone()));
    let ast = try_output_vec!(parser::consume_rules(pairs));
}

fn add_node() {
    let tree = web::document().query_selector(".editor-tree").unwrap().unwrap();

    let node = web::document().create_element("div").unwrap();
    node.class_list().add("node").unwrap();
    node.class_list().add("node-selected").unwrap();
    node.append_child(&web::document().create_text_node("digit"));
    let node_clone = node.clone();
    node.add_event_listener(move |_: ClickEvent| {
        js! {
            selected(@{&node_clone});
        }
    });

    tree.append_child(node.as_node());
}

fn main() {
    listen_for_input();
    add_node();
}