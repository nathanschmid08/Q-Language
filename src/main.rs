use std::fs;

use clap::{Parser as ClapParser, Subcommand};
use pest::iterators::Pair;
use pest::Parser as PestParser;
use pest_derive::Parser;

mod ast;
mod interpreter;
use ast::*;
use interpreter::Interpreter;

#[derive(Parser)]
#[grammar = "q.pest"]
pub struct QParser;

#[derive(ClapParser)]
#[command(name = "quentin")]
#[command(author = "Your Name <youremail@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "Compiler for the Q programming language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a Q file or project
    Build {
        /// The path to the Q file to build
        file: Option<String>,
        #[arg(long)]
        log: bool,
    },
    /// Run the latest build
    Run {
        /// The name of the build to run
        name: Option<String>,
    },
    /// Clear the build cache
    Clear {
        /// The name of the cache to clear
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { file, log } => {
            if let Some(file_path) = file {
                println!("Building file: {}", file_path);
                let content = fs::read_to_string(file_path)
                    .expect("Should have been able to read the file");

                let pairs = QParser::parse(Rule::file, &content)
                    .unwrap_or_else(|e| panic!("Error parsing file: {}", e));

                let ast = build_ast(pairs);
                
                println!("Running interpreter...");
                let mut interpreter = Interpreter::new();
                interpreter.interpret(&ast);


            } else {
                println!("Building the whole project...");
            }
            if *log {
                println!("With logging enabled.");
            }
        }
        Commands::Run { name } => {
            if let Some(name) = name {
                println!("Running build: {}", name);
            } else {
                println!("Running latest build...");
            }
        }
        Commands::Clear { name } => {
            if let Some(name) = name {
                println!("Clearing cache: {}", name);
            } else {
                println!("Clearing all caches...");
            }
        }
    }
}

fn build_ast(pairs: pest::iterators::Pairs<Rule>) -> Vec<AstNode> {
    pairs
        .filter_map(|pair| match pair.as_rule() {
            Rule::statement => Some(AstNode::Statement(build_statement(pair))),
            Rule::EOI => None,
            Rule::comment => None,
            _ => {
                println!("unhandled rule: {:?}", pair.as_rule());
                None
            }
        })
        .collect()
}

fn build_statement(pair: Pair<Rule>) -> Statement {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::system_init => {
            let mut name = None;
            let mut data_type = None;
            let mut value = None;
            for part in inner.into_inner() {
                if let Rule::init_pair = part.as_rule() {
                    let mut pair_inner = part.into_inner();
                    let key = pair_inner.next().unwrap().as_str();
                    let val = pair_inner.next().unwrap();
                    match key {
                        "\"name\"" => name = Some(val.as_str().to_string()),
                        "\"datatype\"" => {
                            data_type = Some(match val.as_str() {
                                "string" => DataType::String,
                                "number" => DataType::Number,
                                "bool" => DataType::Bool,
                                _ => unreachable!(),
                            })
                        }
                        "\"value\"" => value = Some(build_expression(val)),
                        _ => {}
                    }
                }
            }
            Statement::SystemInit(VariableDeclaration {
                name: name.unwrap(),
                data_type: data_type.unwrap(),
                value,
            })
        }
        Rule::system_set => {
            let mut name = None;
            let mut value = None;
            for part in inner.into_inner() {
                if let Rule::set_pair = part.as_rule() {
                    let mut pair_inner = part.into_inner();
                    let key = pair_inner.next().unwrap().as_str();
                    let val = pair_inner.next().unwrap();
                    match key {
                        "\"name\"" => name = Some(val.as_str().to_string()),
                        "\"value\"" => value = Some(build_expression(val)),
                        _ => {}
                    }
                }
            }
            Statement::SystemSet(VariableAssignment {
                name: name.unwrap(),
                value: value.unwrap(),
            })
        }
        Rule::system_log => {
            let mut log_type = None;
            let mut message = None;
            for part in inner.into_inner() {
                if let Rule::log_pair = part.as_rule() {
                    let mut pair_inner = part.into_inner();
                    let key = pair_inner.next().unwrap();
                    let val = pair_inner.next().unwrap();
                    match key.as_str() {
                        "\"type\"" => log_type = Some(val.as_str().to_string()),
                        "\"message\"" => message = Some(build_expression(val)),
                        "arguments" => { /* ignore for now */ }
                        _ => {}
                    }
                }
            }
            Statement::SystemLog(Log {
                log_type: log_type.unwrap(),
                message: message.unwrap(),
            })
        }
        Rule::function_decl => {
            let mut inner_rules = inner.into_inner();
            let name = inner_rules.next().unwrap().as_str().to_string();
            let params_pair = inner_rules.next().unwrap();
            let body_pair = inner_rules.next().unwrap();

            let params = params_pair.into_inner().map(|param_pair| {
                let mut inner_param = param_pair.into_inner();
                let param_name = inner_param.next().unwrap().as_str().to_string();
                let param_type = match inner_param.next().unwrap().as_str() {
                    "string" => DataType::String,
                    "number" => DataType::Number,
                    "bool" => DataType::Bool,
                    _ => unreachable!()
                };
                (param_name, param_type)
            }).collect();

            let body = body_pair.into_inner().map(build_statement).collect();

            Statement::FunctionDeclaration(FunctionDeclaration {
                name,
                params,
                body,
            })
        }
        Rule::system_exec => {
            let mut name = None;
            let mut args = vec![];
            for part in inner.into_inner() {
                if let Rule::exec_pair = part.as_rule() {
                    let mut pair_inner = part.into_inner();
                    let key = pair_inner.next().unwrap();
                    let val = pair_inner.next().unwrap();
                    match key.as_str() {
                        "\"name\"" => name = Some(val.as_str().to_string()),
                        "parameters" => {
                            args = val.into_inner().map(|arg_pair| {
                                let mut inner_arg = arg_pair.into_inner();
                                let arg_name = inner_arg.next().unwrap().as_str().to_string();
                                let arg_val = build_expression(inner_arg.next().unwrap());
                                (arg_name, arg_val)
                            }).collect();
                        }
                        _ => {}
                    }
                }
            }
            Statement::SystemExec(FunctionCall {
                name: name.unwrap(),
                args,
            })
        }
        Rule::comment => panic!("Should be filtered out"),
        Rule::system_include => Statement::SystemInclude, // Placeholder
        _ => todo!("unhandled statement: {:?}", inner.as_rule()),
    }
}

fn build_expression(pair: Pair<Rule>) -> Expression {
    match pair.as_rule() {
        Rule::value => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::string => {
                    let inner_str = inner.into_inner().next().map_or("", |p| p.as_str());
                    Expression::Value(Value::String(inner_str.to_string()))
                }
                Rule::number => Expression::Value(Value::Number(inner.as_str().parse().unwrap())),
                Rule::boolean => Expression::Value(Value::Bool(inner.as_str().parse().unwrap())),
                _ => unreachable!(),
            }
        }
        Rule::argument => {
            Expression::Variable(pair.as_str().to_string())
        }
        Rule::expression => {
            let mut inner = pair.into_inner();
            let left = build_expression(inner.next().unwrap());
            if let Some(right) = inner.next() {
                 Expression::Concat(Box::new(left), Box::new(build_expression(right)))
            } else {
                left
            }
        }
        _ => build_expression(pair.into_inner().next().unwrap())
    }
}
