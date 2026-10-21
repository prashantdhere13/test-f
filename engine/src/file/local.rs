use std::{
    io,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
};

#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;

use actix_multipart::Multipart;
use async_walkdir::WalkDir;
// use futures_util::TryStreamExt as _;
use lexical_sort::{PathSort, natural_lexical_cmp};
use log::*;
use rand::{Rng, SeedableRng, distr::Alphanumeric, rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::AsyncWriteExt,
    sync::{Mutex, RwLock},
    task::{JoinHandle, spawn_blocking},
};
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::file::{
    MediaAsset, MoveObject, PathObject, VideoFile, norm_abs_path, preprocess::has_preprocessed,
    watcher::watch,
};
use crate::player::utils::{Media, file_extension, include_file_extension, probe::MediaProbe};
use crate::utils::{config::PlayoutConfig, errors::ServiceError, logging::Target};

#[derive(Clone, Debug)]
pub struct LocalStorage {
    pub root: Arc<RwLock<PathBuf>>,
    pub extensions: Arc<RwLock<Vec<String>>>,
    pub watch_handler: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Result of an animated channel logo processing run.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AnimatedLogoResult {
    pub path: String,
    pub play_path: String,
    pub process_path: String,
    pub status: String,
}

/// Result of a graphic (bug) processing run.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GraphicsProcessResult {
    pub path: String,
    pub play_path: String,
    pub process_path: String,
    pub status: String,
}

struct EncodeResult {
    mov_path: PathBuf,
    webm_path: PathBuf,
}

impl LocalStorage {
    pub async fn new(root: PathBuf, extensions: Vec<String>) -> Self {
        if !root.is_dir() {
            fs::create_dir_all(&root)
                .await
                .unwrap_or_else(|_| panic!("Can't create storage folder: {root:?}"));
        }

        // create default asset folders for each channel
        for folder in ["media", "logo", "text", "subtitles", "filler", "graphics"] {
            let path = root.join(folder);

            if !path.is_dir()
                && let Err(e) = fs::create_dir_all(&path).await
            {
                error!("Can't create folder {path:?}: {e}");
            }
        }

        // Create media/process/ for preprocessed .ts files
        let process_dir = root.join("media").join("process");
        if !process_dir.is_dir()
            && let Err(e) = fs::create_dir_all(&process_dir).await
        {
            error!("Can't create preprocess folder {process_dir:?}: {e}");
        }

        Self {
            root: Arc::new(RwLock::new(root)),
            extensions: Arc::new(RwLock::new(extensions)),
            watch_handler: Arc::new(Mutex::new(None)),
        }
    }
}

impl Drop for LocalStorage {
    fn drop(&mut self) {
        let watch_handler = self.watch_handler.clone();

        tokio::spawn(async move {
            let mut watch_handler = watch_handler.lock().await;

            if let Some(handler) = watch_handler.as_mut() {
                handler.abort();
            }
        });
    }
}

impl LocalStorage {
    pub async fn browser(&self, path_obj: &PathObject) -> Result<PathObject, ServiceError> {
        let (path, parent, path_component) =
            norm_abs_path(&self.root.read().await, &path_obj.source)?;
        let mut parent_folders = vec![];

        let parent_path = if path_component.is_empty() {
            self.root.read().await.clone()
        } else {
            path.parent().unwrap().to_path_buf()
        };

        let mut obj = PathObject::new(path_component, Some(parent));
        obj.folders_only = path_obj.folders_only;

        if path != parent_path && !path_obj.folders_only {
            let mut parents = fs::read_dir(&parent_path).await?;

            while let Some(child) = parents.next_entry().await? {
                if child.metadata().await?.is_dir() {
                    parent_folders.push(
                        child
                            .path()
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    );
                }
            }

            parent_folders.path_sort(natural_lexical_cmp);

            obj.parent_folders = Some(parent_folders);
        }

        let mut paths_obj = fs::read_dir(path).await?;

        let mut files = vec![];
        let mut folders = vec![];

        while let Some(child) = paths_obj.next_entry().await? {
            let f_meta = child.metadata().await?;

            // ignore hidden files/folders on unix
            if child.path().to_string_lossy().to_string().contains("/.") {
                continue;
            }

            if f_meta.is_dir() {
                folders.push(
                    child
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                );
            } else if f_meta.is_file() && !path_obj.folders_only {
                if let Some(ext) = file_extension(&child.path()) {
                    let ext_lower = ext.to_string().to_lowercase();
                    let in_extensions = self.extensions.read().await.contains(&ext_lower);
                    // Also include .ts files inside the media/process folder
                    // (preprocessed output) even if "ts" is not in configured extensions
                    let in_process_folder = {
                        let root = self.root.read().await;
                        let process_dir = root.join("media").join("process");
                        child.path().starts_with(&process_dir)
                    };

                    if in_extensions || (ext_lower == "ts" && in_process_folder) {
                        files.push(child.path());
                    }
                }
            }
        }

        folders.path_sort(natural_lexical_cmp);
        files.path_sort(natural_lexical_cmp);
        let mut media_files = vec![];

        for file in files {
            match MediaProbe::new(file.to_string_lossy().as_ref()).await {
                Ok(probe) => {
                    // Prefer the first video stream duration for accuracy.
                    // format.duration can be inaccurate for MPEG-TS (container
                    // padding) or MXF (timecode/data streams).
                    let duration = probe
                        .video
                        .first()
                        .and_then(|v| v.duration)
                        .or(probe.format.duration)
                        .unwrap_or_default();

                    let video = VideoFile {
                        name: file.file_name().unwrap().to_string_lossy().to_string(),
                        duration,
                    };
                    media_files.push(video);
                }
                Err(e) => error!("{e:?}"),
            };
        }

        obj.folders = Some(folders);
        obj.files = Some(media_files);

        Ok(obj)
    }

