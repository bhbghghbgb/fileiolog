use std::fs;
use std::io::Write;
use std::path::Path;

use memmap2::MmapMut;

/// Trigger various file system operations to generate FileIo events
pub fn trigger_all_file_operations() {
    let test_dir = Path::new("C:\\temp_fileio_test");

    // Create test directory
    let _ = fs::create_dir_all(test_dir);

    // Create files
    trigger_create_files(test_dir);

    // Read files
    trigger_read_files(test_dir);

    // Write files
    trigger_write_files(test_dir);

    // List directory (DirEnum)
    trigger_dir_enum(test_dir);

    // Rename files
    trigger_rename_files(test_dir);

    // Get file info (QueryInfo)
    trigger_query_info(test_dir);

    // Set file info (SetInfo)
    trigger_set_info(test_dir);

    // Flush files
    trigger_flush_files(test_dir);

    // Delete files
    trigger_delete_files(test_dir);

    // Memory-mapped file operations
    trigger_mmap_operations(test_dir);

    // Clean up
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
    // List directory contents multiple times
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

    // Rename back
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
    // Setting file attributes counts as SetInfo
    for i in 0..3 {
        let path = dir.join(format!("test_create_{}.txt", i));
        if path.exists() {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                // Just reading metadata with attributes triggers some info queries
                if let Ok(meta) = fs::metadata(&path) {
                    let _ = meta.file_attributes();
                }
            }
        }
    }
}

fn trigger_flush_files(dir: &Path) {
    // Opening and closing with flush
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
    // Create some extra files to delete
    for i in 0..3 {
        let path = dir.join(format!("test_delete_{}.txt", i));
        let _ = fs::write(&path, "delete me");
    }

    // Delete them
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

    // Create and size the file
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

    // Memory-map the file for read+write
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };

    // Write data through the mapping
    let pattern: Vec<u8> = (0..256).map(|b| b as u8).collect();
    for slot in mmap.chunks_mut(256) {
        slot.copy_from_slice(&pattern);
    }
    // Flush each chunk to trigger write-back / dirty pages
    for offset in (0..mmap.len()).step_by(256) {
        let end = (offset + 256).min(mmap.len());
        mmap.flush_range(offset, end - offset).unwrap();
    }

    // Read data back through the mapping
    for chunk in mmap.chunks(256) {
        let _sum: u64 = chunk.iter().map(|&b| b as u64).sum();
    }

    // Flush the entire mapping
    mmap.flush().unwrap();

    // Flush a range (offset 0..1024)
    mmap.flush_range(0, 1024).unwrap();

    // Remap with a different size (extend to 8KB)
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

    // Write to the newly extended region
    mmap[4096..4096 + 256].copy_from_slice(&[0xAB; 256]);
    mmap.flush().unwrap();

    // Read from both original and extended regions
    let _ = mmap[0..256].to_vec();
    let _ = mmap[4096..4096 + 256].to_vec();

    drop(mmap);
    drop(file);

    // Clean up
    let _ = fs::remove_file(&path);
}
