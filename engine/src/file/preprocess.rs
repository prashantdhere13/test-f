use std::path::{Path, PathBuf};

use async_walkdir::WalkDir;
use log::*;
use tokio::process::Command;
use tokio_stream::StreamExt;

use crate::player::utils::probe::ffprobe;
use crate::utils::{config::PlayoutConfig, errors::ServiceError, logging::Target};

/// File extensions that are eligible for preprocessing into `.ts`.
/// These camera/encoder formats are the minimum set that always get transcoded.
pub const PREPROCESS_EXTENSIONS: &[&str] = &["mxf", "mov", "mpg"];

/// Decide whether a media file with the given extension should be
/// auto-preprocessed into a standardized `.ts`.
///
/// Every extension configured for playback (`config.storage.extensions`) is
/// preprocessed, so MP4/MKV/etc. also get a `.ts` (logo applied only at
/// playout). `.ts` itself is always excluded to avoid reprocessing existing
/// process output.
pub fn should_preprocess(config: &PlayoutConfig, ext: &str) -> bool {
    let ext = ext.to_lowercase();
    if ext == "ts" {
        return false;
    }

    // Always preprocess the camera/encoder formats even if they are missing
    // from the configured extension list.
    if PREPROCESS_EXTENSIONS.iter().any(|e| *e == ext) {
        return true;
    }

    config
        .storage
        .extensions
        .iter()
        .any(|e| e.eq_ignore_ascii_case(&ext))
}

/// Progress information for preprocessing.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PreprocessProgress {
    pub total: usize,
    pub processed: usize,
    pub failed: usize,
    pub current_file: String,
    pub running: bool,
}

impl Default for PreprocessProgress {
    fn default() -> Self {
        Self {
            total: 0,
            processed: 0,
            failed: 0,
            current_file: String::new(),
            running: false,
        }
    }
}

/// Get the preprocessed path for a given source file.
///
/// Maps `<storage>/media/subdir/file.mp4` → `<storage>/media/process/subdir/file.ts`
/// The preprocessed file lives inside the media folder so it appears in the
/// file browser and can be scheduled in playlists.
pub fn preprocessed_path(config: &PlayoutConfig, source: &Path) -> Option<PathBuf> {
    let storage = &config.channel.storage;

    let source_str = source.to_string_lossy();
    let storage_str = storage.to_string_lossy();

    // Get the relative path from storage
    if let Some(rel) = source_str.strip_prefix(storage_str.as_ref()) {
        let rel = rel.trim_start_matches('\\').trim_start_matches('/');
        let rel_path = Path::new(rel);
        let stem = rel_path.file_stem()?;

        // Place under storage/media/process/ (visible in file browser)
        let mut target = storage.join("media").join("process");

        // The source usually lives under <storage>/media/..., so drop the
        // leading "media" component from the relative path. Otherwise the
        // target would become <storage>/media/process/media/... (a doubled
        // "media" segment) and the .ts file would never appear where expected.
        let mut comps: Vec<_> = rel_path.components().collect();
        if let Some(std::path::Component::Normal(first)) = comps.first() {
            if *first == std::ffi::OsStr::new("media") {
                comps.remove(0);
            }
        }
        let rel_without_media: PathBuf = comps.into_iter().collect();
        if let Some(parent) = rel_without_media.parent() {
            if !parent.as_os_str().is_empty() {
                target = target.join(parent);
            }
        }
        target.push(rel_without_media.file_stem()?);
        target.set_extension("ts");
        Some(target)
    } else {
        None
    }
}

/// Probe the video stream duration (in seconds) of the source and an existing
/// preprocessed `.ts`, used to decide whether a previously created file is
/// still valid.
///
/// Uses video **stream** duration (not `format.duration`) because MPEG-TS
/// containers can report a slightly different container duration due to
/// padding, while the video stream duration is frame-accurate.
/// Returns `None` if either probe fails or no video stream is found.
async fn preprocessed_duration(source: &Path, target: &Path) -> Option<(f64, f64)> {
    let src = ffprobe(source).await.ok()?;
    let tgt = ffprobe(target).await.ok()?;

    // Pick the first video stream duration
    let src_dur = src
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video"))
        .and_then(|s| s.duration)?;
    let tgt_dur = tgt
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video"))
        .and_then(|s| s.duration)?;

    Some((src_dur, tgt_dur))
}

/// Check if a preprocessed .ts file exists for the given source.
pub fn has_preprocessed(config: &PlayoutConfig, source: &Path) -> bool {
    if let Some(target) = preprocessed_path(config, source) {
        target.exists()
    } else {
        false
    }
}