    /// Scan the configured library folders and return a list of media assets
    /// with probed metadata (duration, frame rate, audio) for the media
    /// library view.
    ///
    /// Each category is mapped to the storage folder configured in the
    /// playout config (`storage.media`, `storage.graphics`,
    /// `storage.filler`, `storage.subtitles`).
    pub async fn assets(&self, config: &PlayoutConfig) -> Result<Vec<MediaAsset>, ServiceError> {
        let root = self.root.read().await.clone();
        let mut assets = vec![];
        let process_dir = root.join("media").join("process");

        // Configured storage folders are scanned and mapped to library
        // categories. The media folder is split further by content type.
        let folder_categories: Vec<(&str, PathBuf, bool)> = vec![
            ("videos", root.join(&config.storage.media), true),
            ("graphics", root.join(&config.storage.graphics), false),
            ("rescue", config.storage.filler_path.clone(), false),
            ("subtitles", root.join(&config.storage.subtitles), false),
        ];

        for (default_category, dir, media_folder) in folder_categories {
            if !dir.is_dir() {
                continue;
            }

            let mut entries = WalkDir::new(&dir);
            let mut files = vec![];

            while let Some(Ok(entry)) = entries.next().await {
                if entry.path().is_file() {
                    if let Some(ext) = file_extension(&entry.path()) {
                        let ext_lower = ext.to_string().to_lowercase();
                        let in_extensions = self.extensions.read().await.contains(&ext_lower);
                        let in_process = entry.path().starts_with(&process_dir);

                        // Skip the preprocessed output folder itself for the
                        // "videos" listing so sources aren't duplicated.
                        if (in_extensions || ext_lower == "ts") && !in_process {
                            files.push(entry.path());
                        }
                    }
                }
            }

            files.path_sort(natural_lexical_cmp);

            for file in files {
                let Some(probe) = MediaProbe::new(&file).await.ok() else {
                    continue;
                };

                let video = probe.video.first();
                let duration = video
                    .and_then(|v| v.duration)
                    .or(probe.format.duration)
                    .unwrap_or_default();
                let frame_rate_text = video.map(|v| v.frame_rate.clone()).unwrap_or_default();
                let fps = parse_frame_rate(&frame_rate_text);
                let asset_id = file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let title = file
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                let relative = file
                    .strip_prefix(&root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");

                // Audio-only clips inside the media folder belong to the
                // "audio" category; everything else keeps the folder default.
                let category = if media_folder && probe.video.is_empty() && !probe.audio.is_empty()
                {
                    "audio"
                } else {
                    default_category
                };

                let status = if media_folder {
                    if has_preprocessed(config, &file) {
                        "processed".to_string()
                    } else {
                        "processing".to_string()
                    }
                } else {
                    "processed".to_string()
                };

                assets.push(MediaAsset {
                    asset_id,
                    title,
                    category: category.to_string(),
                    media_type: category_to_type(category),
                    segments: if duration > 0.0 {
                        (duration / 30.0).ceil() as i64
                    } else {
                        0
                    },
                    duration: smpte_time(duration, fps),
                    duration_seconds: duration,
                    audio: probe.audio.len() as i64,
                    cc: false,
                    status,
                    frame_rate: format_frame_rate(&frame_rate_text),
                    path: format!("/{relative}"),
                    play_path: None,
                    process_path: None,
                    graphic_source: None,
                });
            }
        }

        // "logo" category: static images in the channel logo folder plus the
        // processed animated logo video(s) in media/process/logo.
        for dir in [root.join("logo"), process_dir.join("logo")] {
            if !dir.is_dir() {
                continue;
            }

            let mut entries = WalkDir::new(&dir);
            let mut logo_files = vec![];

            while let Some(Ok(entry)) = entries.next().await {
                if entry.path().is_file() {
                    if let Some(ext) = file_extension(&entry.path()) {
                        let ext_lower = ext.to_string().to_lowercase();
                        // Archives and work folders are not assets; the
                        // preview webm is only a preview, not a library entry.
                        if ext_lower == "zip"
                            || ext_lower == "webm"
                            || entry
                                .path()
                                .file_name()
                                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
                        {
                            continue;
                        }

                        logo_files.push(entry.path());
                    }
                }
            }

            logo_files.path_sort(natural_lexical_cmp);

            for file in logo_files {
                let Some(probe) = MediaProbe::new(&file).await.ok() else {
                    continue;
                };

                let video = probe.video.first();
                let duration = video
                    .and_then(|v| v.duration)
                    .or(probe.format.duration)
                    .unwrap_or_default();
                let frame_rate_text = video.map(|v| v.frame_rate.clone()).unwrap_or_default();
                let fps = parse_frame_rate(&frame_rate_text);
                let asset_id = file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let title = file
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                let relative = file
                    .strip_prefix(&root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");

                let is_process = file.starts_with(&process_dir);

                // For generated animated logos the .mov is not browser
                // playable, so point the preview at the VP9 webm copy.
                let play_path = if is_process {
                    let stem = file.file_stem().unwrap().to_string_lossy();
                    let preview = dir.join(format!("{stem}_preview.webm"));

                    preview
                        .is_file()
                        .then(|| {
                            format!(
                                "/{}",
                                preview
                                    .strip_prefix(&root)
                                    .unwrap_or(&preview)
                                    .to_string_lossy()
                                    .replace('\\', "/")
                            )
                        })
                } else {
                    None
                };

                let process_path = if is_process {
                    Some(format!("/{relative}"))
                } else {
                    None
                };

                assets.push(MediaAsset {
                    asset_id,
                    title,
                    category: "logo".to_string(),
                    media_type: if video.is_some() {
                        "video".to_string()
                    } else {
                        "image".to_string()
                    },
                    segments: if duration > 0.0 {
                        (duration / 30.0).ceil() as i64
                    } else {
                        1
                    },
                    duration: smpte_time(duration, fps),
                    duration_seconds: duration,
                    audio: probe.audio.len() as i64,
                    cc: false,
                    status: "processed".to_string(),
                    frame_rate: format_frame_rate(&frame_rate_text),
                    path: format!("/{relative}"),
                    play_path,
                    process_path,
                    graphic_source: None,
                });
            }
        }

        // "graphics" supplemental scan: ZIP sources in the channel graphics
        // folder that still need processing, plus the processed alpha MOV
        // videos in media/process/graphics. Regular video/image files in the
        // graphics folder are already listed by the folder_categories loop.
        let graphics_dirs = [
            root.join(&config.storage.graphics),
            process_dir.join("graphics"),
        ];

        for (dir_index, dir) in graphics_dirs.iter().enumerate() {
            if !dir.is_dir() {
                continue;
            }

            let mut entries = WalkDir::new(dir);
            let mut graphic_files = vec![];

            while let Some(Ok(entry)) = entries.next().await {
                if entry.path().is_file() {
                    if let Some(ext) = file_extension(&entry.path()) {
                        let ext_lower = ext.to_string().to_lowercase();

                        if entry
                            .path()
                            .file_name()
                            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
                        {
                            continue;
                        }

                        // 1st pass (graphics folder): only ZIP sources, the
                        // rest was already handled by folder_categories.
                        // 2nd pass (processed folder): playable MOV/WEBM only,
                        // skip the checkerboard preview webm files.
                        if dir_index == 0 {
                            if ext_lower == "zip" {
                                graphic_files.push(entry.path());
                            }
                        } else if ext_lower == "mov" {
                            graphic_files.push(entry.path());
                        }
                    }
                }
            }

            graphic_files.path_sort(natural_lexical_cmp);

            for file in graphic_files {
                let is_process = dir_index == 1;
                let ext_lower = file_extension(&file)
                    .unwrap_or_default()
                    .to_lowercase();
                let prober = MediaProbe::new(&file).await.ok();
                let video = prober.as_ref().and_then(|p| p.video.first());
                let duration = video
                    .and_then(|v| v.duration)
                    .or_else(|| prober.as_ref().and_then(|p| p.format.duration))
                    .unwrap_or_default();
                let frame_rate_text = video.map(|v| v.frame_rate.clone()).unwrap_or_default();
                let fps = parse_frame_rate(&frame_rate_text);
                let asset_id = file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let title = file
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                let relative = file
                    .strip_prefix(&root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");

                let play_path = if is_process {
                    let stem = file.file_stem().unwrap().to_string_lossy();
                    let preview = dir.join(format!("{stem}_preview.webm"));

                    preview
                        .is_file()
                        .then(|| {
                            format!(
                                "/{}",
                                preview
                                    .strip_prefix(&root)
                                    .unwrap_or(&preview)
                                    .to_string_lossy()
                                    .replace('\\', "/")
                            )
                        })
                } else {
                    None
                };

                let is_source_archive = !is_process && ext_lower == "zip";

                assets.push(MediaAsset {
                    asset_id,
                    title,
                    category: "graphics".to_string(),
                    media_type: if is_process && video.is_some() {
                        "video".to_string()
                    } else {
                        "image".to_string()
                    },
                    segments: if duration > 0.0 {
                        (duration / 30.0).ceil() as i64
                    } else {
                        1
                    },
                    duration: smpte_time(duration, fps),
                    duration_seconds: duration,
                    audio: prober.as_ref().map(|p| p.audio.len()).unwrap_or(0) as i64,
                    cc: false,
                    status: if is_source_archive {
                        "processing".to_string()
                    } else {
                        "processed".to_string()
                    },
                    frame_rate: format_frame_rate(&frame_rate_text),
                    path: format!("/{relative}"),
                    play_path,
                    process_path: is_process.then(|| format!("/{relative}")),
                    graphic_source: is_source_archive.then(|| relative.clone()),
                });
            }
        }

        Ok(assets)
    }

    /// Process an animated channel logo from a ZIP archive of image frames.
    ///
    /// It looks for a `*.zip` in `<storage>/logo` (preferring the conventional
    /// `animated_channel_logo.zip` name), extracts the image frame sequence
    /// and writes two files into `<storage>/media/process/logo/`:
    /// - `<stem>.mov` — ProRes 4444 with alpha, used for the on-air overlay
    /// - `<stem>_preview.webm` — VP9 preview over a checkerboard that plays
    ///   in the browser
    ///
    /// Returns `Ok(None)` when no ZIP archive exists in the logo folder.
    pub async fn process_animated_logo(
        &self,
        config: &PlayoutConfig,
    ) -> Result<Option<AnimatedLogoResult>, ServiceError> {
        let root = self.root.read().await.clone();
        let logo_dir = root.join("logo");

        let Some(zip_path) = find_animated_logo_zip(&logo_dir).await? else {
            return Ok(None);
        };

        let target_dir = root.join("media").join("process").join("logo");
        fs::create_dir_all(&target_dir).await?;

        let work_dir = target_dir.join(format!(".work_{}", Uuid::new_v4()));
        fs::create_dir_all(&work_dir).await?;

        let fps = if config.processing.fps > 0.0 {
            config.processing.fps
        } else {
            25.0
        };

        let encode = {
            let work_dir = work_dir.clone();
            let zip_path = zip_path.clone();
            let target_dir = target_dir.clone();

            spawn_blocking(move || encode_logo_movie(&work_dir, &zip_path, &target_dir, fps))
                .await
                .map_err(|e| ServiceError::BadRequest(format!("Logo task failed: {e}")))?
        }?;

        // Remove the extracted frame sequence (~25 GB for a full 1080p run).
        let _ = fs::remove_dir_all(&work_dir).await;

        let mov_relative = encode
            .mov_path
            .strip_prefix(&root)
            .unwrap_or(&encode.mov_path)
            .to_string_lossy()
            .replace('\\', "/");
        let webm_relative = encode
            .webm_path
            .strip_prefix(&root)
            .unwrap_or(&encode.webm_path)
            .to_string_lossy()
            .replace('\\', "/");

        Ok(Some(AnimatedLogoResult {
            path: encode.mov_path.to_string_lossy().to_string(),
            process_path: format!("/{mov_relative}"),
            play_path: format!("/{webm_relative}"),
            status: "processed".to_string(),
        }))
    }

    /// Process a graphic source into an alpha `.mov` used for the on-air bug
    /// overlay.
    ///
    /// The `source` is a path (relative to the channel storage root) to either
    /// a single static image (`.png`/`.tga`/...), a video (`.mp4`/`.mov`/...)
    /// or a `.zip` archive holding a TGA/PNG frame sequence. The result is
    /// written to `<storage>/media/process/graphics/<stem>.mov` (ProRes 4444
    /// with alpha). Files that are already ProRes 4444 `.mov` (alpha) are kept
    /// in place and returned as-is.
    pub async fn process_graphic(
        &self,
        config: &PlayoutConfig,
        source: &str,
    ) -> Result<GraphicsProcessResult, ServiceError> {
        let root = self.root.read().await.clone();
        let (source_abs, _, source_rel) = norm_abs_path(&root, source)?;

        if !source_abs.is_file() {
            return Err(ServiceError::NotFound(format!(
                "Graphic source not found: {source}"
            )));
        }

        let ext = source_abs
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Already alpha-capable MOV/ProRes — nothing to convert.
        if ext == "mov" {
            let rel = source_rel.replace('\\', "/");
            return Ok(GraphicsProcessResult {
                path: source_abs.to_string_lossy().to_string(),
                play_path: String::new(),
                process_path: format!("/{rel}"),
                status: "processed".to_string(),
            });
        }

        let stem = source_abs
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("graphic")
            .to_string();
        let target_dir = root.join("media").join("process").join("graphics");
        fs::create_dir_all(&target_dir).await?;
        let mov_path = target_dir.join(format!("{stem}.mov"));

        let work_dir = target_dir.join(format!(".work_{}", Uuid::new_v4()));
        fs::create_dir_all(&work_dir).await?;

        let fps = if config.processing.fps > 0.0 {
            config.processing.fps
        } else {
            25.0
        };

        if ext == "zip" {
            let (frame_count, frame_ext) = extract_frames(&source_abs, &work_dir)?;

            if frame_count == 0 {
                let _ = fs::remove_dir_all(&work_dir).await;
                return Err(ServiceError::BadRequest(
                    "The ZIP archive does not contain a TGA/PNG frame sequence.".into(),
                ));
            }

            let input_pattern = work_dir.join(format!("frame_%05d.{frame_ext}"));

            run_ffmpeg(&[
                "-hide_banner".to_string(),
                "-y".to_string(),
                "-loglevel".to_string(),
                "error".to_string(),
                "-framerate".to_string(),
                fps.to_string(),
                "-start_number".to_string(),
                "0".to_string(),
                "-i".to_string(),
                input_pattern.to_string_lossy().to_string(),
                "-c:v".to_string(),
                "prores_ks".to_string(),
                "-profile:v".to_string(),
                "4444".to_string(),
                "-vendor".to_string(),
                "apl0".to_string(),
                mov_path.to_string_lossy().to_string(),
            ])?;
        } else {
            // Static image or a short GL graphic: encode as a long single
            // stream with an alpha-capable codec.
            run_ffmpeg(&[
                "-hide_banner".to_string(),
                "-y".to_string(),
                "-loglevel".to_string(),
                "error".to_string(),
                "-i".to_string(),
                source_abs.to_string_lossy().to_string(),
                "-r".to_string(),
                fps.to_string(),
                "-c:v".to_string(),
                "prores_ks".to_string(),
                "-profile:v".to_string(),
                "4444".to_string(),
                "-vendor".to_string(),
                "apl0".to_string(),
                mov_path.to_string_lossy().to_string(),
            ])?;
        }

        let _ = fs::remove_dir_all(&work_dir).await;

        let webm_path = encode_preview_webm(&stem, &mov_path, &target_dir, fps).ok();

        let mov_relative = mov_path
            .strip_prefix(&root)
            .unwrap_or(&mov_path)
            .to_string_lossy()
            .replace('\\', "/");

        let play_path = webm_path.map(|preview| {
            format!(
                "/{}",
                preview
                    .strip_prefix(&root)
                    .unwrap_or(&preview)
                    .to_string_lossy()
                    .replace('\\', "/")
            )
        });

        Ok(GraphicsProcessResult {
            path: mov_path.to_string_lossy().to_string(),
            play_path: play_path.unwrap_or_default(),
            process_path: format!("/{mov_relative}"),
            status: "processed".to_string(),
        })
    }

    pub async fn mkdir(&self, path_obj: &PathObject) -> Result<(), ServiceError> {
        let (path, _, _) = norm_abs_path(&self.root.read().await, &path_obj.source)?;

        if let Err(e) = fs::create_dir_all(&path).await {
            return Err(ServiceError::BadRequest(e.to_string()));
        }

        info!(
            "create folder: <span class=\"log-addr\">{}</span>",
            path.to_string_lossy()
        );

        Ok(())
    }

    pub async fn rename(&self, move_object: &MoveObject) -> Result<MoveObject, ServiceError> {
        let root = self.root.read().await.clone();
        let (source_path, _, _) = norm_abs_path(&root, &move_object.source)?;
        let (mut target_path, _, _) = norm_abs_path(&root, &move_object.target)?;

        if !source_path.exists() {
            return Err(ServiceError::BadRequest("Source file not exist!".into()));
        }

        if (source_path.is_dir() || source_path.is_file())
            && source_path.parent() == Some(&target_path)
        {
            return rename_only(&source_path, &target_path).await;
        }

        if target_path.is_dir() {
            target_path = target_path.join(source_path.file_name().unwrap());
        }

        if target_path.is_file() {
            return Err(ServiceError::BadRequest(
                "Target file already exists!".into(),
            ));
        }

        if source_path.is_file() && target_path.parent().is_some() {
            return rename_only(&source_path, &target_path).await;
        }

        Err(ServiceError::InternalServerError)
    }

    pub async fn remove(&self, source_path: &str, recursive: bool) -> Result<(), ServiceError> {
        let (source, _, _) = norm_abs_path(&self.root.read().await, source_path)?;

        if !source.exists() {
            return Err(ServiceError::BadRequest("Source does not exists!".into()));
        }

        if source.is_dir() {
            let res = if recursive {
                fs::remove_dir_all(source).await
            } else {
                fs::remove_dir(source).await
            };

            match res {
                Ok(..) => return Ok(()),
                Err(e) => {
                    error!("{e}");
                    return Err(ServiceError::BadRequest(
                        "Delete folder failed! (Folder must be empty)".into(),
                    ));
                }
            }
        }

        if source.is_file() {
            match fs::remove_file(source).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!("{e}");
                    return Err(ServiceError::BadRequest("Delete file failed!".into()));
                }
            };
        }

        Err(ServiceError::InternalServerError)
    }

    pub async fn upload(
        &self,
        mut data: Multipart,
        path: &Path,
        is_abs: bool,
    ) -> Result<(), ServiceError> {
        while let Some(mut field) = data.try_next().await? {
            let content_disposition = field.content_disposition().ok_or("No content")?;
            debug!("{content_disposition}");
            let rand_string: String = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(20)
                .map(char::from)
                .collect();
            let filename = content_disposition
                .get_filename()
                .map_or_else(|| rand_string.to_string(), sanitize_filename::sanitize);

            let filepath = if is_abs {
                path.to_path_buf()
            } else {
                let (target_path, _, _) =
                    norm_abs_path(&self.root.read().await, &path.to_string_lossy())?;

                target_path.join(filename)
            };

            // INFO: File exist check should be enough because file size and content length are different.
            // The error catching in the loop should normally prevent unfinished files from existing on disk.
            // If this is not enough, a second check can be implemented: is_close(file_size as i64, size as i64, 1000)
            if filepath.is_file() {
                return Err(ServiceError::Conflict("Target already exists!".into()));
            }

            let mut f = fs::File::create(&filepath).await?;

            loop {
                match field.try_next().await {
                    Ok(Some(chunk)) => {
                        f = f.write_all(&chunk).await.map(|_| f)?;
                    }

                    Ok(None) => break,

                    Err(e) => {
                        if e.to_string().contains("stream is incomplete") {
                            info!("Delete non finished file: {filepath:?}");

                            tokio::fs::remove_file(filepath).await?;
                        }

                        return Err(e.into());
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn watchman(
        &self,
        config: PlayoutConfig,
        is_alive: Arc<AtomicBool>,
        sources: Arc<Mutex<Vec<Media>>>,
    ) {
        if let Some(old_handle) = self.watch_handler.lock().await.take() {
            old_handle.abort();
        }

        let handle = tokio::spawn(watch(config, is_alive, sources));

        *self.watch_handler.lock().await = Some(handle);
    }

    pub async fn stop_watch(&self) {
        if let Some(handle) = self.watch_handler.lock().await.take() {
            handle.abort();
        }
    }

    pub async fn fill_filler_list(
        &self,
        config: &PlayoutConfig,
        fillers: Option<Arc<Mutex<Vec<Media>>>>,
    ) -> Vec<Media> {
        let id = config.general.channel_id;
        let mut filler_list = vec![];
        let filler_path = &config.storage.filler_path;

        if filler_path.is_dir() {
            let config_clone = config.clone();
            let mut index = 0;
            let mut entries = WalkDir::new(&config_clone.storage.filler_path);

            while let Some(Ok(entry)) = entries.next().await {
                if entry.path().is_file() && include_file_extension(config, &entry.path()) {
                    let mut media = Media::new(index, &entry.path().to_string_lossy(), false).await;

                    if fillers.is_none()
                        && let Err(e) = media.add_probe(false).await
                    {
                        error!(target: Target::file_mail(), channel = id; "{e:?}");
                    };

                    filler_list.push(media);
                    index += 1;
                }
            }

            if config.storage.shuffle {
                let mut rng = StdRng::from_os_rng();

                filler_list.shuffle(&mut rng);
            } else {
                filler_list.sort_by(|d1, d2| natural_lexical_cmp(&d1.source, &d2.source));
            }

            for (index, item) in filler_list.iter_mut().enumerate() {
                item.index = Some(index);
            }

            if let Some(f) = fillers.as_ref() {
                f.lock().await.clone_from(&filler_list);
            }
        } else if filler_path.is_file() {
            let mut media =
                Media::new(0, &config.storage.filler_path.to_string_lossy(), false).await;

            if fillers.is_none()
                && let Err(e) = media.add_probe(false).await
            {
                error!(target: Target::file_mail(), channel = id; "{e:?}");
            };

            filler_list.push(media);

            if let Some(f) = fillers.as_ref() {
                f.lock().await.clone_from(&filler_list);
            }
        }

        filler_list
    }

    pub async fn copy_assets(&self) -> Result<(), std::io::Error> {
        let root = self.root.read().await.clone();
        if root.is_dir() {
            let logo_folder = root.join("logo");
            let text_folder = root.join("text");
            let subtitle_folder = root.join("subtitles");
            let mut dummy_source = Path::new("/usr/share/ffplayout/dummy.vtt");
            let mut font_source = Path::new("/usr/share/ffplayout/DejaVuSans.ttf");
            let mut logo_source = Path::new("/usr/share/ffplayout/logo.png");

            if !dummy_source.is_file() {
                dummy_source = Path::new("./assets/dummy.vtt");
            }
            if !font_source.is_file() {
                font_source = Path::new("./assets/DejaVuSans.ttf");
            }
            if !logo_source.is_file() {
                logo_source = Path::new("./assets/logo.png");
            }

            fs::create_dir_all(&logo_folder).await?;
            fs::create_dir_all(&text_folder).await?;
            fs::create_dir_all(&subtitle_folder).await?;

            let dummy_target = subtitle_folder.join("dummy.vtt");
            let font_target = text_folder.join("DejaVuSans.ttf");
            let logo_target = logo_folder.join("logo.png");

            // never overwrite files that already exists
            if !logo_target.is_file() {
                fs::copy(&logo_source, &logo_target).await?;
            }

            if !font_target.is_file() {
                fs::copy(&font_source, &font_target).await?;
            }

            if !dummy_target.is_file() {
                fs::copy(&dummy_source, &dummy_target).await?;
            }

            #[cfg(target_family = "unix")]
            {
                let uid = nix::unistd::Uid::current();
                let parent_owner = root.metadata().unwrap().uid();

                if uid.is_root() && uid.to_string() != parent_owner.to_string() {
                    let user = nix::unistd::User::from_uid(parent_owner.into())
                        .unwrap_or_default()
                        .unwrap();

                    for folder in [&logo_folder, &text_folder, &subtitle_folder] {
                        nix::unistd::chown(folder, Some(user.uid), Some(user.gid))?;
                    }

                    if logo_target.is_file() {
                        nix::unistd::chown(&logo_target, Some(user.uid), Some(user.gid))?;
                    }
                    if font_target.is_file() {
                        nix::unistd::chown(&font_target, Some(user.uid), Some(user.gid))?;
                    }
                    if dummy_target.is_file() {
                        nix::unistd::chown(&dummy_target, Some(user.uid), Some(user.gid))?;
                    }
                }
            }
        } else {
            error!("Storage path {:?} not exists!", self.root);
        }

        Ok(())
    }
}

async fn rename_only(source: &PathBuf, target: &PathBuf) -> Result<MoveObject, ServiceError> {
    match fs::rename(source, target).await {
        Ok(_) => Ok(MoveObject {
            source: source
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            target: target
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }),
        Err(e) => {
            error!("{e}");
            copy_and_delete(source, target).await
        }
    }
}

async fn copy_and_delete(source: &PathBuf, target: &PathBuf) -> Result<MoveObject, ServiceError> {
    match fs::copy(&source, &target).await {
        Ok(_) => {
            if let Err(e) = fs::remove_file(source).await {
                error!("{e}");
                return Err(ServiceError::BadRequest(
                    "Removing File not possible!".into(),
                ));
            };

            Ok(MoveObject {
                source: source
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                target: target
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            })
        }
        Err(e) => {
            error!("{e}");
            Err(ServiceError::BadRequest("Error in file copy!".into()))
        }
    }
}

/// Find the animated logo ZIP in the logo folder, preferring the conventional
/// `animated_channel_logo.zip` name.
async fn find_animated_logo_zip(logo_dir: &Path) -> Result<Option<PathBuf>, ServiceError> {
    if !logo_dir.is_dir() {
        return Ok(None);
    }

    let mut zips = vec![];
    let mut entries = fs::read_dir(logo_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if file_extension(&path).is_some_and(|e| e.eq_ignore_ascii_case("zip")) {
            zips.push(path);
        }
    }

    if zips.is_empty() {
        return Ok(None);
    }

    zips.path_sort(natural_lexical_cmp);

    if let Some(pos) = zips
        .iter()
        .position(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("animated_channel_logo.zip"))
        })
    {
        return Ok(zips.into_iter().nth(pos));
    }

    Ok(zips.into_iter().next())
}

/// Encode the extracted frame sequence into a ProRes 4444 alpha `.mov` and a
/// browser-playable VP9 preview `.webm` in `target_dir`.
fn encode_logo_movie(
    work_dir: &Path,
    zip_path: &Path,
    target_dir: &Path,
    fps: f64,
) -> Result<EncodeResult, ServiceError> {
    let (frame_count, ext) = extract_frames(zip_path, work_dir)?;

    if frame_count == 0 {
        return Err(ServiceError::BadRequest(
            "The ZIP archive does not contain a TGA/PNG frame sequence.".into(),
        ));
    }

    let stem = zip_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("animated_logo")
        .to_string();
    let mov_path = target_dir.join(format!("{stem}.mov"));
    let input_pattern = work_dir.join(format!("frame_%05d.{ext}"));

    // The frame count distributes well for shorter loops; ProRes 4444 is the
    // de-facto standard for broadcast alpha video.
    run_ffmpeg(&[
        "-hide_banner".to_string(),
        "-y".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-framerate".to_string(),
        fps.to_string(),
        "-start_number".to_string(),
        "0".to_string(),
        "-i".to_string(),
        input_pattern.to_string_lossy().to_string(),
        "-c:v".to_string(),
        "prores_ks".to_string(),
        "-profile:v".to_string(),
        "4444".to_string(),
        "-vendor".to_string(),
        "apl0".to_string(),
        mov_path.to_string_lossy().to_string(),
    ])?;

    // Browser-friendly preview: the logo over a checkerboard (so transparent
    // areas are clearly visible) scaled down to 960x540.
    let webm_path = encode_preview_webm(&stem, &mov_path, &target_dir, fps)?;

    Ok(EncodeResult { mov_path, webm_path })
}

/// Render a browser-playable VP9 preview of an alpha MOV: the graphic over a
/// checkerboard so transparent areas are clearly visible, scaled down to
/// 960x540. Returns the written `.webm` path.
fn encode_preview_webm(
    stem: &str,
    mov_path: &Path,
    target_dir: &Path,
    fps: f64,
) -> Result<PathBuf, ServiceError> {
    let webm_path = target_dir.join(format!("{stem}_preview.webm"));
    let checkerboard = format!(
        "color=c=black:s=960x540:r={fps}:d=600,geq=r='if(eq(mod(floor(X/20)+floor(Y/20),2),0),200,64)':g='if(eq(mod(floor(X/20)+floor(Y/20),2),0),200,64)':b='if(eq(mod(floor(X/20)+floor(Y/20),2),0),200,64)'"
    );
    let filter_complex = format!(
        "{checkerboard}[bg];[0:v]format=rgba,scale=960:540[fg];[bg][fg]overlay=shortest=1:format=auto,format=yuv420p[v]"
    );

    run_ffmpeg(&[
        "-hide_banner".to_string(),
        "-y".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-i".to_string(),
        mov_path.to_string_lossy().to_string(),
        "-filter_complex".to_string(),
        filter_complex,
        "-map".to_string(),
        "[v]".to_string(),
        "-c:v".to_string(),
        "libvpx-vp9".to_string(),
        "-b:v".to_string(),
        "800k".to_string(),
        "-deadline".to_string(),
        "realtime".to_string(),
        "-cpu-used".to_string(),
        "8".to_string(),
        "-row-mt".to_string(),
        "1".to_string(),
        webm_path.to_string_lossy().to_string(),
    ])?;

    Ok(webm_path)
}

/// Extract the TGA/PNG frame sequence from a ZIP archive into `work_dir` as
/// zero-padded `frame_00000.<ext>` files. Returns the frame count and the
/// image extension.
fn extract_frames(zip_path: &Path, work_dir: &Path) -> Result<(usize, String), ServiceError> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| ServiceError::BadRequest(format!("Can not open ZIP archive: {e}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| ServiceError::BadRequest(format!("Invalid ZIP archive: {e}")))?;

    let mut frames = vec![];
    let mut ext = "tga".to_string();

    for archive_index in 0..archive.len() {
        let entry = archive
            .by_index(archive_index)
            .map_err(|e| ServiceError::BadRequest(format!("ZIP read error: {e}")))?;

        if !entry.is_file() {
            continue;
        }

        let name = entry.name().to_string();
        let lower = name.to_lowercase();

        if lower.ends_with(".tga") || lower.ends_with(".png") {
            if let Some(frame_index) = parse_frame_index(&name) {
                if lower.ends_with(".png") {
                    ext = "png".to_string();
                }

                frames.push((frame_index, archive_index));
            }
        }
    }

    if frames.is_empty() {
        return Ok((0, ext));
    }

    frames.sort_by_key(|f| f.0);

    for (order, (_, archive_index)) in frames.iter().enumerate() {
        let mut entry = archive
            .by_index(*archive_index)
            .map_err(|e| ServiceError::BadRequest(format!("ZIP read error: {e}")))?;
        let target = work_dir.join(format!("frame_{order:05}.{ext}"));
        let mut out = std::fs::File::create(&target)
            .map_err(|e| ServiceError::BadRequest(format!("Can not write {target:?}: {e}")))?;

        io::copy(&mut entry, &mut out)
            .map_err(|e| ServiceError::BadRequest(format!("ZIP extract error: {e}")))?;
    }

    Ok((frames.len(), ext))
}

/// Extract a trailing numeric frame index from a frame file name
/// (e.g. `Aflam logo bug v2 HD_00042.tga` -> 42).
fn parse_frame_index(name: &str) -> Option<u64> {
    let stem = name.rsplitn(2, '.').last().unwrap_or(name);

    let digits: String = stem
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

/// Run ffmpeg with the given arguments and check for success.
fn run_ffmpeg(args: &[String]) -> Result<(), ServiceError> {
    let output = std::process::Command::new("ffmpeg")
        .args(args)
        .output()
        .map_err(|e| ServiceError::BadRequest(format!("Can not start ffmpeg: {e}")))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(ServiceError::BadRequest(format!(
        "ffmpeg failed: {}",
        stderr.chars().take(500).collect::<String>()
    )))
}

/// Map a library category to a human readable media type.
fn category_to_type(category: &str) -> String {
    match category {
        "videos" => "MEDIA".to_string(),
        "graphics" => "GRAPHIC".to_string(),
        "audio" => "AUDIO".to_string(),
        "rescue" => "RESCUE".to_string(),
        "subtitles" => "SUBTITLE".to_string(),
        _ => "MEDIA".to_string(),
    }
}

/// Parse an ffprobe frame rate string ("25/1", "30000/1001") into a float.
fn parse_frame_rate(text: &str) -> f64 {
    if let Some((num, den)) = text.split_once('/') {
        let n: f64 = num.trim().parse().unwrap_or(0.0);
        let d: f64 = den.trim().parse().unwrap_or(1.0);

        if d > 0.0 { n / d } else { 0.0 }
    } else {
        text.trim().parse().unwrap_or(0.0)
    }
}

/// Render a frame rate for display ("25/1" -> "25", "30000/1001" -> "29.97").
fn format_frame_rate(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    if let Some((num, den)) = text.split_once('/') {
        let n: f64 = num.trim().parse().unwrap_or(0.0);
        let d: f64 = den.trim().parse().unwrap_or(1.0);

        if d > 0.0 && n > 0.0 {
            let v = n / d;
            return format!("{v:.2}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
        }
    }

    text.to_string()
}

/// Format a duration (seconds) as SMPTE timecode HH:MM:SS:FF.
fn smpte_time(duration: f64, fps: f64) -> String {
    let total = duration.max(0.0);
    let hours = (total / 3600.0).floor() as u64;
    let minutes = ((total % 3600.0) / 60.0).floor() as u64;
    let seconds = (total % 60.0).floor() as u64;
    let frames = if fps > 0.0 {
        ((total % 1.0) * fps).round() as u64
    } else {
        0
    };

    format!("{hours:02}:{minutes:02}:{seconds:02}:{frames:02}")
}
