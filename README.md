# btrfs-subvolume-cphier

Copy hierarchy from src to dst, create subvolumes and necessary directories.

## Usage

```bash
btrfs-subvolume-cphier SRC DST
```

`SRC` should be valid btrfs subvolume.
Only create subvolumes and necessary directories when they doesn't exist under `DST`.

## Example

cooperate with `rsync` to backup files.

```bash
btrfs-subvolume-cphier SRC DST
rsync --delete -aAX SRC/ DST
```

## Known Issue

It's not easy to handle relationship between subvolume and its snapshots gracefully.
