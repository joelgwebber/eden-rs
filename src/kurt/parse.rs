use pest::iterators::Pair;
use pest::Parser;

use crate::kurt::Block;
use crate::kurt::Node;
use crate::kurt::NodeRef;

#[derive(Parser)]
#[grammar = "kurt/kurt.pest"]
struct KurtParser;

pub fn parse(src: String) -> Node {
    let file = KurtParser::parse(Rule::file, &src)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    fn parse_value(expr: Pair<Rule>) -> Node {
        match expr.as_rule() {
            Rule::dict => {
                let mut vec = Vec::<(Node, Node)>::new();
                expr.into_inner().for_each(|pair| match pair.as_rule() {
                    Rule::pair => {
                        let mut inner_rules = pair.into_inner();
                        let key = parse_value(inner_rules.next().unwrap());
                        let value = parse_value(inner_rules.next().unwrap());
                        vec.push((key, value));
                    }
                    _ => unreachable!(),
                });
                Node::Assoc(NodeRef::new(vec))
            }

            Rule::block => {
                let mut rules = expr.into_inner();
                let params = rules
                    .next()
                    .unwrap()
                    .into_inner()
                    .map(|expr| match expr.as_rule() {
                        Rule::id => expr.as_str().to_string(),
                        _ => panic!(),
                    })
                    .collect();
                let exprs = rules.map(parse_value).collect();
                Node::Block(NodeRef::new(Block {
                    params: params,
                    env: Node::Nil,
                    expr: Node::Apply(NodeRef::new(exprs)),
                }))
            }

            Rule::apply => Node::Apply(NodeRef::new(expr.into_inner().map(parse_value).collect())),
            Rule::list => Node::List(NodeRef::new(expr.into_inner().map(parse_value).collect())),
            Rule::number => Node::Num(expr.as_str().parse().unwrap()),
            Rule::boolean => Node::Bool(expr.as_str().parse().unwrap()),
            Rule::string => Node::Str(String::from(expr.as_str())),
            Rule::id => Node::Id(String::from(expr.as_str())),
            Rule::prim => parse_value(expr.into_inner().next().unwrap()),
            Rule::expr => parse_value(expr.into_inner().next().unwrap()),

            Rule::quote => Node::Quote(NodeRef::new(parse_value(expr.into_inner().next().unwrap()))),
            Rule::unquote => Node::Unquote(NodeRef::new(parse_value(expr.into_inner().next().unwrap()))),

            _ => unreachable!(),
        }
    }

    let expr = file.into_inner().next().unwrap();
    parse_value(expr)
}
