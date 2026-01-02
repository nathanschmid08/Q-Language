use std::fs;
use std::path::Path;

use clap::{Parser as ClapParser, Subcommand};

mod ast;
mod parser;
mod semantic;
mod ir;
mod codegen;
mod build;
mod vm;

use parser::*;
use semantic::*;
use ir::*;
use codegen::*;
use build::*;
use vm::*;

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
            compile_file(file, *log);
        }
        Commands::Run { file } => {
            run_file(file);
        }
        Commands::Clear { name } => {
            if let Some(name) = name {
                println!("Clearing cache: {}", name);
            } else {
                println!("Clearing all caches...");
                if Path::new(BUILD_DIR).exists() {
                    fs::remove_dir_all(BUILD_DIR).expect("Failed to clear build directory");
                    println!("Build directory cleared.");
                }
            }
        }
    }
}

/// Compilation pipeline: source -> parse -> AST -> semantic -> IR -> bytecode
fn compile_file(source_file: &str, _log: bool) {
    println!("Building file: {}", source_file);
    let input_path = Path::new(source_file);
    
    // Stage 1: Lexical Analysis & Parsing
    let source = fs::read_to_string(source_file)
        .expect("Should have been able to read the file");
    
    let parse_tree = parse_source(&source)
        .expect("Failed to parse source");
    
    // Stage 2: AST Construction
    let ast = build_ast(parse_tree);
    
    // Stage 3: Semantic Analysis
    analyze(&ast)
        .expect("Semantic analysis failed");
    
    // Stage 4: IR Generation
    let ir = ast_to_ir(&ast);
    
    // Stage 5: Binary Emission
    let package = PackageBuilder::new(input_path);
    package.create()
        .expect("Failed to create package directory");
    
    let bytecode_size = emit_bytecode(&ir, &package.bytecode_path())
        .expect("Failed to emit bytecode");
    
    package.write_manifest(bytecode_size)
        .expect("Failed to write manifest");
    
    println!("Successfully built to {}", package.package_dir().display());
}

/// Execution pipeline: load bytecode -> execute in VM
fn run_file(source_file: &str) {
    let input_path = Path::new(source_file);
    let bytecode_path = load_package(input_path)
        .expect("Failed to load package");
    
    println!("Running build: {}", bytecode_path.parent().unwrap().display());
    
    // Load bytecode
    let program = load_bytecode(&bytecode_path)
        .expect("Failed to load bytecode");
    
    // Execute in VM
    let mut vm = VM::new(program);
    vm.execute();
}
