//! Triggers a mix of file-system operations intended to produce a rich stream
//! of both IRP-based and Fast-I/O FileIo/FltIo completion events.
//!
//! Small fine-grained cached reads/writes (via the normal I/O stack) tend to
//! be dispatched through FastIo, so they should register as FltIoCompletion
//! events regardless of which `PERF_FLT_*` mask is active. The point is simply
//! to generate a reproducible burst of minifilter I/O traffic.

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use memmap2::MmapMut;

fn work_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("flt_io_fastio_compare");
    fs::create_dir_all(&dir).ok();
    dir
}

/// Run the workload once. Produces a few thousand operations.
pub fn trigger_workload() {
    let dir = work_dir();

    // Clean up any stale files first.
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).ok();

    // Create + write a batch of small files.
    for round in 0..3 {
        for i in 0..40 {
            let path = dir.join(format!("f{}_{}.bin", round, i));
            if let Ok(mut f) = fs::File::create(&path) {
                let payload: Vec<u8> = (0..512u32).map(|b| (b as u8).wrapping_add(i)).collect();
                let _ = f.write_all(&payload);
                let _ = f.flush();
            }
        }
    }

    // Read them back with small chunked cached I/O (favors FastIo).
    for round in 0..3 {
        for i in 0..40 {
            let path = dir.join(format!("f{}_{}.bin", round, i));
            if let Ok(mut f) = fs::File::open(&path) {
                let mut buf = [0u8; 128];
                loop {
                    match f.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                }
            }
        }
    }

    // Rewrite via OpenOptions with small writes.
    for i in 0..40 {
        let path = dir.join(format!("f0_{}.bin", i));
        if let Ok(mut f) = OpenOptions::new().write(true).open(&path) {
            let _ = f.write_all(b"rewrite-burst");
            let _ = f.flush();
        }
    }

    // Seek + random small reads (cached, FastIo path).
    for i in 0..40 {
        let path = dir.join(format!("f1_{}.bin", i));
        if let Ok(mut f) = fs::File::open(&path) {
            let _ = f.seek(SeekFrom::Start((i % 512) as u64));
            let mut buf = [0u8; 64];
            let _ = f.read(&mut buf);
        }
    }

    // Some mapped I/O to spice the mix.
    trigger_mmap(&dir);

    // Tidy up.
    let _ = fs::remove_dir_all(&dir);
}

fn trigger_mmap(dir: &std::path::Path) {
    let path = dir.join("mmap.bin");
    {
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        f.write_all(&[0u8; 4096]).unwrap();
        f.flush().unwrap();
    }
    if let Ok(file) = OpenOptions::new().read(true).write(true).open(&path) {
        if let Ok(mut mmap) = unsafe { MmapMut::map_mut(&file) } {
            mmap[0..512].copy_from_slice(&[0xAB; 512]);
            let _ = mmap.flush_range(0, 512);
            let _ = mmap.flush();
        }
    }
    let _ = fs::remove_file(&path);
}