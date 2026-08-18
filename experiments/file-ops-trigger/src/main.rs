use std::fs;
use std::io::Write;
use std::path::Path;

use memmap2::MmapMut;

fn main() {
    let test_dir = std::env::args()
        .nth(1)
        .map_or_else(|| Path::new("C:\\temp_fileio_test").to_path_buf(), std::path::PathBuf::from);

    let test_dir = test_dir.as_path();

    let _ = fs::create_dir_all(test_dir);

    trigger_create_files(test_dir);
    trigger_read_files(test_dir);
    trigger_write_files(test_dir);
    trigger_dir_enum(test_dir);
    trigger_rename_files(test_dir);
    trigger_query_info(test_dir);
    trigger_set_info(test_dir);
    trigger_flush_files(test_dir);
    trigger_delete_files(test_dir);
    trigger_mmap_operations(test_dir);

    let _ = fs::remove_dir_all(test_dir);
}

fn trigger_create_files(dir: &Path) {
    for i in 0..5 {
        let path = dir.join(format!("test_create_{}.txt", i));
        let _ = fs::write(&path, format!("content {}", i));
    }
}

fn trigger_read_files(dir: &Path) {
    for i in 0..5 {
        let path = dir.join(format!("test_create_{}.txt", i));
        if path.exists() {
            let _ = fs::read(&path);
        }
    }
}

fn trigger_write_files(dir: &Path) {
    for i in 0..5 {
        let path = dir.join(format!("test_create_{}.txt", i));
        if path.exists() {
            let _ = fs::write(&path, format!("updated content {}", i));
        }
    }
}

fn trigger_dir_enum(dir: &Path) {
    for _ in 0..3 {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let _ = entry.metadata();
            }
        }
    }
}

fn trigger_rename_files(dir: &Path) {
    for i in 0..3 {
        let old_path = dir.join(format!("test_create_{}.txt", i));
        let new_path = dir.join(format!("test_renamed_{}.txt", i));
        if old_path.exists() {
            let _ = fs::rename(&old_path, &new_path);
        }
    }

    for i in 0..3 {
        let old_path = dir.join(format!("test_renamed_{}.txt", i));
        let new_path = dir.join(format!("test_create_{}.txt", i));
        if old_path.exists() {
            let _ = fs::rename(&old_path, &new_path);
        }
    }
}

fn trigger_query_info(dir: &Path) {
    for i in 0..5 {
        let path = dir.join(format!("test_create_{}.txt", i));
        if path.exists() {
            let _ = fs::metadata(&path);
            let _ = fs::symlink_metadata(&path);
        }
    }
}

fn trigger_set_info(dir: &Path) {
    for i in 0..3 {
        let path = dir.join(format!("test_create_{}.txt", i));
        if path.exists() {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if let Ok(meta) = fs::metadata(&path) {
                    let _ = meta.file_attributes();
                }
            }
        }
    }
}

fn trigger_flush_files(dir: &Path) {
    for i in 0..3 {
        let path = dir.join(format!("test_create_{}.txt", i));
        if path.exists() {
            if let Ok(mut file) = fs::OpenOptions::new().write(true).open(&path) {
                let _ = file.write_all(b"flush test");
                let _ = file.flush();
            }
        }
    }
}

fn trigger_delete_files(dir: &Path) {
    for i in 0..3 {
        let path = dir.join(format!("test_delete_{}.txt", i));
        let _ = fs::write(&path, "delete me");
    }

    for i in 0..3 {
        let path = dir.join(format!("test_delete_{}.txt", i));
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
    }
}

fn trigger_mmap_operations(dir: &Path) {
    use std::fs::OpenOptions;

    let path = dir.join("test_mmap.bin");

    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        file.write_all(&[0u8; 4096]).unwrap();
        file.flush().unwrap();
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };

    let pattern: Vec<u8> = (0..256).map(|b| b as u8).collect();
    for slot in mmap.chunks_mut(256) {
        slot.copy_from_slice(&pattern);
    }
    for offset in (0..mmap.len()).step_by(256) {
        let end = (offset + 256).min(mmap.len());
        mmap.flush_range(offset, end - offset).unwrap();
    }

    for chunk in mmap.chunks(256) {
        let _sum: u64 = chunk.iter().map(|&b| b as u64).sum();
    }

    mmap.flush().unwrap();
    mmap.flush_range(0, 1024).unwrap();

    drop(mmap);
    drop(file);
    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.set_len(8192).unwrap();
        file.flush().unwrap();
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };

    mmap[4096..4096 + 256].copy_from_slice(&[0xAB; 256]);
    mmap.flush().unwrap();

    let _ = mmap[0..256].to_vec();
    let _ = mmap[4096..4096 + 256].to_vec();

    drop(mmap);
    drop(file);

    let _ = fs::remove_file(&path);
}
