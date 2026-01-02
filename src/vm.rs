use crate::ir::*;
use crate::ast::*;
use std::collections::HashMap;
use colored::*;

/// Virtual Machine for executing IR bytecode
pub struct VM {
    variables: HashMap<u32, Value>,
    functions: HashMap<u32, FunctionInfo>,
    stack: Vec<Value>,
    program: Program,
}

#[derive(Clone)]
struct FunctionInfo {
    param_count: u32,
    body_start: u32,
    body_end: u32,
}

impl VM {
    pub fn new(program: Program) -> Self {
        let mut vm = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            stack: Vec::new(),
            program,
        };
        
        // Register functions from symbol table
        for symbol in &vm.program.symbol_table {
            if let SymbolKind::Function { param_types, return_type: _ } = &symbol.kind {
                vm.functions.insert(symbol.id, FunctionInfo {
                    param_count: param_types.len() as u32,
                    body_start: 0, // Will be set from instructions
                    body_end: 0,
                });
            }
        }
        
        vm
    }

    pub fn execute(&mut self) {
        let mut pc = 0;
        while pc < self.program.instructions.len() {
            match &self.program.instructions[pc] {
                Instruction::InitVar { symbol_id, value } => {
                    self.variables.insert(*symbol_id, value.clone());
                    pc += 1;
                }
                Instruction::SetVar { symbol_id, value } => {
                    self.variables.insert(*symbol_id, value.clone());
                    pc += 1;
                }
                Instruction::LoadValue { value } => {
                    self.stack.push(value.clone());
                    pc += 1;
                }
                Instruction::LoadVar { symbol_id } => {
                    if let Some(val) = self.variables.get(symbol_id) {
                        self.stack.push(val.clone());
                    }
                    pc += 1;
                }
                Instruction::Concat => {
                    if self.stack.len() >= 2 {
                        let right = self.stack.pop().unwrap();
                        let left = self.stack.pop().unwrap();
                        let result = Value::String(format!("{}{}", value_to_string(&left), value_to_string(&right)));
                        self.stack.push(result);
                    }
                    pc += 1;
                }
                Instruction::Log { log_type, message_expr_start, message_expr_end } => {
                    // Execute expression to get message
                    let saved_pc = pc;
                    let expr_start = *message_expr_start as usize;
                    let expr_end = *message_expr_end as usize;
                    let log_type_clone = log_type.clone();
                    
                    let mut expr_pc = expr_start;
                    while expr_pc < expr_end {
                        self.execute_instruction_at(&mut expr_pc);
                    }
                    let message = if let Some(val) = self.stack.pop() {
                        value_to_string(&val)
                    } else {
                        String::new()
                    };
                    
                    let colored_type = match log_type_clone {
                        LogType::Info => "info".blue().bold(),
                        LogType::Warn => "warn".yellow().bold(),
                        LogType::Error => "error".red().bold(),
                    };
                    println!("[{}] {}", colored_type, message);
                    
                    pc = saved_pc + 1;
                }
                Instruction::DeclareFunc { symbol_id, param_count, body_start, body_end } => {
                    self.functions.insert(*symbol_id, FunctionInfo {
                        param_count: *param_count,
                        body_start: *body_start,
                        body_end: *body_end,
                    });
                    pc += 1;
                }
                Instruction::CallFunc { symbol_id, arg_count } => {
                    if let Some(func_info) = self.functions.get(symbol_id).cloned() {
                        // Pop arguments from stack
                        let mut args = Vec::new();
                        for _ in 0..*arg_count {
                            if let Some(arg) = self.stack.pop() {
                                args.push(arg);
                            }
                        }
                        args.reverse();
                        
                        // Save current state
                        let saved_vars = self.variables.clone();
                        
                        // Set up function parameters
                        // TODO: Map parameters to symbol IDs
                        
                        // Execute function body
                        let saved_pc = pc;
                        let mut func_pc = func_info.body_start as usize;
                        while func_pc < func_info.body_end as usize {
                            self.execute_instruction_at(&mut func_pc);
                        }
                        pc = saved_pc + 1;
                        
                        // Restore variables
                        self.variables = saved_vars;
                    }
                    pc += 1;
                }
                Instruction::Return => {
                    // Return from function
                    pc += 1;
                }
            }
        }
    }

    fn execute_instruction_at(&mut self, pc: &mut usize) {
        if *pc >= self.program.instructions.len() {
            return;
        }
        
        match &self.program.instructions[*pc] {
            Instruction::LoadValue { value } => {
                self.stack.push(value.clone());
            }
            Instruction::LoadVar { symbol_id } => {
                if let Some(val) = self.variables.get(symbol_id) {
                    self.stack.push(val.clone());
                }
            }
            Instruction::Concat => {
                if self.stack.len() >= 2 {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    let result = Value::String(format!("{}{}", value_to_string(&left), value_to_string(&right)));
                    self.stack.push(result);
                }
            }
            _ => {
                // Other instructions handled at top level
            }
        }
        *pc += 1;
    }
}

fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
    }
}

