pub mod parsing;

use crate::parsing::parsing::parse_repo;

pub fn scan_repo(repo_path: String) {
    parse_repo(repo_path);
}
