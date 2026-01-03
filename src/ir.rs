use serde::{Serialize, Deserialize};
use crate::ast::*;

/// Intermediate Representation - a lower-level representation
/// that is independent of the source syntax and suitable for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub string_table: Vec<String>,
    pub symbol_table: Vec<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // Variable operations
    InitVar { symbol_id: u32, value: Value },
    SetVar { symbol_id: u32, value: Value }, // For compile-time constants
    SetVarFromStack { symbol_id: u32 }, // For runtime expressions
    
    // Function operations
    DeclareFunc { symbol_id: u32, param_count: u32, param_symbol_ids: Vec<u32>, body_start: u32, body_end: u32 },
    CallFunc { symbol_id: u32, arg_count: u32 },
    
    // Expression operations
    LoadValue { value: Value },
    LoadVar { symbol_id: u32 },
    Concat,
    
    // System operations
    Log { log_type: LogType, message_expr_start: u32, message_expr_end: u32 },
    
    // Control flow
    Return,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogType {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: u32,
    pub name: String,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    Variable { data_type: DataType },
    Function { param_types: Vec<DataType>, return_type: Option<DataType> },
}

/// Convert AST to IR
pub fn ast_to_ir(ast: &[AstNode]) -> Program {
    let mut instructions = Vec::new();
    let string_table = Vec::new();
    let mut symbol_table = Vec::new();
    let mut symbol_counter = 0u32;
    
    // First pass: collect all symbols
    let mut symbol_map = std::collections::HashMap::new();
    
    for node in ast {
        match node {
            AstNode::Statement(stmt) => {
                match stmt {
                    Statement::SystemInit(var_decl) => {
                        let symbol_id = symbol_counter;
                        symbol_counter += 1;
                        symbol_map.insert(var_decl.name.clone(), symbol_id);
                        symbol_table.push(Symbol {
                            id: symbol_id,
                            name: var_decl.name.clone(),
                            kind: SymbolKind::Variable {
                                data_type: var_decl.data_type.clone(),
                            },
                        });
                    }
                    Statement::FunctionDeclaration(func_decl) => {
                        let symbol_id = symbol_counter;
                        symbol_counter += 1;
                        symbol_map.insert(func_decl.name.clone(), symbol_id);
                        let param_types: Vec<DataType> = func_decl.params.iter().map(|(_, dt)| dt.clone()).collect();
                        symbol_table.push(Symbol {
                            id: symbol_id,
                            name: func_decl.name.clone(),
                            kind: SymbolKind::Function {
                                param_types,
                                return_type: None, // TODO: infer return type
                            },
                        });
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Second pass: generate instructions
    for node in ast {
        match node {
            AstNode::Statement(stmt) => {
                match stmt {
                    Statement::SystemInit(var_decl) => {
                        let symbol_id = symbol_map[&var_decl.name];
                        let value = var_decl.value.as_ref()
                            .map(|expr| expression_to_value(expr))
                            .unwrap_or(Value::Null);
                        instructions.push(Instruction::InitVar { symbol_id, value });
                    }
                    Statement::SystemSet(var_assign) => {
                        let symbol_id = symbol_map[&var_assign.name];
                        // Evaluate expression and leave result on stack
                        expression_to_instructions(&var_assign.value, &mut instructions, &symbol_map);
                        instructions.push(Instruction::SetVarFromStack { symbol_id });
                    }
                    Statement::SystemLog(log) => {
                        let log_type = match log.log_type.to_lowercase().as_str() {
                            "info" => LogType::Info,
                            "warn" => LogType::Warn,
                            "error" => LogType::Error,
                            _ => LogType::Info,
                        };
                        let expr_start = instructions.len() as u32;
                        expression_to_instructions(&log.message, &mut instructions, &symbol_map);
                        let expr_end = instructions.len() as u32;
                        instructions.push(Instruction::Log { log_type, message_expr_start: expr_start, message_expr_end: expr_end });
                    }
                    Statement::FunctionDeclaration(func_decl) => {
                        let symbol_id = symbol_map[&func_decl.name];
                        
                        // Create symbol IDs for function parameters (they need their own scope)
                        let mut param_symbol_ids = Vec::new();
                        let mut func_symbol_map = symbol_map.clone();
                        for (param_name, _) in &func_decl.params {
                            let param_symbol_id = symbol_counter;
                            symbol_counter += 1;
                            param_symbol_ids.push(param_symbol_id);
                            func_symbol_map.insert(param_name.clone(), param_symbol_id);
                            
                            // Add to symbol table
                            symbol_table.push(Symbol {
                                id: param_symbol_id,
                                name: param_name.clone(),
                                kind: SymbolKind::Variable {
                                    data_type: func_decl.params.iter()
                                        .find(|(name, _)| name == param_name)
                                        .map(|(_, dt)| dt.clone())
                                        .unwrap_or(DataType::String),
                                },
                            });
                        }
                        
                        let body_start = instructions.len() as u32;
                        for body_stmt in &func_decl.body {
                            statement_to_instructions(body_stmt, &mut instructions, &func_symbol_map);
                        }
                        let body_end = instructions.len() as u32;
                        let param_count = func_decl.params.len() as u32;
                        instructions.push(Instruction::DeclareFunc { 
                            symbol_id, 
                            param_count, 
                            param_symbol_ids: param_symbol_ids.clone(),
                            body_start, 
                            body_end 
                        });
                    }
                    Statement::SystemExec(func_call) => {
                        let symbol_id = symbol_map[&func_call.name];
                        let arg_count = func_call.args.len() as u32;
                        for (_, arg_expr) in &func_call.args {
                            expression_to_instructions(arg_expr, &mut instructions, &symbol_map);
                        }
                        instructions.push(Instruction::CallFunc { symbol_id, arg_count });
                    }
                    Statement::Return(expr) => {
                        expression_to_instructions(expr, &mut instructions, &symbol_map);
                        instructions.push(Instruction::Return);
                    }
                    Statement::SystemInclude => {
                        // Placeholder
                    }
                }
            }
        }
    }
    
    Program {
        instructions,
        string_table,
        symbol_table,
    }
}

fn expression_to_value(expr: &Expression) -> Value {
    match expr {
        Expression::Value(val) => val.clone(),
        Expression::Variable(_) => Value::Null, // Can't resolve at compile time
        Expression::Concat(_, _) => Value::Null, // Can't resolve at compile time
    }
}

fn expression_to_instructions(
    expr: &Expression,
    instructions: &mut Vec<Instruction>,
    symbol_map: &std::collections::HashMap<String, u32>,
) {
    match expr {
        Expression::Value(val) => {
            instructions.push(Instruction::LoadValue { value: val.clone() });
        }
        Expression::Variable(var_name) => {
            let parts: Vec<&str> = var_name.split('.').collect();
            let name = parts[0];
            if let Some(&symbol_id) = symbol_map.get(name) {
                instructions.push(Instruction::LoadVar { symbol_id });
            } else {
                // Variable not found - push null as fallback
                instructions.push(Instruction::LoadValue { value: Value::Null });
            }
        }
        Expression::Concat(left, right) => {
            expression_to_instructions(left, instructions, symbol_map);
            expression_to_instructions(right, instructions, symbol_map);
            instructions.push(Instruction::Concat);
        }
    }
}

fn statement_to_instructions(
    stmt: &Statement,
    instructions: &mut Vec<Instruction>,
    symbol_map: &std::collections::HashMap<String, u32>,
) {
    match stmt {
        Statement::SystemInit(var_decl) => {
            let symbol_id = symbol_map[&var_decl.name];
            let value = var_decl.value.as_ref()
                .map(|expr| expression_to_value(expr))
                .unwrap_or(Value::Null);
            instructions.push(Instruction::InitVar { symbol_id, value });
        }
        Statement::SystemSet(var_assign) => {
            let symbol_id = symbol_map[&var_assign.name];
            // Evaluate expression and leave result on stack
            expression_to_instructions(&var_assign.value, instructions, symbol_map);
            instructions.push(Instruction::SetVarFromStack { symbol_id });
        }
        Statement::SystemLog(log) => {
            let log_type = match log.log_type.to_lowercase().as_str() {
                "info" => LogType::Info,
                "warn" => LogType::Warn,
                "error" => LogType::Error,
                _ => LogType::Info,
            };
            let expr_start = instructions.len() as u32;
            expression_to_instructions(&log.message, instructions, symbol_map);
            let expr_end = instructions.len() as u32;
            instructions.push(Instruction::Log { log_type, message_expr_start: expr_start, message_expr_end: expr_end });
        }
        Statement::Return(expr) => {
            expression_to_instructions(expr, instructions, symbol_map);
            instructions.push(Instruction::Return);
        }
        _ => {
            // Other statements handled at program level
        }
    }
}

