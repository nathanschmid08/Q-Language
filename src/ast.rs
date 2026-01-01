
#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Statement(Statement),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    SystemInclude, // Placeholder
    SystemInit(VariableDeclaration),
    SystemSet(VariableAssignment),
    SystemLog(Log),
    FunctionDeclaration(FunctionDeclaration),
    SystemExec(FunctionCall),
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclaration {
    pub name: String,
    pub data_type: DataType,
    pub value: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableAssignment {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Log {
    pub log_type: String,
    pub message: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<(String, DataType)>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<(String, Expression)>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Value(Value),
    Variable(String),
    Concat(Box<Expression>, Box<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    String,
    Number,
    Bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}
