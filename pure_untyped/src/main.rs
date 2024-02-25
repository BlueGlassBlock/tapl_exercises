use thiserror::Error;
use pest_typed_derive::TypedParser;
use pest_typed::TypedParser;
use clap::Parser as ClapParser;

#[derive(TypedParser)]
#[grammar = "grammar.pest"]
#[emit_rule_reference]
#[emit_tagged_node_reference]
struct Parser;



fn main() {
    
}