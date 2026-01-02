use std::fs;
use std::path::{Path, PathBuf};
use serde_json;

pub const BUILD_DIR: &str = "build";
pub const BYTECODE_VERSION: u32 = 1;
pub const COMPILER_VERSION: &str = "0.1.0";

/// Represents the build output structure for a compiled package
pub struct PackageBuilder {
    package_dir: PathBuf,
    source_file: String,
}

impl PackageBuilder {
    /// Create a new package builder for a source file
    pub fn new(source_file: &Path) -> Self {
        let source_name = source_file
            .file_stem()
            .expect("Source file must have a name")
            .to_string_lossy()
            .to_string();
        
        let package_name = format!("{}.qpkg", source_name);
        let package_dir = PathBuf::from(BUILD_DIR).join(&package_name);
        
        Self {
            package_dir,
            source_file: source_file
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        }
    }

    /// Create the package directory structure
    pub fn create(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.package_dir)?;
        Ok(())
    }

    /// Get the path to the bytecode file
    pub fn bytecode_path(&self) -> PathBuf {
        self.package_dir.join("program.qbin")
    }

    /// Get the path to the manifest file
    pub fn manifest_path(&self) -> PathBuf {
        self.package_dir.join("manifest.json")
    }

    /// Write the manifest file with metadata
    pub fn write_manifest(&self, bytecode_size: usize) -> std::io::Result<()> {
        let manifest = serde_json::json!({
            "source_file": self.source_file,
            "compiler_version": COMPILER_VERSION,
            "bytecode_version": BYTECODE_VERSION,
            "bytecode_size": bytecode_size,
        });

        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(self.manifest_path(), manifest_json)?;
        Ok(())
    }

    /// Get the package directory path
    pub fn package_dir(&self) -> &Path {
        &self.package_dir
    }

    /// Get the source file name
    pub fn source_file(&self) -> &str {
        &self.source_file
    }
}

/// Load a package for execution
pub fn load_package(source_file: &Path) -> std::io::Result<PathBuf> {
    let source_name = source_file
        .file_stem()
        .expect("Source file must have a name")
        .to_string_lossy()
        .to_string();
    
    let package_name = format!("{}.qpkg", source_name);
    let package_dir = PathBuf::from(BUILD_DIR).join(&package_name);
    
    if !package_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Package not found: {}", package_dir.display()),
        ));
    }
    
    Ok(package_dir.join("program.qbin"))
}

