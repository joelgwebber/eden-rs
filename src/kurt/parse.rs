use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashMap;

use crate::kurt::Node;
use crate::kurt::NodeRef;

#[derive(Parser)]
#[grammar = "kurt/kurt.pest"]
struct KurtParser;

pub fn parse(src: String) -> NodeRef {
    let file = KurtParser::parse(Rule::file, &src)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    fn parse_value(expr: Pair<Rule>) -> NodeRef {
        match expr.as_rule() {
            Rule::dict => {
                let mut map = HashMap::<String, NodeRef>::new();
                expr.into_inner().for_each(|pair| match pair.as_rule() {
                    Rule::pair => {
                        let mut inner_rules = pair.into_inner();
                        let sym = inner_rules.next().unwrap().as_str();
                        let value = parse_value(inner_rules.next().unwrap());
                        map.insert(String::from(sym), value);
                    }
                    _ => unreachable!(),
                });
                NodeRef::new(Node::Dict(map))
            }

            Rule::block => {
                let mut rules = expr.into_inner();
                let args = rules
                    .next()
                    .unwrap()
                    .into_inner()
                    .map(parse_value)
                    .collect();
                let exprs = rules.map(parse_value).collect();
                NodeRef::new(Node::Block(args, NodeRef::new(Node::List(exprs))))
            }

            Rule::exec => {
                let mut list = vec![NodeRef::new(Node::Id("$".to_string()))];
                list.extend(expr.into_inner().map(parse_value));
                NodeRef::new(Node::List(list))
            }

            Rule::list => NodeRef::new(Node::List(expr.into_inner().map(parse_value).collect())),
            Rule::number => NodeRef::new(Node::Num(expr.as_str().parse().unwrap())),
            Rule::boolean => NodeRef::new(Node::Bool(expr.as_str().parse().unwrap())),
            Rule::string => NodeRef::new(Node::Str(String::from(expr.as_str()))),
            Rule::sym => NodeRef::new(Node::Sym(String::from(&expr.as_str()[1..]))),
            Rule::id => NodeRef::new(Node::Id(String::from(expr.as_str()))),
            Rule::prim => parse_value(expr.into_inner().next().unwrap()),
            Rule::expr => parse_value(expr.into_inner().next().unwrap()),

            _ => unreachable!(),
        }
    }

    let expr = file.into_inner().next().unwrap();
    parse_value(expr)
}
