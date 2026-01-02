use std::path::{Path, PathBuf};

pub fn strip_all_extensions(path: &Path) -> PathBuf {
    let mut p = PathBuf::from(path);

    while p.extension().is_some() {
        if let Some(stem) = p.file_stem() {
            p = p.with_file_name(stem);
        } else {
            break;
        }
    }

    p
}
