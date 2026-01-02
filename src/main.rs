use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use clap::{Parser as ClapParser, Subcommand};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use serde_json;

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
        file: String,
        #[arg(long)]
        log: bool,
    },
    /// Run a built Q file
    Run {
        /// The path to the Q file to run
        file: String,
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
            println!("Building file: {}", file);
            let content =
                fs::read_to_string(file).expect("Should have been able to read the file");

            let pairs = QParser::parse(Rule::file, &content).expect("Failed to parse");
            let ast = build_ast(pairs);

            let serialized_ast = serde_json::to_string(&ast).expect("Failed to serialize AST");
            let output_path = Path::new(file).with_extension("q.out");
            let mut output_file = File::create(&output_path).expect("Failed to create output file");
            output_file
                .write_all(serialized_ast.as_bytes())
                .expect("Failed to write to output file");

            println!("Successfully built to {}", output_path.display());

            if *log {
                println!("With logging enabled.");
            }
        }
        Commands::Run { file } => {
            let build_path = Path::new(file).with_extension("q.out");
            println!("Running build: {}", build_path.display());

            let content = fs::read_to_string(&build_path)
                .expect("Should have been able to read the build artifact");
            
            let ast: Vec<AstNode> = serde_json::from_str(&content).expect("Failed to deserialize AST");

            let mut interpreter = Interpreter::new();
            interpreter.interpret(&ast);
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

fn build_ast(mut pairs: pest::iterators::Pairs<Rule>) -> Vec<AstNode> {
    let file = pairs.next().unwrap();
    if file.as_rule() != Rule::file {
        return vec![];
    }

    file.into_inner()
        .filter_map(|pair| match pair.as_rule() {
            Rule::statement => build_statement(pair).map(AstNode::Statement),
            Rule::EOI => None,
            Rule::comment => None,
            _ => {
                println!("unhandled rule: {:?}", pair.as_rule());
                None
            }
        })
        .collect()
}

fn build_statement(pair: Pair<Rule>) -> Option<Statement> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::system_init => {
            let mut name = None;
            let mut data_type = None;
            let mut value = None;
            // Find init_pairs in the children
            let mut inner_iter = inner.into_inner();
            let init_pairs = inner_iter.find(|p| p.as_rule() == Rule::init_pairs);
            if let Some(init_pairs) = init_pairs {
                for part in init_pairs.into_inner() {
                    if part.as_rule() == Rule::init_pair {
                        // The init_pair only contains the value part (the key is consumed by the alternative match)
                        // We determine which alternative matched by looking at the child's rule type
                        let mut pair_inner = part.into_inner();
                        if let Some(val_pair) = pair_inner.next() {
                            match val_pair.as_rule() {
                                Rule::variable_type => {
                                    // This is the "type" alternative - ignore for now
                                }
                                Rule::identifier => {
                                    // This is the "name" alternative
                                    name = Some(val_pair.as_str().to_string());
                                }
                                Rule::datatype => {
                                    data_type = Some(match val_pair.as_str() {
                                        "string" => DataType::String,
                                        "number" => DataType::Number,
                                        "bool" => DataType::Bool,
                                        _ => unreachable!(),
                                    })
                                }
                                Rule::value => {
                                    value = Some(build_expression(val_pair));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Some(Statement::SystemInit(VariableDeclaration {
                name: name.unwrap(),
                data_type: data_type.unwrap(),
                value,
            }))
        }
        Rule::system_set => {
            let mut name = None;
            let mut value = None;
            let mut inner_iter = inner.into_inner();
            let set_pairs = inner_iter.find(|p| p.as_rule() == Rule::set_pairs);
            if let Some(set_pairs) = set_pairs {
                for part in set_pairs.into_inner() {
                    if part.as_rule() == Rule::set_pair {
                        let mut pair_inner = part.into_inner();
                        if let Some(val_pair) = pair_inner.next() {
                            match val_pair.as_rule() {
                                Rule::identifier => {
                                    name = Some(val_pair.as_str().to_string());
                                }
                                Rule::value => {
                                    value = Some(build_expression(val_pair));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Some(Statement::SystemSet(VariableAssignment {
                name: name.unwrap(),
                value: value.unwrap(),
            }))
        }
        Rule::system_log => {
            let mut log_type = None;
            let mut message = None;
            let mut inner_iter = inner.into_inner();
            let log_pairs = inner_iter.find(|p| p.as_rule() == Rule::log_pairs);
            if let Some(log_pairs) = log_pairs {
                for part in log_pairs.into_inner() {
                    if part.as_rule() == Rule::log_pair {
                        let mut pair_inner = part.into_inner();
                        if let Some(val_pair) = pair_inner.next() {
                            match val_pair.as_rule() {
                                Rule::log_type => {
                                    log_type = Some(val_pair.as_str().to_string());
                                }
                                Rule::expression => {
                                    message = Some(build_expression(val_pair));
                                }
                                Rule::arguments => {
                                    // Ignore arguments for now
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Some(Statement::SystemLog(Log {
                log_type: log_type.unwrap(),
                message: message.unwrap(),
            }))
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

            let body = body_pair.into_inner().filter_map(build_statement).collect();

            Some(Statement::FunctionDeclaration(FunctionDeclaration {
                name,
                params,
                body,
            }))
        }
        Rule::system_exec => {
            let mut name = None;
            let mut args = vec![];
            let mut inner_iter = inner.into_inner();
            let exec_pairs = inner_iter.find(|p| p.as_rule() == Rule::exec_pairs);
            if let Some(exec_pairs) = exec_pairs {
                for part in exec_pairs.into_inner() {
                    if part.as_rule() == Rule::exec_pair {
                        let mut pair_inner = part.into_inner();
                        if let Some(val_pair) = pair_inner.next() {
                            match val_pair.as_rule() {
                                Rule::identifier => {
                                    name = Some(val_pair.as_str().to_string());
                                }
                                Rule::exec_params => {
                                    args = val_pair.into_inner().map(|arg_pair| {
                                        let mut inner_arg = arg_pair.into_inner();
                                        let arg_name = inner_arg.next().unwrap().as_str().to_string();
                                        let arg_val = build_expression(inner_arg.next().unwrap());
                                        (arg_name, arg_val)
                                    }).collect();
                                }
                                Rule::exec_type => {
                                    // Ignore type for now
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Some(Statement::SystemExec(FunctionCall {
                name: name.unwrap(),
                args,
            }))
        }
        Rule::return_statement => {
            let inner = inner.into_inner().next().unwrap();
            Some(Statement::Return(build_expression(inner)))
        }
        Rule::comment => None,
        Rule::system_include => Some(Statement::SystemInclude), // Placeholder
        _ => todo!("unhandled statement: {:?}", inner.as_rule()),
    }
}

fn build_expression(pair: Pair<Rule>) -> Expression {
    match pair.as_rule() {
        Rule::value => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::string => {
                    let s = inner.as_str();
                    Expression::Value(Value::String(s[1..s.len() - 1].to_string()))
                }
                Rule::number => Expression::Value(Value::Number(inner.as_str().parse().unwrap())),
                Rule::boolean => Expression::Value(Value::Bool(inner.as_str().parse().unwrap())),
                Rule::null => Expression::Value(Value::Null),
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
