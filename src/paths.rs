//! Paths utils.

use std::path::PathBuf;

use convert_case::{Boundary, Case, Casing};

/// Helper struct that can produce builder releated paths.
/// By default all paths starts with .builder_*/.
pub struct BuilderPaths {
    backend_name: String,
    full: bool,
    project_root: PathBuf,
}

/// Implementation of BuilderPaths struct.
impl BuilderPaths {
    /// Creates a new BuilderPath for a given backend.
    pub fn new(backend_name: String, project_root: PathBuf) -> Self {
        Self {
            backend_name,
            project_root,
            full: true,
        }
    }

    /// Returns a new instance that produces paths without the .builder*/.
    pub fn relative(&self) -> Self {
        BuilderPaths {
            backend_name: self.backend_name.clone(),
            full: false,
            project_root: Default::default(),
        }
    }

    /// Returns root directory of the builder.
    pub fn root(&self) -> PathBuf {
        if self.full {
            self.project_root
                .join(PathBuf::from(format!(".builder_{}", self.backend_name)))
        } else {
            self.project_root.join(PathBuf::new())
        }
    }

    /// Returns Cargo.toml path of the builder.
    pub fn cargo_toml(&self) -> PathBuf {
        self.root().join("Cargo.toml")
    }

    /// Returns src directory path of the builder.
    pub fn src(&self) -> PathBuf {
        self.root().join("src")
    }

    /// Returns *_build.rs path.
    pub fn wasm_build(&self, contract_name: &str) -> PathBuf {
        self.src().join(format!("{contract_name}_build.rs"))
    }

    /// Returns *_build.rs path as a String.
    pub fn wasm_build_as_string(&self, contract_name: &str) -> String {
        self.wasm_build(contract_name)
            .into_os_string()
            .into_string()
            .unwrap()
    }

    /// Returns *_wasm.rs path.
    pub fn wasm_source(&self, contract_name: &str) -> PathBuf {
        self.src().join(format!("{contract_name}_wasm.rs"))
    }

    /// Returns *_wasm.rs path as a String.
    pub fn wasm_source_as_string(&self, contract_name: &str) -> String {
        self.wasm_source(contract_name)
            .into_os_string()
            .into_string()
            .unwrap()
    }
}

/// Returns *.wasm filename.
pub fn wasm_file_name(contract_name: &str) -> PathBuf {
    PathBuf::from(contract_name).with_extension("wasm")
}

/// Returns *.wasm file path in target directory.
pub fn wasm_path_in_target(contract_name: &str, project_root: PathBuf) -> PathBuf {
    project_root
        .join("target/wasm32-unknown-unknown/release")
        .join(wasm_file_name(contract_name))
}

/// Returns *.wasm file path in wasm directory.
pub fn wasm_path_in_wasm_dir(contract_name: &str, project_root: PathBuf) -> PathBuf {
    wasm_dir(project_root).join(wasm_file_name(contract_name))
}

/// Returns wasm directory path.
pub fn wasm_dir(project_root: PathBuf) -> PathBuf {
    project_root.join("wasm")
}

/// Convert text to a sneak case.
pub fn to_snake_case<T: AsRef<str>>(text: T) -> String {
    text.as_ref()
        .with_boundaries(&Boundary::defaults())
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}
