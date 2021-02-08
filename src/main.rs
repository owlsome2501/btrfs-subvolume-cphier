use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args()
        .into_iter()
        .skip(1)
        .take(2)
        .collect::<Vec<String>>();
    let (src, dst) = match &args[..] {
        [src, dst] => (Path::new(src), Path::new(dst)),
        _ => return Err("Invalid parameters".into()),
    };

    let mut src_volume = Subvolume::from(src).unwrap();
    src_volume.read_hierarchy();
    println!("{}", src_volume.abs_path);
    println!("{:?}", src_volume.hierachy);
    let mut dst_volume = Subvolume::create(dst, &src_volume.hierachy.unwrap()).unwrap();
    dst_volume.read_hierarchy();
    println!("{}", dst_volume.abs_path);
    println!("{:?}", dst_volume.hierachy);

    Ok(())
}

#[derive(Debug)]
enum FileType {
    Directory,
    Subvolume,
}

struct Subvolume {
    path: PathBuf,
    abs_path: String,
    hierachy: Option<Vec<(String, FileType)>>,
}

impl Subvolume {
    fn from(path: &Path) -> Result<Self, String> {
        let abs_path = Subvolume::_btrfs_subvolume_show_abs_path(path)?;
        Ok(Subvolume {
            path: path.to_owned(),
            abs_path,
            hierachy: None,
        })
    }

    fn create(path: &Path, hierachy: &Vec<(String, FileType)>) -> Result<Self, String> {
        Subvolume::_btrfs_subvolume_create(path)?;
        let abs_path = Subvolume::_btrfs_subvolume_show_abs_path(path)?;
        for (child_rela_path, ft) in hierachy {
            let child_path = path.join(child_rela_path);
            // println!("create {:?} of {:?}", child_path, ft);
            match ft {
                FileType::Directory => {
                    std::fs::create_dir(child_path).or_else(|e| Err(e.to_string()))?
                }
                FileType::Subvolume => Subvolume::_btrfs_subvolume_create(&child_path)?,
            }
        }
        Ok(Subvolume {
            path: path.to_owned(),
            abs_path,
            hierachy: None,
        })
    }

    fn read_hierarchy(&mut self) {
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
            .unwrap();
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
            .unwrap();
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
            .unwrap();
        if !output.status.success() {
            let error = String::from_utf8(output.stderr).unwrap();
            return Err(error);
        }
        Ok(())
    }
}
