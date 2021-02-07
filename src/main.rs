use std::env;
use std::process;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args()
        .into_iter()
        .skip(1)
        .take(2)
        .collect::<Vec<String>>();
    println!("len={}, cap={}", args.len(), args.capacity());
    let (src, dst) = match &args[..] {
        [src, dst] => (src, dst),
        _ => return Err("Invalid parameters".into()),
    };
    btrfs_get_hierarchy(src)?;
    Ok(())
}

fn btrfs_get_hierarchy(path: &str) -> Result<(), String> {
    let output = process::Command::new("btrfs")
        .arg("subvolume")
        .arg("list")
        .arg("-ot")
        .arg("--sort=path")
        .arg(path)
        .output()
        .unwrap();
    if !output.status.success() {
        let error = String::from_utf8(output.stderr).unwrap();
        return Err(error);
    }
    let hierachy: Vec<String> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .skip(2)
        .map(|line| line.split_whitespace().last().unwrap().to_string())
        .collect();
    println!("{:?}", hierachy);
    Ok(())
}