/// Get the preprocessed source path if available, otherwise return the original.
pub fn resolved_source(config: &PlayoutConfig, source: &Path) -> (PathBuf, bool) {
    if let Some(target) = preprocessed_path(config, source) {
        if target.exists() {
            return (target, true);
        }
    }
    (source.to_path_buf(), false)
}

/// Build the ffmpeg command for preprocessing a single file.
///
/// Transcodes to standardized MPEG-TS with:
/// - Target resolution and fps from processing config
/// - Audio normalization
/// - yuv420p pixel format for maximum compatibility
///
/// Note: Channel logo is NOT applied here. The logo is added only during the
/// final HLS/UDP playout stream.
fn build_preprocess_cmd(config: &PlayoutConfig, input: &Path, output: &Path) -> Vec<String> {
    let p = &config.processing;
    let mut cmd = vec![
        "ffmpeg".to_string(),
        "-hide_banner".to_string(),
        "-y".to_string(),
        "-i".to_string(),
        input.to_string_lossy().to_string(),
    ];

    let filter_str = format!(
        "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,fps={},format=yuv420p",
        p.width, p.height, p.width, p.height, p.fps,
    );
    cmd.extend(["-vf".to_string(), filter_str]);

    cmd.extend([
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-tune".to_string(),
        "zerolatency".to_string(),
        // Cap the number of encoding threads so a background batch transcode
        // never saturates every core - keeps CPU headroom for live playout.
        "-threads".to_string(),
        "4".to_string(),
        "-b:v".to_string(),
        "5000k".to_string(),
        "-maxrate".to_string(),
        "5000k".to_string(),
        "-bufsize".to_string(),
        "10000k".to_string(),
        "-g".to_string(),
        "25".to_string(),
    ]);

    // Audio
    cmd.extend([
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "192k".to_string(),
        "-ar".to_string(),
        "48000".to_string(),
        "-ac".to_string(),
        p.audio_channels.to_string(),
    ]);

    // Output format
    cmd.extend([
        "-f".to_string(),
        "mpegts".to_string(),
        "-mpegts_flags".to_string(),
        "+initial_discontinuity".to_string(),
        output.to_string_lossy().to_string(),
    ]);

    cmd
}

/// Preprocess a single media file into a standardized `.ts`.
///
/// Returns `Ok(true)` if the file was (or already was) preprocessed,
/// `Ok(false)` if it was skipped, or `Err(...)` on failure.
pub async fn preprocess_file(config: &PlayoutConfig, source: &Path) -> Result<bool, ServiceError> {
    let id = config.general.channel_id;

    let Some(target) = preprocessed_path(config, source) else {
        return Ok(false);
    };

    // Skip if already preprocessed and still valid
    if target.exists() {
        if let (Ok(src_meta), Ok(tgt_meta)) = (
            tokio::fs::metadata(source).await,
            tokio::fs::metadata(&target).await,
        ) {
            let size_ok = tgt_meta.len() > 10_000;

            // Force a one-time reprocess of preprocessed .ts files created
            // BEFORE the channel-logo was removed from the transcode step.
            // Those old files still have the logo baked in; a full refresh
            // strips it so the logo is only applied on the final HLS/UDP
            // playout. Targets written on/after this epoch use the logo-free
            // pipeline and are treated normally.
            const REPROCESS_EPOCH: u64 = 1_788_220_800; // 2026-09-01T00:00:00Z
            let epoch_ok = tgt_meta.modified()
                .map(|m| m.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() >= REPROCESS_EPOCH)
                    .unwrap_or(false))
                .unwrap_or(false);

            if let (Ok(src_time), Ok(tgt_time)) = (src_meta.modified(), tgt_meta.modified()) {
                let duration_ok = match preprocessed_duration(source, &target).await {
                    Some((src_dur, tgt_dur)) => {
                        let diff = (tgt_dur - src_dur).abs();
                        debug!(target: Target::file_mail(), channel = id;
                            "Duration check: src={:.3}s tgt={:.3}s diff={:.3}s ok={}",
                            src_dur, tgt_dur, diff, diff < 2.0
                        );
                        src_dur > 0.0 && diff < 2.0
                    }
                    None => {
                        debug!(target: Target::file_mail(), channel = id;
                            "Duration check: probe failed for source or target");
                        false
                    }
                };

                if tgt_time >= src_time && size_ok && duration_ok && epoch_ok {
                    debug!(target: Target::file_mail(), channel = id;
                        "Skipping preprocessed (valid): {}", target.display());
                    return Ok(true);
                } else {
                    info!(target: Target::file_mail(), channel = id;
                        "Reprocessing: size_ok={} tgt_newer={} duration_ok={} epoch_ok={} file={}",
                        size_ok, tgt_time >= src_time, duration_ok, epoch_ok, target.display());
                }
            }
        }
    }

    // Create target directory
    if let Some(parent) = target.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    let cmd = build_preprocess_cmd(config, source, &target);

    info!(target: Target::file_mail(), channel = id;
        "Preprocessing: {} -> {}",
        source.display(),
        target.display()
    );

    let result = Command::new(&cmd[0]).args(&cmd[1..]).output().await;

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(true)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!(target: Target::file_mail(), channel = id;
                    "Preprocess failed for {}: {}",
                    source.display(),
                    &stderr.chars().take(200).collect::<String>()
                );
                Err(ServiceError::BadRequest(format!(
                    "Preprocess failed: {}",
                    source.display()
                )))
            }
        }
        Err(e) => {
            warn!(target: Target::file_mail(), channel = id;
                "Preprocess error for {}: {e}",
                source.display()
            );
            Err(ServiceError::BadRequest(format!("Preprocess error: {e}")))
        }
    }
}

