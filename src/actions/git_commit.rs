use gix::{prelude::*, Repository};

const MESSAGE: &str = "cargo-frc automated commit";

/// commit all files in the repository.
// pub fn make_commit(repo: &Repository, comp_acronymn: Option<String>) {
//     //get entry for every changed file
//     let mut index = repo.index().unwrap();
//     let mut changed_files = Vec::new();
//     for entry in index.entries().iter() {
//         if entry.stage() != 0 {
//             changed_files.push(entry.path().unwrap().to_str().unwrap().to_string());
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use gix::{Repository, index::entry};

    #[test]
    fn test_make_commit() {
        let cwd = std::env::current_dir().unwrap();
        println!("cwd: {:?}", cwd);
        let repo = gix::discover(cwd).unwrap();
        // for entry in repo.
    }
}