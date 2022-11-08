use std::path::PathBuf;

#[derive(Debug)]
pub struct BuilderPaths {
    backend_name: String,
    full: bool,
}

impl BuilderPaths {
    pub fn new(backend_name: String) -> Self {
        Self {
            backend_name,
            full: true,
        }
    }

    pub fn relative(&self) -> Self {
        BuilderPaths {
            backend_name: self.backend_name.clone(),
            full: false,
        }
    }

    pub fn root(&self) -> PathBuf {
        if self.full {
            PathBuf::from(format!(".builder_{}", self.backend_name))
        } else {
            PathBuf::new()
        }
    }

    pub fn root_as_string(&self) -> String {
        self.root().into_os_string().into_string().unwrap()
    }

    pub fn cargo_toml(&self) -> PathBuf {
        self.root().join("Cargo.toml")
    }

    pub fn src(&self) -> PathBuf {
        self.root().join("src")
    }

    pub fn wasm_build(&self, contract_name: &str) -> PathBuf {
        self.src().join(format!("{}_build.rs", contract_name))
    }

    pub fn wasm_build_as_string(&self, contract_name: &str) -> String {
        self.wasm_build(contract_name)
            .into_os_string()
            .into_string()
            .unwrap()
    }

    pub fn wasm_source(&self, contract_name: &str) -> PathBuf {
        self.src().join(format!("{}_wasm.rs", contract_name))
    }

    pub fn wasm_source_file(&self, contract_name: &str) -> String {
        self.wasm_source(contract_name)
            .into_os_string()
            .into_string()
            .unwrap()
    }
}

pub fn wasm_file_name(contract_name: &str) -> PathBuf {
    PathBuf::from(contract_name).with_extension("wasm")
}

pub fn wasm_path_in_target(contract_name: &str) -> PathBuf {
    PathBuf::from("target/wasm32-unknown-unknown/release").join(wasm_file_name(contract_name))
}

pub fn wasm_path_in_wasm_dir(contract_name: &str) -> PathBuf {
    PathBuf::from("wasm").join(wasm_file_name(contract_name))
}

pub fn project_dir() -> PathBuf {
    PathBuf::from(".")
}
