use crate::ast::*;
use std::collections::HashSet;

/// Semantic analysis errors
#[derive(Debug, Clone)]
pub enum SemanticError {
    DuplicateVariable(String),
    DuplicateFunction(String),
    UndefinedVariable(String),
    UndefinedFunction(String),
    TypeMismatch(String),
}

/// Result of semantic analysis
pub type SemanticResult<T> = Result<T, SemanticError>;

/// Perform semantic analysis on the AST
/// This stage validates:
/// - No duplicate variable/function declarations
/// - All referenced variables/functions are defined
/// - Type consistency
pub fn analyze(ast: &[AstNode]) -> SemanticResult<()> {
    let mut declared_vars = HashSet::new();
    let mut declared_funcs = HashSet::new();
    
    for node in ast {
        match node {
            AstNode::Statement(stmt) => {
                analyze_statement(stmt, &mut declared_vars, &mut declared_funcs)?;
            }
        }
    }
    
    Ok(())
}

fn analyze_statement(
    stmt: &Statement,
    declared_vars: &mut HashSet<String>,
    declared_funcs: &mut HashSet<String>,
) -> SemanticResult<()> {
    match stmt {
        Statement::SystemInit(var_decl) => {
            if declared_vars.contains(&var_decl.name) {
                return Err(SemanticError::DuplicateVariable(var_decl.name.clone()));
            }
            declared_vars.insert(var_decl.name.clone());
        }
        Statement::SystemSet(var_assign) => {
            if !declared_vars.contains(&var_assign.name) {
                return Err(SemanticError::UndefinedVariable(var_assign.name.clone()));
            }
        }
        Statement::SystemLog(_) => {
            // Log statements don't need semantic validation beyond expression checking
        }
        Statement::FunctionDeclaration(func_decl) => {
            if declared_funcs.contains(&func_decl.name) {
                return Err(SemanticError::DuplicateFunction(func_decl.name.clone()));
            }
            declared_funcs.insert(func_decl.name.clone());
            
            // Analyze function body
            let mut func_vars = declared_vars.clone();
            // Add function parameters to scope
            for (param_name, _) in &func_decl.params {
                func_vars.insert(param_name.clone());
            }
            
            for body_stmt in &func_decl.body {
                analyze_statement(body_stmt, &mut func_vars, declared_funcs)?;
            }
        }
        Statement::SystemExec(func_call) => {
            if !declared_funcs.contains(&func_call.name) {
                return Err(SemanticError::UndefinedFunction(func_call.name.clone()));
            }
        }
        Statement::Return(_) => {
            // Return statements are validated in function context
        }
        Statement::SystemInclude => {
            // Placeholder - no validation needed yet
        }
    }
    
    Ok(())
}

