/*
This module write the files compression directly to a hls (m3u8) playlist,
without pre- and post-processing.

Example config:

out:
    output_param: >-
        ...

        -flags +cgop
        -f hls
        -hls_time 6
        -hls_list_size 600
        -hls_flags append_list+delete_segments+omit_endlist+program_date_time
        -hls_segment_filename /var/www/html/live/stream-%d.ts /var/www/html/live/stream.m3u8

*/

use std::{
    path::PathBuf,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::SystemTime,
};

use log::*;
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::{Duration, sleep},
};

use crate::player::output::scte35;
use crate::utils::{logging::log_line, task_runner};
use crate::vec_strings;
use crate::{
    player::{
        controller::{ChannelManager, ProcessUnit::*},
        input::source_generator,
        utils::{
            Media, get_delta, insert_readrate, is_free_tcp_port, prepare_output_cmd, sec_to_time,
            stderr_reader, valid_stream,
        },
    },
    utils::{
        errors::ServiceError,
        logging::{Target, fmt_cmd},
    },
};

/// Periodically checks if HLS segments are still being updated.
/// If no new segment is written for longer than `hls_time * 3`, returns an error.
pub async fn hls_watchdog(
    channel_id: i32,
    segment_dir: PathBuf,
    hls_time: Duration,
    is_alive: Arc<AtomicBool>,
) -> Result<(), ServiceError> {
    let mut init = true;

    loop {
        if init {
            sleep(hls_time * 3).await;
            init = false;
        } else {
            sleep(hls_time).await;
        }

        if !is_alive.load(Ordering::SeqCst) {
            break;
        }

        let mut newest: Option<SystemTime> = None;
        let mut entries = fs::read_dir(&segment_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(meta) = entry.metadata().await
                && let Ok(modified) = meta.modified()
                && newest.map(|t| modified > t).unwrap_or(true)
            {
                newest = Some(modified);
            }
        }

        if let Some(last_mod) = newest {
            let age = SystemTime::now()
                .duration_since(last_mod)
                .unwrap_or_default();

            if age > hls_time * 3 {
                error!(target: Target::file_mail(), channel = channel_id;
                    "HLS segment write timeout! Last update: <span class=\"log-number\">{:.3}s</span>", age.as_secs_f32()
                );
                return Err(ServiceError::Conflict("Timeout".to_string()));
            }
        }
    }

    Ok(())
}

