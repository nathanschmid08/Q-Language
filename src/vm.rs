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
    param_symbol_ids: Vec<u32>,
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
        
        // Functions will be registered when DeclareFunc instructions are executed
        
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
                Instruction::SetVarFromStack { symbol_id } => {
                    if let Some(value) = self.stack.pop() {
                        self.variables.insert(*symbol_id, value);
                    }
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
                    // Save current stack depth to isolate expression evaluation
                    let stack_depth_before = self.stack.len();
                    let expr_start = *message_expr_start as usize;
                    let expr_end = *message_expr_end as usize;
                    let log_type_clone = *log_type;
                    
                    // Execute expression to get message
                    let mut expr_pc = expr_start;
                    while expr_pc < expr_end {
                        self.execute_instruction_at(&mut expr_pc);
                    }
                    
                    // Pop the expression result (should be exactly one value)
                    let message = if self.stack.len() > stack_depth_before {
                        self.stack.pop().map(|v| value_to_string(&v)).unwrap_or_default()
                    } else {
                        String::new()
                    };
                    
                    // Ensure stack is clean after expression evaluation
                    while self.stack.len() > stack_depth_before {
                        self.stack.pop();
                    }
                    
                    let colored_type = match log_type_clone {
                        LogType::Info => "info".blue().bold(),
                        LogType::Warn => "warn".yellow().bold(),
                        LogType::Error => "error".red().bold(),
                    };
                    println!("[{}] {}", colored_type, message);
                    
                    pc += 1;
                }
                Instruction::DeclareFunc { symbol_id, param_count, param_symbol_ids, body_start, body_end } => {
                    self.functions.insert(*symbol_id, FunctionInfo {
                        param_count: *param_count,
                        param_symbol_ids: param_symbol_ids.clone(),
                        body_start: *body_start,
                        body_end: *body_end,
                    });
                    pc += 1;
                }
                Instruction::CallFunc { symbol_id, arg_count } => {
                    if let Some(func_info) = self.functions.get(symbol_id).cloned() {
                        // Pop arguments from stack (they should already be evaluated)
                        // Arguments are on stack in reverse order (last argument on top)
                        let mut args = Vec::new();
                        for _ in 0..*arg_count {
                            if let Some(arg) = self.stack.pop() {
                                args.push(arg);
                            }
                        }
                        // Reverse to get correct order (first argument first)
                        args.reverse();
                        
                        // Save current execution state
                        let saved_vars = self.variables.clone();
                        let saved_pc = pc;
                        
                        // Create isolated execution frame: set up function parameters
                        // Map arguments to parameter symbol IDs
                        for (i, arg_value) in args.iter().enumerate() {
                            if i < func_info.param_symbol_ids.len() {
                                let param_symbol_id = func_info.param_symbol_ids[i];
                                self.variables.insert(param_symbol_id, arg_value.clone());
                            }
                        }
                        
                        // Execute function body in isolated frame
                        let mut func_pc = func_info.body_start as usize;
                        while func_pc < func_info.body_end as usize {
                            // Check for return instruction
                            if let Instruction::Return = &self.program.instructions[func_pc] {
                                // Pop return value if any (currently not used)
                                if !self.stack.is_empty() {
                                    self.stack.pop();
                                }
                                func_pc += 1;
                                break;
                            }
                            // Execute instruction and advance PC
                            match &self.program.instructions[func_pc] {
                                Instruction::LoadValue { value } => {
                                    self.stack.push(value.clone());
                                    func_pc += 1;
                                }
                                Instruction::LoadVar { symbol_id } => {
                                    if let Some(val) = self.variables.get(symbol_id) {
                                        self.stack.push(val.clone());
                                    } else {
                                        self.stack.push(Value::Null);
                                    }
                                    func_pc += 1;
                                }
                                Instruction::Concat => {
                                    if self.stack.len() >= 2 {
                                        let right = self.stack.pop().unwrap();
                                        let left = self.stack.pop().unwrap();
                                        let result = Value::String(format!("{}{}", value_to_string(&left), value_to_string(&right)));
                                        self.stack.push(result);
                                    }
                                    func_pc += 1;
                                }
                                Instruction::SetVarFromStack { symbol_id } => {
                                    if let Some(value) = self.stack.pop() {
                                        self.variables.insert(*symbol_id, value);
                                    }
                                    func_pc += 1;
                                }
                                Instruction::InitVar { symbol_id, value } => {
                                    self.variables.insert(*symbol_id, value.clone());
                                    func_pc += 1;
                                }
                                Instruction::SetVar { symbol_id, value } => {
                                    self.variables.insert(*symbol_id, value.clone());
                                    func_pc += 1;
                                }
                                Instruction::Log { log_type, message_expr_start, message_expr_end } => {
                                    let stack_depth_before = self.stack.len();
                                    let expr_start = *message_expr_start as usize;
                                    let expr_end = *message_expr_end as usize;
                                    let log_type_clone = *log_type;
                                    
                                    // Evaluate expression by executing instructions in the expression range
                                    let mut expr_pc = expr_start;
                                    while expr_pc < expr_end && expr_pc < self.program.instructions.len() {
                                        match &self.program.instructions[expr_pc] {
                                            Instruction::LoadValue { value } => {
                                                self.stack.push(value.clone());
                                                expr_pc += 1;
                                            }
                                            Instruction::LoadVar { symbol_id } => {
                                                if let Some(val) = self.variables.get(symbol_id) {
                                                    self.stack.push(val.clone());
                                                } else {
                                                    self.stack.push(Value::Null);
                                                }
                                                expr_pc += 1;
                                            }
                                            Instruction::Concat => {
                                                if self.stack.len() >= 2 {
                                                    let right = self.stack.pop().unwrap();
                                                    let left = self.stack.pop().unwrap();
                                                    let result = Value::String(format!("{}{}", value_to_string(&left), value_to_string(&right)));
                                                    self.stack.push(result);
                                                }
                                                expr_pc += 1;
                                            }
                                            _ => {
                                                expr_pc += 1;
                                            }
                                        }
                                    }
                                    
                                    let message = if self.stack.len() > stack_depth_before {
                                        self.stack.pop().map(|v| value_to_string(&v)).unwrap_or_default()
                                    } else {
                                        String::new()
                                    };
                                    
                                    // Clean up stack
                                    while self.stack.len() > stack_depth_before {
                                        self.stack.pop();
                                    }
                                    
                                    let colored_type = match log_type_clone {
                                        LogType::Info => "info".blue().bold(),
                                        LogType::Warn => "warn".yellow().bold(),
                                        LogType::Error => "error".red().bold(),
                                    };
                                    println!("[{}] {}", colored_type, message);
                                    func_pc += 1;
                                }
                                Instruction::Return => {
                                    if !self.stack.is_empty() {
                                        self.stack.pop();
                                    }
                                    func_pc += 1;
                                    break;
                                }
                                _ => {
                                    func_pc += 1;
                                }
                            }
                        }
                        
                        // Restore previous execution state (isolated frame cleanup)
                        self.variables = saved_vars;
                        pc = saved_pc + 1;
                    } else {
                        pc += 1;
                    }
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
                } else {
                    self.stack.push(Value::Null);
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
            Instruction::SetVarFromStack { symbol_id } => {
                if let Some(value) = self.stack.pop() {
                    self.variables.insert(*symbol_id, value);
                }
            }
            Instruction::InitVar { symbol_id, value } => {
                self.variables.insert(*symbol_id, value.clone());
            }
            Instruction::SetVar { symbol_id, value } => {
                self.variables.insert(*symbol_id, value.clone());
            }
            Instruction::Return => {
                // Return handled at call site
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