/// Preprocess all media files in the storage folder.
///
/// Scans for supported media files, transcodes each to standardized .ts
/// and saves to the process folder.
pub async fn preprocess_all(config: PlayoutConfig) -> Result<PreprocessProgress, ServiceError> {
    let storage = &config.channel.storage;
    let process = &config.channel.process;

    // Ensure process directory exists
    tokio::fs::create_dir_all(process)
        .await
        .map_err(|e| ServiceError::BadRequest(format!("Can't create process folder: {e}")))?;

    let mut progress = PreprocessProgress {
        running: true,
        ..Default::default()
    };

    // Collect media files eligible for preprocessing (all configured extensions)
    let mut files = vec![];
    let skip_dir = process.clone();

    info!(target: Target::file_mail(), channel = config.general.channel_id;
        "Preprocess scan: storage={:?} process={:?} extensions={:?}",
        storage, process, config.storage.extensions
    );

    let mut entries = WalkDir::new(storage);
    while let Some(Ok(entry)) = entries.next().await {
        if entry.path().is_file() {
            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if should_preprocess(&config, &ext_str) {
                    // Skip files already in the process folder
                    if entry.path().starts_with(&skip_dir) {
                        continue;
                    }
                    files.push(entry.path());
                    debug!(target: Target::file_mail(), channel = config.general.channel_id;
                        "Found preprocess candidate: {:?}", entry.path());
                }
            }
        }
    }

    progress.total = files.len();
    info!(target: Target::file_mail(), channel = config.general.channel_id;
        "Preprocessing {} files", progress.total
    );

    for source in &files {
        progress.current_file = source
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        match preprocess_file(&config, &source).await {
            Ok(true) => progress.processed += 1,
            Ok(false) => progress.processed += 1,
            Err(_) => progress.failed += 1,
        }
    }

    progress.running = false;

    info!(target: Target::file_mail(), channel = config.general.channel_id;
        "Preprocessing complete: {}/{} processed, {} failed",
        progress.processed, progress.total, progress.failed
    );

    Ok(progress)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::{Channel, General, Processing};
    use std::path::PathBuf;

    fn make_test_config() -> PlayoutConfig {
        let mut config = PlayoutConfig::default();
        config.channel.storage = PathBuf::from("/tmp/storage");
        config.channel.process = PathBuf::from("/tmp/storage/process");
        config.storage.extensions = vec!["mp4".to_string(), "mkv".to_string()];
        config.processing.add_logo = false;
        config
    }

    #[test]
    fn test_preprocessed_path_mapping() {
        let config = make_test_config();
        let source = PathBuf::from("/tmp/storage/media/subdir/video.mp4");
        let result = preprocessed_path(&config, &source);
        assert_eq!(
            result,
            Some(PathBuf::from("/tmp/storage/media/process/subdir/video.ts"))
        );
    }

    #[test]
    fn test_preprocessed_path_root() {
        let config = make_test_config();
        let source = PathBuf::from("/tmp/storage/media/video.mp4");
        let result = preprocessed_path(&config, &source);
        assert_eq!(
            result,
            Some(PathBuf::from("/tmp/storage/media/process/video.ts"))
        );
    }

    #[test]
    fn test_preprocessed_path_outside_storage() {
        let config = make_test_config();
        let source = PathBuf::from("/other/path/video.mp4");
        let result = preprocessed_path(&config, &source);
        assert_eq!(result, None);
    }
}
