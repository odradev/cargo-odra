use crate::odra_toml::OdraToml;
use std::path::PathBuf;
use crate::cargo_toml::{load_main_cargo_toml, members};

#[derive(Debug)]
pub struct Project {
    /// Root directory of the project.
    pub root: PathBuf,
    /// Path to the main Cargo.toml file.
    pub cargo_toml: PathBuf,
    /// Path to the Odra.toml file.
    pub odra_toml: PathBuf,
    /// Members of the project.
    pub members: Vec<Member>,
}
impl Project {
    pub fn detect() -> Project {
        let odra_toml_path = Self::find_odra_toml().unwrap();
        let cargo_toml_path = Self::find_cargo_toml().unwrap();
        let root = odra_toml_path.parent().unwrap().to_path_buf();
        let members = Self::find_members(cargo_toml_path.clone());
        Project {
            root,
            cargo_toml: cargo_toml_path,
            odra_toml: odra_toml_path,
            members,
        }
    }
    pub fn find_members(cargo_toml_path: PathBuf) -> Vec<Member> {
        members()
            .iter()
            .map(|member| {
                let root = cargo_toml_path.parent().unwrap().join(member.clone().1);
                let cargo_toml = root.join("Cargo.toml");
                Member { name: member.clone().0, root, cargo_toml }
            })
            .collect()
    }
    pub fn find_odra_toml() -> Option<PathBuf> {
        Self::find_file_upwards("Odra.toml")
    }

    pub fn find_cargo_toml() -> Option<PathBuf> {
        match Self::find_file_upwards("Odra.toml") {
            None => None,
            Some(odra_toml_path) => {
                let cargo_toml_path = Some(odra_toml_path.parent().unwrap().join("Cargo.toml"));
                if cargo_toml_path.as_ref().unwrap().exists() {
                    cargo_toml_path
                } else {
                    None
                }
            }
        }
    }

    fn find_file_upwards(filename: &str) -> Option<PathBuf> {
        let mut path = std::env::current_dir().unwrap();
        loop {
            let file_path = path.join(filename);
            if file_path.exists() {
                return Some(file_path);
            }
            if path == path.parent().unwrap() {
                return None;
            }
            path = path.parent().unwrap().to_path_buf();
        }
    }
}
#[derive(Debug)]
pub struct Member {
    /// Name of the member.
    pub name: String,
    /// Root directory of the member.
    pub root: PathBuf,
    /// Path to the Cargo.toml file.
    pub cargo_toml: PathBuf,
}

impl Member {


}
