use anyhow::{Result, anyhow};
use log::error;
use std::fs;
use std::path::{Path, PathBuf};

pub fn to_absolute_path(relative_path: &str) -> Result<PathBuf> {
    let path = Path::new(relative_path);
    fs::canonicalize(path).map_err(|e| {
        anyhow!(
            "To absolute path failed,relative_path:{},error:{}",
            relative_path,
            e
        )
    })
}

pub fn check_and_move(file_path: &str, dest_path: &str) -> Result<()> {
    if !Path::new(file_path).exists() {
        return Err(anyhow!("Source file does not exist: {}", file_path));
    }
    if let Some(parent) = Path::new(dest_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(file_path, dest_path).or_else(|e| {
        error!("source:{},target:{}", file_path, dest_path);
        Err(e)
    })?;
    Ok(())
}

mod test {
    use super::*;
    use crate::utils::app_util;

    #[test]
    fn test_to_absolute_path() {
        println!(
            "Current dir: {}",
            std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .to_string()
        );

        let abs_path = to_absolute_path("./assets/model/all-minilm-l6-v2.onnx").unwrap();
        println!("Absolute path1: {}", abs_path.display()); // e.g., "/home/user/project/src/main.rs"

        let abs_path =
            to_absolute_path(&format!("{}/model.onnx", app_util::get_assets_tmp_path())).unwrap();
        println!("Absolute path2: {}", abs_path.display());

        let normalized = to_absolute_path("../target/debug").unwrap();
        println!("Canonical path3: {}", normalized.display()); // e.g., "/home/user/project/target/debug"

        assert!(abs_path.is_absolute());
        assert_eq!(abs_path.to_str().unwrap(), "/home/user/project/src/main.rs");

        let normalized = to_absolute_path("../target/debug").unwrap();
        assert!(normalized.is_absolute());
    }
}
