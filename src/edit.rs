use std::path::PathBuf;

use log::trace;

use crate::config::Config;

fn get_path(base_path: &str, path: &str) -> PathBuf {
    let mut file_path = PathBuf::from(base_path);

    let path_string: String = path
        .chars()
        .map(|chr| {
            if chr.is_whitespace() {
                return '_';
            }
            chr.to_ascii_lowercase()
        })
        .collect();

    trace!("{}", path_string);

    file_path.push(path_string);
    file_path.set_extension("md");

    trace!("{:?}", file_path);

    file_path
}

pub fn edit_note(config: &Config, path: &str) {
    trace!("{}", path);

    let _file_path = get_path(&config.note_path, path);
}
