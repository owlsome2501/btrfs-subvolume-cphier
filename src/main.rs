use std::env;
use std::path::Path;
use btrfs_subvolume_cphier::Subvolume;

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

    let mut src_volume = Subvolume::from(src)?;
    src_volume.read_hierarchy();
    println!("{}", src_volume.abs_path);
    println!("{:?}", src_volume.hierachy);
    let mut dst_volume = Subvolume::create(dst, &src_volume.hierachy.unwrap())?;
    dst_volume.read_hierarchy();
    println!("{}", dst_volume.abs_path);
    println!("{:?}", dst_volume.hierachy);

    Ok(())
}