/// Ingest Server for HLS
async fn ingest_writer(manager: ChannelManager) -> Result<(), ServiceError> {
    let config = manager.config.read().await.clone();
    let id = config.general.channel_id;
    let playlist_init = manager.list_init.clone();
    let chain = manager.filter_chain.clone();
    let stream_input = config.ingest.input_cmd.clone().unwrap();
    let mut server_prefix = vec_strings!["-hide_banner", "-nostats", "-v", "level+info"];
    let mut dummy_media = Media::new(0, "Live Stream", false).await;

    dummy_media.unit = Ingest;
    dummy_media.add_filter(&config, &chain).await;

    let is_alive = manager.is_alive.clone();
    let ingest_is_alive = manager.ingest_is_alive.clone();

    if let Some(ingest_input_cmd) = &config.advanced.ingest.input_cmd {
        server_prefix.append(&mut ingest_input_cmd.clone());
    }

    server_prefix.append(&mut stream_input.clone());

    if config.processing.vtt_enable {
        let vtt_dummy = config
            .channel
            .storage
            .join(config.processing.vtt_dummy.clone().unwrap_or_default());

        if vtt_dummy.is_file() {
            server_prefix.append(&mut vec_strings!["-i", vtt_dummy.to_string_lossy()]);
        }
    }

    let server_cmd = prepare_output_cmd(&config, server_prefix.clone(), &dummy_media.filter);
    let mut is_running;

    if let Some(url) = stream_input.iter().find(|s| s.contains("://")) {
        for num in 0..5 {
            if is_free_tcp_port(url) {
                break;
            }

            error!(target: Target::file_mail(), channel = id; "Address <span class=\"log-addr\">{url}</span> already in use!");

            manager.stop(Ingest).await;

            if num >= 4 {
                return Err(ServiceError::Conflict(
                    "Can't run ingest server!".to_string(),
                ));
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        info!(target: Target::file_mail(), channel = id; "Start ingest server, listening on: <span class=\"log-addr\">{url}</span>");
    };

    debug!(target: Target::file_mail(), channel = id;
        "Server CMD: <span class=\"log-cmd\">ffmpeg {}</span>",
        fmt_cmd(&server_cmd)
    );

    loop {
        let mut level = &config.logging.ffmpeg_level;
        let mut server_proc = Command::new("ffmpeg")
            .args(server_cmd.clone())
            .current_dir(&config.channel.public)
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let server_err = BufReader::new(server_proc.stderr.take().unwrap());
        let mut lines = server_err.lines();

        *manager.ingest.lock().await = Some(server_proc);
        is_running = false;

        while let Some(line) = lines.next_line().await? {
            if line.contains("rtmp")
                && (line.contains("Unexpected stream") || line.contains("App field don't match up"))
                && !valid_stream(&line)
            {
                warn!(target: Target::file_mail(), channel = id; "Unexpected ingest stream: {line}");

                manager.stop(Ingest).await;
            } else if !is_running && line.contains("Input #0") {
                level = &config.logging.ingest_level;
                ingest_is_alive.store(true, Ordering::SeqCst);
                playlist_init.store(true, Ordering::SeqCst);
                is_running = true;

                info!(target: Target::file_mail(), channel = id; "Switch from {} to live ingest", config.processing.mode);

                manager.stop(Decoder).await;
            }

            log_line(id, &line, level);
        }

        if ingest_is_alive.load(Ordering::SeqCst) {
            info!(target: Target::file_mail(), channel = id; "Switch from live ingest to {}", config.processing.mode);
        }

        ingest_is_alive.store(false, Ordering::SeqCst);

        manager.wait(Ingest).await;

        if !is_alive.load(Ordering::SeqCst) {
            break;
        }
    }

    Ok(())
}

async fn write(manager: &ChannelManager, ff_log_format: &str) -> Result<(), ServiceError> {
    let config = manager.config.read().await.clone();
    let get_source = source_generator(manager.clone());
    let ingest_is_alive = manager.ingest_is_alive.clone();
    let id = config.general.channel_id;
    let current_media = manager.current_media.clone();
    let is_alive = manager.is_alive.clone();

    let mut get_source = get_source.await;
    // Continuous channel PTS in 90kHz ticks for HLS CUE-OUT/CUE-IN tags.
    let mut channel_pts: u64 = 0;
    // Track whether we need to re-inject CUE-IN after the first program clip
    // following an ad break (the program clip's ffmpeg overwrites the m3u8).
    let mut cue_in_reinject: Option<(String, String)> = None; // (m3u8_path, tag_line)

    while let Some(node) = get_source.next().await {
        *current_media.lock().await = Some(node.clone());
        let ignore = config.logging.ignore_lines.clone();

        if !is_alive.load(Ordering::SeqCst) {
            break;
        }

        if node.skip {
            continue;
        }

        let mut cmd = match &node.cmd {
            Some(cmd) => cmd.clone(),
            None => break,
        };

        // SCTE-35: detect ad break boundaries for HLS CUE-OUT/CUE-IN injection.
        // Use the current clip's category to determine if it IS an ad,
        // then use last_ad/next_ad to determine if it's the first/last in a block.
        let is_ad = node.category == "advertisement";
        let is_ad_start = is_ad && !node.last_ad;
        let is_ad_end = is_ad && !node.next_ad;
        let clip_duration = node.out - node.seek;

        if is_ad_start || is_ad_end {
            debug!(target: Target::file_mail(), channel = id;
                "SCTE-35 HLS: ad boundary detected: category={}, last_ad={}, next_ad={}, is_ad_start={}, is_ad_end={}, clip_duration={:.2}s, channel_pts={}",
                node.category, node.last_ad, node.next_ad, is_ad_start, is_ad_end, clip_duration, channel_pts);
        }

        // SCTE-35: inject CUE-OUT tag BEFORE the ad clip starts
        if is_ad_start {
            if let Some(ref output_cmd) = config.output.output_cmd {
                if let Some(m3u8_path) = scte35::find_m3u8_path(output_cmd) {
                    let break_duration_90k = (clip_duration * 90_000.0) as u64;
                    let tag =
                        scte35::build_cue_out_tag(clip_duration, channel_pts, break_duration_90k);
                    let path = std::path::Path::new(&m3u8_path);
                    match scte35::inject_tag_atomic(path, &tag).await {
                        Ok(()) => {
                            let pts_secs = channel_pts as f64 / 90_000.0;
                            info!(target: Target::file_mail(), channel = id;
                                "SCTE-35 HLS: CUE-OUT -> {m3u8_path} (PTS {:.2}s, break {}ms)",
                                pts_secs, break_duration_90k / 90);

                            // Write JSON log to scte folder
                            let event = scte35::Scte35Event {
                                timestamp: String::new(),
                                event: "cue_out".to_string(),
                                pts: pts_secs,
                                break_duration: Some(clip_duration),
                                hex: scte35::build_splice_insert_hex(
                                    scte35::Scte35Command::SpliceStart,
                                    channel_pts,
                                    Some(break_duration_90k),
                                ),
                                base64: {
                                    let bytes = scte35::build_splice_insert_bytes(
                                        scte35::Scte35Command::SpliceStart,
                                        channel_pts,
                                        Some(break_duration_90k),
                                    );
                                    scte35::base64_encode(&bytes)
                                },
                                output: "hls".to_string(),
                                playlist: Some(m3u8_path),
                            };
                            if let Err(e) =
                                scte35::log_scte35_event(&config.channel.logs, id, event).await
                            {
                                warn!(target: Target::file_mail(), channel = id;
                                    "SCTE-35: Failed to write log event: {e}");
                            }

                            // Sleep to give HLS players time to poll the updated
                            // m3u8 before the ad clip's ffmpeg rewrites it.
                            sleep(Duration::from_secs(2)).await;
                        }
                        Err(e) => {
                            error!(target: Target::file_mail(), channel = id;
                                "SCTE-35 HLS: Failed to inject CUE-OUT into {m3u8_path}: {e}");
                        }
                    }
                } else {
                    warn!(target: Target::file_mail(), channel = id;
                        "SCTE-35 HLS: No m3u8 path found in output_cmd for CUE-OUT");
                }
            } else {
                warn!(target: Target::file_mail(), channel = id;
                    "SCTE-35 HLS: output_cmd is None, cannot inject CUE-OUT");
            }
        }

        info!(target: Target::file_mail(), channel = id;
            "Play for <span class=\"log-number\">{}</span>: <span class=\"log-addr\">{}</span>",
            sec_to_time(clip_duration),
            node.source
        );

        if config.task.enable {
            if config.task.path.is_file() {
                let manager3 = manager.clone();

                tokio::spawn(task_runner::run(manager3));
            } else {
                error!(target: Target::file_mail(), channel = id;
                    "<span class=\"log-cmd\">{:?}</span> executable not exists!",
                    config.task.path
                );
            }
        }

        let mut dec_prefix = vec_strings!["-hide_banner", "-nostats", "-v", &ff_log_format];

        if let Some(decoder_input_cmd) = &config.advanced.decoder.input_cmd {
            dec_prefix.append(&mut decoder_input_cmd.clone());
        }

        let mut read_rate = 1.0;

        if let Some(begin) = &node.begin {
            let (delta, _) = get_delta(&config, begin);
            let duration = node.out - node.seek;
            let speed = duration / (duration + delta);

            if node.seek == 0.0
                && speed > 0.0
                && speed < 1.3
                && delta < config.general.stop_threshold
            {
                read_rate = speed;
            }
        }

        insert_readrate(&config.general.ffmpeg_options, &mut cmd, read_rate);

        dec_prefix.append(&mut cmd);
        let dec_cmd = prepare_output_cmd(&config, dec_prefix, &node.filter);

        debug!(target: Target::file_mail(), channel = id;
            "HLS writer CMD: <span class=\"log-cmd\">ffmpeg {}</span>",
            fmt_cmd(&dec_cmd)
        );

        let mut dec_proc = Command::new("ffmpeg")
            .args(dec_cmd)
            .current_dir(&config.channel.public)
            .kill_on_drop(true)
            .stderr(Stdio::piped())
            .spawn()?;

        let dec_err = BufReader::new(dec_proc.stderr.take().unwrap());
        *manager.decoder.lock().await = Some(dec_proc);

        stderr_reader(dec_err, ignore, Decoder, id).await?;

        manager.wait(Decoder).await;

        // SCTE-35: re-inject CUE-IN if the previous clip was the last ad.
        // The first program clip's ffmpeg overwrites the m3u8, destroying
        // our CUE-IN tag. Re-inject it before the first EXTINF.
        if !is_ad {
            if let Some((ref m3u8_path, ref tag)) = cue_in_reinject {
                let path = std::path::Path::new(m3u8_path);
                if let Err(e) = scte35::inject_tag_at(path, tag, Some("#EXTINF")).await {
                    warn!(target: Target::file_mail(), channel = id;
                        "SCTE-35 HLS: Failed to re-inject CUE-IN into {m3u8_path}: {e}");
                } else {
                    debug!(target: Target::file_mail(), channel = id;
                        "SCTE-35 HLS: CUE-IN re-injected -> {m3u8_path}");
                }
                cue_in_reinject = None;
            }
        }

        // SCTE-35: re-inject CUE-OUT after ad clip exits.
        // The ad clip's ffmpeg rewrites the m3u8 when writing segments,
        // destroying our earlier CUE-OUT tag. Re-inject it now, placed
        // before the first #EXTINF so players see it before ad segments.
        if is_ad_start || (is_ad && !node.last_ad) {
            if let Some(ref output_cmd) = config.output.output_cmd {
                if let Some(m3u8_path) = scte35::find_m3u8_path(output_cmd) {
                    let break_duration_90k = (clip_duration * 90_000.0) as u64;
                    let tag =
                        scte35::build_cue_out_tag(clip_duration, channel_pts, break_duration_90k);
                    let path = std::path::Path::new(&m3u8_path);
                    if let Err(e) = scte35::inject_tag_at(path, &tag, Some("#EXTINF")).await {
                        warn!(target: Target::file_mail(), channel = id;
                            "SCTE-35 HLS: Failed to re-inject CUE-OUT into {m3u8_path}: {e}");
                    } else {
                        debug!(target: Target::file_mail(), channel = id;
                            "SCTE-35 HLS: CUE-OUT re-injected -> {m3u8_path}");
                    }
                }
            }
        }

        // SCTE-35: inject CUE-IN tag AFTER the last ad clip finishes.
        // PTS must be at the END of the ad clip (where program resumes).
        if is_ad_end {
            if let Some(ref output_cmd) = config.output.output_cmd {
                if let Some(m3u8_path) = scte35::find_m3u8_path(output_cmd) {
                    // CUE-IN PTS = start of ad + duration of ad = end of ad
                    let cue_in_pts = channel_pts + (clip_duration * 90_000.0) as u64;
                    let tag = scte35::build_cue_in_tag(cue_in_pts, clip_duration);
                    let path = std::path::Path::new(&m3u8_path);
                    match scte35::inject_tag_atomic(path, &tag).await {
                        Ok(()) => {
                            let pts_secs = cue_in_pts as f64 / 90_000.0;
                            info!(target: Target::file_mail(), channel = id;
                                "SCTE-35 HLS: CUE-IN -> {m3u8_path} (PTS {:.2}s, dur {:.2}s)",
                                pts_secs, clip_duration);

                            // Write JSON log to scte folder
                            let event = scte35::Scte35Event {
                                timestamp: String::new(),
                                event: "cue_in".to_string(),
                                pts: pts_secs,
                                break_duration: None,
                                hex: scte35::build_splice_insert_hex(
                                    scte35::Scte35Command::SpliceEnd,
                                    cue_in_pts,
                                    None,
                                ),
                                base64: {
                                    let bytes = scte35::build_splice_insert_bytes(
                                        scte35::Scte35Command::SpliceEnd,
                                        cue_in_pts,
                                        None,
                                    );
                                    scte35::base64_encode(&bytes)
                                },
                                output: "hls".to_string(),
                                playlist: Some(m3u8_path.clone()),
                            };
                            if let Err(e) =
                                scte35::log_scte35_event(&config.channel.logs, id, event).await
                            {
                                warn!(target: Target::file_mail(), channel = id;
                                    "SCTE-35: Failed to write log event: {e}");
                            }

                            // Remember to re-inject after the first program clip
                            // overwrites the m3u8.
                            cue_in_reinject = Some((m3u8_path.clone(), tag));
                            // Sleep to give HLS players time to poll the updated
                            // m3u8 before the next clip's ffmpeg rewrites it.
                            sleep(Duration::from_secs(2)).await;
                        }
                        Err(e) => {
                            error!(target: Target::file_mail(), channel = id;
                                "SCTE-35 HLS: Failed to inject CUE-IN into {m3u8_path}: {e}");
                        }
                    }
                } else {
                    warn!(target: Target::file_mail(), channel = id;
                        "SCTE-35 HLS: No m3u8 path found in output_cmd for CUE-IN");
                }
            } else {
                warn!(target: Target::file_mail(), channel = id;
                    "SCTE-35 HLS: output_cmd is None, cannot inject CUE-IN");
            }
        }

        // Advance continuous channel PTS by this clip's duration
        channel_pts += (clip_duration * 90_000.0) as u64;

        while ingest_is_alive.load(Ordering::SeqCst) {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    Ok(())
}

/// HLS Writer
///
/// Write with single ffmpeg instance directly to a HLS playlist.
pub async fn writer(manager: &ChannelManager, ff_log_format: &str) -> Result<(), ServiceError> {
    let config = manager.config.read().await.clone();
    let manager2 = manager.clone();
    let is_alive = manager.is_alive.clone();
    let output_cmd = config.output.output_cmd.unwrap_or_default();
    let hls_duration = output_cmd
        .windows(2)
        .find_map(|pair| {
            if pair[0] == "-hls_time" {
                pair[1].parse::<u64>().ok()
            } else {
                None
            }
        })
        .unwrap_or(10);

    let handle_ingest = if config.ingest.enable {
        // spawn a thread for ffmpeg ingest server
        Some(tokio::spawn(ingest_writer(manager2)))
    } else {
        None
    };

    let watchdog_hls = tokio::spawn(hls_watchdog(
        config.general.channel_id,
        config.channel.public.clone(),
        Duration::from_secs(hls_duration),
        is_alive,
    ));

    tokio::select! {
        result = async {
            if let Some(f) = handle_ingest {
                f.await?
            } else {
                Ok(())
            }
        }, if handle_ingest.is_some() => {
            result?;
        }

        result = watchdog_hls => {
            result??;
        }

        result = write(manager, ff_log_format) => {
            result?;
        }
    }

    Ok(())
}
