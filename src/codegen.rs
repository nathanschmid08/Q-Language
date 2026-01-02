use std::fs::File;
use std::io::Write;
use crate::ir::*;
use crate::build::BYTECODE_VERSION;
use bincode;

/// Generate binary bytecode from IR
/// This is the only stage that should know about binary format
/// It operates exclusively on IR, not on AST or parser structures
pub fn emit_bytecode(program: &Program, output_path: &std::path::Path) -> std::io::Result<usize> {
    // Create bytecode structure
    let bytecode = Bytecode {
        version: BYTECODE_VERSION,
        program: program.clone(),
    };
    
    // Serialize to binary
    let binary_data = bincode::serialize(&bytecode)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Serialization error: {}", e)))?;
    
    // Write to file
    let mut file = File::create(output_path)?;
    file.write_all(&binary_data)?;
    
    Ok(binary_data.len())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Bytecode {
    version: u32,
    program: Program,
}

/// Load bytecode from file
pub fn load_bytecode(input_path: &std::path::Path) -> std::io::Result<Program> {
    use std::io::Read;
    
    let mut file = File::open(input_path)?;
    let mut binary_data = Vec::new();
    file.read_to_end(&mut binary_data)?;
    
    let bytecode: Bytecode = bincode::deserialize(&binary_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Deserialization error: {}", e)))?;
    
    // Validate version
    if bytecode.version != BYTECODE_VERSION {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Bytecode version mismatch: expected {}, got {}", BYTECODE_VERSION, bytecode.version),
        ));
    }
    
    Ok(bytecode.program)
}

