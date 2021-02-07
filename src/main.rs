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
    println!("len={}, cap={}", args.len(), args.capacity());
    let (src, dst) = match &args[..] {
        [src, dst] => (Path::new(src), Path::new(dst)),
        _ => return Err("Invalid parameters".into()),
    };

    let src_volume = Subvolume::new(src).unwrap();
    println!("{:?}", Subvolume::_btrfs_list_child(src).unwrap());
    println!("{}", src_volume.abs_path);
    println!("{:?}", src_volume.hierachy);

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
    hierachy: HashMap<String, FileType>,
}

impl Subvolume {
    fn new(path: &Path) -> Result<Self, String> {
        let abs_path = Subvolume::_btrfs_subvolume_show_abs_path(path)?;
        let hierachy = Subvolume::_build_hierarchy(path, &abs_path);
        Ok(Subvolume {
            path: path.to_owned(),
            abs_path,
            hierachy,
        })
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
                    .strip_prefix("/")
                    .unwrap(),
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
        let abs_path = abs_path[0].to_string();
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
        // let mut hierachy: Vec<Vec<&str>> = output
        let child: Vec<String> = output
            .lines()
            .skip(2)
            // .map(|line| line.split_whitespace().last().unwrap().split('/').collect())
            .map(|line| line.split_whitespace().last().unwrap().to_string())
            .collect();
        // hierachy.sort_by_key(|path| path.len());
        // let mut subvolume = HashSet::new();
        Ok(child)
    }
}
