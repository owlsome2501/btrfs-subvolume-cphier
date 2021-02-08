use std::collections::HashMap;
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Debug)]
pub enum FileType {
    Directory,
    Subvolume,
}

pub struct Subvolume {
    pub path: PathBuf,
    pub abs_path: String,
    pub hierachy: Option<Vec<(String, FileType)>>,
}

impl Subvolume {
    pub fn from(path: &Path) -> Result<Self, String> {
        // get absolute path against btrfs filesystem top level
        // meanwhile, check whether the path is a valid btrfs subvolume
        let abs_path = Subvolume::_btrfs_subvolume_show_abs_path(path)?;
        Ok(Subvolume {
            path: path.to_owned(),
            abs_path,
            hierachy: None,
        })
    }

    pub fn create(path: &Path, hierachy: &Vec<(String, FileType)>) -> Result<Self, String> {
        if let Err(_) = Subvolume::_btrfs_subvolume_show_abs_path(path) {
            Subvolume::_btrfs_subvolume_create(path)?;
        }
        // check whether the subvolume is created successfully
        let abs_path = Subvolume::_btrfs_subvolume_show_abs_path(path)?;
        for (child_rela_path, ft) in hierachy {
            let child_path = path.join(child_rela_path);
            match ft {
                FileType::Directory => {
                    if let Err(e) = std::fs::create_dir(child_path) {
                        match e.kind() {
                            io::ErrorKind::AlreadyExists => (),
                            _ => return Err(e.to_string()),
                        }
                    }
                }
                FileType::Subvolume => {
                    if let Err(_) = Subvolume::_btrfs_subvolume_show_abs_path(&child_path) {
                        Subvolume::_btrfs_subvolume_create(&child_path)?;
                    }
                }
            }
        }
        Ok(Subvolume {
            path: path.to_owned(),
            abs_path,
            hierachy: None,
        })
    }

    pub fn read_hierarchy(&mut self) {
        let mut hierachy: Vec<(String, FileType)> =
            Subvolume::_build_hierarchy(&self.path, &self.abs_path)
                .into_iter()
                .map(|(path, ft)| {
                    (
                        path.strip_prefix(&self.abs_path)
                            .unwrap()
                            .trim_start_matches('/')
                            .to_string(),
                        ft,
                    )
                })
                .collect();
        hierachy.sort_by_key(|(path, _)| path.len());
        self.hierachy = Some(hierachy);
    }

    fn _build_hierarchy(path: &Path, abs_path: &str) -> HashMap<String, FileType> {
        let children = Subvolume::_btrfs_list_child(path).unwrap();
        let mut hierachy: HashMap<String, FileType> = HashMap::new();
        for child_abs_path in children {
            hierachy.insert(child_abs_path.to_owned(), FileType::Subvolume);
            for ancestor in Path::new(&child_abs_path).ancestors().skip(1) {
                let ancestor = ancestor.to_str().unwrap();

                assert_ne!(ancestor, "");

                if ancestor == abs_path {
                    break;
                } else {
                    hierachy.insert(ancestor.to_owned(), FileType::Directory);
                }
            }
            let child_path = path.join(
                child_abs_path
                    .strip_prefix(abs_path)
                    .unwrap()
                    .trim_start_matches('/'),
            );
            hierachy.extend(Subvolume::_build_hierarchy(&child_path, &child_abs_path));
        }
        hierachy
    }

    fn _btrfs_subvolume_show_abs_path(path: &Path) -> Result<String, String> {
        let output = process::Command::new("btrfs")
            .arg("subvolume")
            .arg("show")
            .arg(path.to_str().unwrap())
            .output()
            .or_else(|e| Err(e.to_string()))?;
        if !output.status.success() {
            let error = String::from_utf8(output.stderr).unwrap();
            return Err(error);
        }
        let output = String::from_utf8(output.stdout).unwrap();
        let abs_path: Vec<&str> = output.lines().take(1).collect();
        let mut abs_path = abs_path[0].to_string();
        if !abs_path.starts_with('/') {
            abs_path.insert(0, '/');
        }
        Ok(abs_path)
    }

    fn _btrfs_list_child(path: &Path) -> Result<Vec<String>, String> {
        let output = process::Command::new("btrfs")
            .arg("subvolume")
            .arg("list")
            .arg("-ot")
            .arg("--sort=path")
            .arg(path.to_str().unwrap())
            .output()
            .or_else(|e| Err(e.to_string()))?;
        if !output.status.success() {
            let error = String::from_utf8(output.stderr).unwrap();
            return Err(error);
        }
        let output = String::from_utf8(output.stdout).unwrap();
        let mut child: Vec<String> = output
            .lines()
            .skip(2)
            .map(|line| line.split_whitespace().last().unwrap().to_string())
            .collect();
        child.iter_mut().map(|s| s.insert(0, '/')).count();
        Ok(child)
    }

    fn _btrfs_subvolume_create(path: &Path) -> Result<(), String> {
        let output = process::Command::new("btrfs")
            .arg("subvolume")
            .arg("create")
            .arg(path.to_str().unwrap())
            .output()
            .or_else(|e| Err(e.to_string()))?;
        if !output.status.success() {
            let error = String::from_utf8(output.stderr).unwrap();
            return Err(error);
        }
        Ok(())
    }
}
