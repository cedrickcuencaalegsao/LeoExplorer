use shared::CacheData;
use base_path::{cache_file_path, cache_temp_file_path};
use std::fs::{self,File};
use std::io::{BufWriter, Write};

pub fn write_cache_to_disk(data: &CacheData) -> Result<(), String> {
    let cache_file = cache_file_path();
    let temp_file = cache_temp_file_path();

    match File::create(&temp_file) {
        Ok(file) => {
            let writer = BufWriter::with_capacity(16 * 1024 * 1024, file);
            let mut encoder = zstd::Encoder::new(writer, 0)
                .map_err(|e| format!("Failed to init encoder: {}", e))?;

            // Multi-threaded zstd uses extra CPU threads for compression (no GPU in portable zstd).
            let zstd_workers = (num_cpus::get() as u32).clamp(1, 8);
            if let Err(e) = encoder.multithread(zstd_workers) {
                eprintln!("zstd multithread disabled ({}); continuing single-threaded", e);
            }

            bincode::serialize_into(&mut encoder, data)
                .map_err(|e| format!("Failed to serialize cache: {}", e))?;

            let writer = encoder
                .finish()
                .map_err(|e| format!("Failed to finalize compression: {}", e))?;

            let mut writer = writer;
            writer
                .flush()
                .map_err(|e| format!("Failed to flush cache file: {}", e))?;

            drop(writer);

            match fs::metadata(&temp_file) {
                Ok(meta) => {
                    if meta.len() == 0 {
                        let _ = fs::remove_file(&temp_file);
                        return Err("Cache file is empty after write".to_string());
                    }
                }
                Err(e) => {
                    let _ = fs::remove_file(&temp_file);
                    return Err(format!("Failed to verify temp cache file: {}", e));
                }
            }

            fs::rename(&temp_file, &cache_file)
                .map_err(|e| format!("Failed to finalize cache file: {}", e))?;

            Ok(())
        }
        Err(e) => Err(format!("Failed to create temp cache file: {}", e)),
    }
}