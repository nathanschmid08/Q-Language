use std::collections::HashMap;
use crate::ast::*;

pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, FunctionDeclaration>,
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, ast: &[AstNode]) {
        for node in ast {
            let AstNode::Statement(stmt) = node;
            self.execute_statement(stmt);
        }
    }

    fn execute_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::SystemInit(var_decl) => {
                let value = var_decl.value.as_ref().map_or(Value::Null, |v| self.evaluate_expression(v));
                self.variables.insert(var_decl.name.clone(), value);
            }
            Statement::SystemSet(var_assign) => {
                let value = self.evaluate_expression(&var_assign.value);
                if self.variables.contains_key(&var_assign.name) {
                    self.variables.insert(var_assign.name.clone(), value);
                } else {
                    panic!("Variable '{}' not declared", var_assign.name);
                }
            }
            Statement::SystemLog(log) => {
                let message = self.evaluate_expression(&log.message);
                println!("[{}] {}", log.log_type, message.to_string());
            }
            Statement::FunctionDeclaration(func_decl) => {
                self.functions.insert(func_decl.name.clone(), func_decl.clone());
            }
            Statement::SystemExec(func_call) => {
                self.execute_function_call(func_call);
            }
            Statement::Return(_) => { /* Not implemented */ }
            Statement::SystemInclude => { /* Not implemented */ }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::Value(val) => val.clone(),
            Expression::Variable(var_name) => {
                let parts: Vec<&str> = var_name.split('.').collect();
                let name = parts[0];
                
                if let Some(val) = self.variables.get(name) {
                    val.clone()
                } else {
                    panic!("Variable '{}' not found", name);
                }
            }
            Expression::Concat(left, right) => {
                let left_val = self.evaluate_expression(left);
                let right_val = self.evaluate_expression(right);
                Value::String(format!("{}{}", left_val.to_string(), right_val.to_string()))
            }
        }
    }

    fn execute_function_call(&mut self, func_call: &FunctionCall) {
        if let Some(func_decl) = self.functions.get(&func_call.name).cloned() {
            let mut local_scope = self.variables.clone();
            
            for (param_name, arg_expr) in func_call.args.iter() {
                let arg_value = self.evaluate_expression(arg_expr);
                local_scope.insert(param_name.clone(), arg_value);
            }

            let original_vars = self.variables.clone();
            self.variables = local_scope;

            for stmt in &func_decl.body {
                self.execute_statement(stmt);
            }

            self.variables = original_vars;

        } else {
            panic!("Function '{}' not found", func_call.name);
        }
    }
}
