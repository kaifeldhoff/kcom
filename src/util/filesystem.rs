use std::env;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

pub fn get_current_dir() -> Result<PathBuf, failure::Error> {
    Ok(env::current_dir()?)
}

type Tgdc = Vec<String>;
pub fn get_dir_content(dir: &PathBuf) -> Result<(Tgdc, Tgdc, Tgdc), failure::Error> {
    let mut subdirs = vec![];
    let mut files = vec![];
    let symlinks = vec![];
    for entry_result in fs::read_dir(dir)? {
        let entry = entry_result?;
        let raw_filename = entry.file_name();
        let filename: String = String::from_utf8_lossy(&raw_filename.into_vec()).into();
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            subdirs.push(filename)
        } else if filetype.is_file() {
            files.push(filename)
        } else if filetype.is_symlink() {
            if entry.path().exists() {
                if entry.path().metadata()?.is_dir() {
                    subdirs.push(filename)
                } else {
                    files.push(filename)
                }
            }
        }
    }
    Ok((subdirs, files, symlinks))
}
