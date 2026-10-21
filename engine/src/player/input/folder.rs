use std::sync::atomic::Ordering;

use async_walkdir::WalkDir;
use regex::Regex;

use log::*;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use tokio_stream::StreamExt;

use crate::file::preprocess::{has_preprocessed, resolved_source};
use crate::player::{
    controller::ChannelManager,
    utils::{Media, include_file_extension, time_in_seconds},
};
use crate::utils::{config::PlayoutConfig, logging::Target};

/// Folder Sources
///
/// Like playlist source, we create here a folder list for iterate over it.
#[derive(Debug, Clone)]
pub struct FolderSource {
    manager: ChannelManager,
    current_node: Media,
}

impl FolderSource {
    pub async fn new(config: &PlayoutConfig, manager: ChannelManager) -> Self {
        let id = config.general.channel_id;
        let mut path_list = vec![];
        let mut media_list = vec![];
        let mut index: usize = 0;

        if !config.storage.paths.is_empty() && config.general.generate.is_some() {
            path_list.extend(&config.storage.paths);
        } else {
            path_list.push(&config.channel.storage);
        }

        if let Some(dates) = &config.general.generate {
            debug!(target: Target::file_mail(), channel = id;
                "generate: {dates:?}, paths: {path_list:?}"
            );
        }

        // Compile ad pattern regex if configured
        let ad_re = if !config.storage.ad_pattern.is_empty() {
            Regex::new(&config.storage.ad_pattern).ok()
        } else {
            None
        };

        for path in &path_list {
            if !path.is_dir() {
                error!(target: Target::file_mail(), channel = id; "Path not exists: <span class=\"log-addr\">{path:?}</span>");
            }

            let mut entries = WalkDir::new(path);

            while let Some(Ok(entry)) = entries.next().await {
                if entry.path().is_file() && include_file_extension(config, &entry.path()) {
                    // Check if a preprocessed .ts version exists
                    let (resolved_path, is_preprocessed) = resolved_source(config, &entry.path());
                    let mut media = Media::new(0, &resolved_path.to_string_lossy(), false).await;
                    media.preprocessed = is_preprocessed;

                    // Detect ad files by filename regex pattern
                    if let Some(ref re) = ad_re {
                        let filename = entry
                            .path()
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if re.is_match(&filename) {
                            media.category = "advertisement".to_string();
                        }
                    }

                    if is_preprocessed {
                        debug!(target: Target::file_mail(), channel = id;
                            "Using preprocessed: {} -> {}",
                            entry.path().display(),
                            resolved_path.display()
                        );
                    }

                    media_list.push(media);
                }
            }
        }

        if media_list.is_empty() {
            error!(target: Target::file_mail(), channel = id;
                "no playable files found under: <span class=\"log-addr\">{:?}</span>",
                path_list
            );
        }

        if config.storage.shuffle {
            info!(target: Target::file_mail(), channel = id; "Shuffle files");
            let mut rng = StdRng::from_os_rng();
            media_list.shuffle(&mut rng);
        } else {
            media_list.sort_by(|d1, d2| d1.source.cmp(&d2.source));
        }

        // Set last_ad / next_ad flags based on adjacent items in the sorted list.
        // This mirrors the playlist source behavior so SCTE-35 detection works.
        for i in 0..media_list.len() {
            if media_list[i].category == "advertisement" {
                if i + 1 < media_list.len() && media_list[i + 1].category == "advertisement" {
                    media_list[i].next_ad = true;
                }
                if i > 0 && media_list[i - 1].category == "advertisement" {
                    media_list[i].last_ad = true;
                }
            }
        }

        for item in &mut media_list {
            item.index = Some(index);

            index += 1;
        }

        *manager.current_list.lock().await = media_list;

        Self {
            manager,
            current_node: Media::default(),
        }
    }

    pub async fn from_list(manager: &ChannelManager, list: Vec<Media>) -> Self {
        *manager.current_list.lock().await = list;

        Self {
            manager: manager.clone(),
            current_node: Media::default(),
        }
    }

    async fn shuffle(&mut self) {
        let mut rng = StdRng::from_os_rng();
        let mut nodes = self.manager.current_list.lock().await;

        nodes.shuffle(&mut rng);

        for (index, item) in nodes.iter_mut().enumerate() {
            item.index = Some(index);
        }
    }

    async fn sort(&mut self) {
        let mut nodes = self.manager.current_list.lock().await;

        nodes.sort_by(|d1, d2| d1.source.cmp(&d2.source));

        for (index, item) in nodes.iter_mut().enumerate() {
            item.index = Some(index);
        }
    }
}

/// Create iterator for folder source
impl FolderSource {
    pub async fn next(&mut self) -> Option<Media> {
        let config = self.manager.config.read().await.clone();
        let id = config.general.id;

        if self.manager.current_index.load(Ordering::SeqCst)
            < self.manager.current_list.lock().await.len()
        {
            let i = self.manager.current_index.load(Ordering::SeqCst);
            self.current_node = self.manager.current_list.lock().await[i].clone();
            let _ = self.current_node.add_probe(false).await.ok();
            self.current_node
                .add_filter(&config, &self.manager.filter_chain)
                .await;
            self.current_node.begin = Some(time_in_seconds(&config.channel.timezone));
            self.manager.current_index.fetch_add(1, Ordering::SeqCst);
        } else {
            if config.storage.shuffle {
                if config.general.generate.is_none() {
                    info!(target: Target::file_mail(), channel = id; "Shuffle files");
                }

                self.shuffle().await;
            } else {
                if config.general.generate.is_none() {
                    info!(target: Target::file_mail(), channel = id; "Sort files");
                }

                self.sort().await;
            }

            self.current_node = match self.manager.current_list.lock().await.first() {
                Some(m) => m.clone(),
                None => return None,
            };
            let _ = self.current_node.add_probe(false).await.ok();
            self.current_node
                .add_filter(&config, &self.manager.filter_chain)
                .await;
            self.current_node.begin = Some(time_in_seconds(&config.channel.timezone));
            self.manager.current_index.store(1, Ordering::SeqCst);
        }

        Some(self.current_node.clone())
    }
}
