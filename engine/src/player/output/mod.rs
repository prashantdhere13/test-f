use std::{process::Stdio, sync::atomic::Ordering};

use log::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, Command},
};

mod desktop;
mod hls;
mod null;
pub mod scte35;
mod stream;

use crate::player::{
    controller::{ChannelManager, ProcessUnit::*},
    input::{ingest_server, source_generator},
    utils::{sec_to_time, stderr_reader},
};
use crate::utils::{
    config::OutputMode::*,
    errors::ServiceError,
    logging::{Target, fmt_cmd},
    task_runner,
};
use crate::vec_strings;

/// Write a short burst of black video + silence to the encoder stdin.
/// This bridges the gap between decoder processes at ad/SCTE transitions,
/// preventing the encoder from entering error-recovery or freeze state
/// when it momentarily receives no data.
async fn write_transition_filler(
    enc_writer: &mut ChildStdin,
    width: u32,
    height: u32,
    fps: f64,
    channels: u32,
) -> Result<(), ServiceError> {
    let duration = "0.5";
    let filler_cmd = vec![
        "-hide_banner".to_string(),
        "-nostats".to_string(),
        "-v".to_string(),
        "quiet".to_string(),
        "-f".to_string(),
        "lavfi".to_string(),
        "-i".to_string(),
        format!("color=c=black:s={width}x{height}:d={duration}:r={fps}"),
        "-f".to_string(),
        "lavfi".to_string(),
        "-i".to_string(),
        format!("anullsrc=channel_layout=stereo:sample_rate=48000"),
        "-t".to_string(),
        duration.to_string(),
        "-c:v".to_string(),
        "mpeg2video".to_string(),
        "-g".to_string(),
        "1".to_string(),
        "-b:v".to_string(),
        "5000k".to_string(),
        "-c:a".to_string(),
        "s302m".to_string(),
        "-strict".to_string(),
        "-2".to_string(),
        "-sample_fmt".to_string(),
        "s16".to_string(),
        "-ar".to_string(),
        "48000".to_string(),
        "-ac".to_string(),
        channels.to_string(),
        "-f".to_string(),
        "mpegts".to_string(),
        "-".to_string(),
    ];

    let mut filler_proc = Command::new("ffmpeg")
        .args(&filler_cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut filler_stdout = filler_proc.stdout.take().unwrap();
    let mut filler_buf = vec![0u8; 64 * 1024];

    loop {
        let num = filler_stdout.read(&mut filler_buf[..]).await?;
        if num == 0 {
            break;
        }
        enc_writer.write_all(&filler_buf[..num]).await?;
    }

    filler_proc.wait().await.ok();
    Ok(())
}

async fn play(
    manager: ChannelManager,
    mut enc_writer: ChildStdin,
    ff_log_format: &str,
) -> Result<(), ServiceError> {
    let config = manager.config.read().await.clone();
    let id = config.general.channel_id;
    let playlist_init = manager.list_init.clone();
    let is_alive = manager.is_alive.clone();
    let ingest_is_alive = manager.ingest_is_alive.clone();
    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB pipe buffer for high throughput
    let mut live_on = false;
    // Continuous channel PTS in 90kHz ticks.
    // Each clip contributes (out - seek) * 90000 ticks.
    // SCTE-35 splice_insert markers use this instead of clip-local seek PTS,
    // so downstream decoders see a monotonically increasing timeline.
    let mut channel_pts: u64 = 0;

    // get source iterator
    let mut node_sources = source_generator(manager.clone()).await;

    while let Some(node) = node_sources.next().await {
        *manager.current_media.lock().await = Some(node.clone());
        let ignore_dec = config.logging.ignore_lines.clone();

        if !is_alive.load(Ordering::SeqCst) {
            debug!(target: Target::file_mail(), channel = id; "Playout is stopped, break out from source loop");
            break;
        }

        trace!("Decoder CMD: {:?}", node.cmd);

        if node.skip {
            // skip is different from node.cmd = None.
            // This source is valid, but too short to play,
            // so better skip it and go to the next one.
            continue;
        }

        let mut cmd = match node.cmd {
            Some(cmd) => cmd,
            None => break,
        };

        let c_index = if cfg!(debug_assertions) {
            format!(
                " ({}/{})",
                node.index.unwrap() + 1,
                manager.current_list.lock().await.len()
            )
        } else {
            String::new()
        };

        info!(target: Target::file_mail(), channel = id;
            "Play for <span class=\"log-number\">{}</span>{c_index}: <span class=\"log-addr\">{}  {}</span>",
            sec_to_time(node.out - node.seek),
            node.source,
            node.audio
        );

        if config.task.enable {
            if config.task.path.is_file() {
                let channel_mgr_3 = manager.clone();

                tokio::spawn(task_runner::run(channel_mgr_3));
            } else {
                error!(target: Target::file_mail(), channel = id;
                    "<span class=\"log-cmd\">{:?}</span> executable not exists!",
                    config.task.path
                );
            }
        }

        // --- SCTE-35: force I-frame at ad break boundaries ---
        let is_ad = node.category == "advertisement";
        let is_ad_start = is_ad && !node.last_ad;
        let is_ad_end = is_ad && !node.next_ad;
        let is_scte_boundary = config.output.scte_pid > 0 && (is_ad_start || is_ad_end);

        if is_scte_boundary {
            // Insert a forced keyframe at the boundary so the encoder emits an I-frame
            // immediately, preventing downstream decoder freeze.
            info!(target: Target::file_mail(), channel = id;
                "SCTE-35: {} I-frame boundary (scte_pid={})",
                if is_ad_start { "Ad start" } else { "Ad end" },
                config.output.scte_pid
            );
        }

        let mut dec_cmd = vec_strings!["-hide_banner", "-nostats", "-v", &ff_log_format];

        if let Some(decoder_input_cmd) = &config.advanced.decoder.input_cmd {
            dec_cmd.append(&mut decoder_input_cmd.clone());
        }

        dec_cmd.append(&mut cmd);

        // Force I-frame at the start of each clip at ad boundaries.
        // Since the intermediate pipe already uses -g 1 (every frame is I-frame),
        // the output encoder will see clean keyframes. We add force_key_frames
        // to the output encoder to ensure keyframes at exact transition points.
        // NOTE: We do NOT add -vf here as it would conflict with -filter_complex.

        if let Some(mut filter) = node.filter {
            dec_cmd.append(&mut filter.cmd());
            dec_cmd.append(&mut filter.map());
        }

        if config.processing.vtt_enable && dec_cmd.iter().any(|s| s.ends_with(".vtt")) {
            let i = dec_cmd
                .iter()
                .filter(|&n| n == "-i")
                .count()
                .saturating_sub(1);

            dec_cmd.append(&mut vec_strings!("-map", format!("{i}:s?")));
        }

        if let Some(cmd) = &config.processing.cmd {
            dec_cmd.extend_from_slice(cmd);
        }

        debug!(target: Target::file_mail(), channel = id;
            "Decoder CMD: <span class=\"log-cmd\">ffmpeg {}</span>",
            fmt_cmd(&dec_cmd)
        );

        // create ffmpeg decoder instance, for reading the input files
        let mut dec_proc = Command::new("ffmpeg")
            .args(dec_cmd)
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut decoder_stdout = dec_proc.stdout.take().unwrap();
        let dec_err = BufReader::new(dec_proc.stderr.take().unwrap());

        *manager.clone().decoder.lock().await = Some(dec_proc);

        let error_decoder_task = tokio::spawn(stderr_reader(dec_err, ignore_dec, Decoder, id));

        loop {
            if ingest_is_alive.load(Ordering::SeqCst) {
                // read from ingest server instance
                if !live_on {
                    info!(target: Target::file_mail(), channel = id; "Switch from {} to live ingest", config.processing.mode);
                    playlist_init.store(true, Ordering::SeqCst);

                    manager.stop(Decoder).await;
                    live_on = true;
                }

                let num = {
                    let mut ingest_reader_guard = manager.ingest_reader.lock().await;
                    if let Some(ref mut ingest_stdout) = *ingest_reader_guard {
                        ingest_stdout.read(&mut buffer[..]).await?
                    } else {
                        0
                    }
                };

                if num == 0 {
                    enc_writer.flush().await?;
                    continue;
                }

                enc_writer.write_all(&buffer[..num]).await?;
            } else {
                // read from decoder instance
                if live_on {
                    info!(target: Target::file_mail(), channel = id; "Switch from live ingest to {}", config.processing.mode);

                    live_on = false;
                    break;
                }

                let num = decoder_stdout.read(&mut buffer[..]).await?;

                if num == 0 {
                    enc_writer.flush().await?;
                    break;
                }

                enc_writer.write_all(&buffer[..num]).await?;
            }
        }

        // --- SCTE-35: inject splice_insert marker after clip ends ---
        if is_scte_boundary {
            let marker_type = if is_ad_start {
                scte35::Scte35Command::SpliceStart
            } else {
                scte35::Scte35Command::SpliceEnd
            };

            // SpliceStart (CUE-OUT): include break_duration with auto_return=1
            // so downstream decoders know exactly when to return to program.
            // SpliceEnd (CUE-IN): no break_duration needed.
            let break_dur = if is_ad_start {
                Some(((node.out - node.seek) * 90_000.0) as u64)
            } else {
                None
            };

            let hex_marker = scte35::build_splice_insert_hex(marker_type, channel_pts, break_dur);

            let pts_secs = channel_pts as f64 / 90_000.0;
            let break_secs = break_dur.map(|d| d as f64 / 90_000.0);

            info!(target: Target::file_mail(), channel = id;
                "SCTE-35: Injected splice_{} at PTS {:.2}s break_dur={}ms (hex: {}...)",
                if is_ad_start { "insert (CUE-OUT)" } else { "insert (CUE-IN)" },
                pts_secs,
                break_dur.unwrap_or(0) / 90,
                &hex_marker[..hex_marker.len().min(40)]
            );

            // Write JSON log to scte folder
            let event = scte35::Scte35Event {
                timestamp: String::new(),
                event: if is_ad_start {
                    "cue_out".to_string()
                } else {
                    "cue_in".to_string()
                },
                pts: pts_secs,
                break_duration: break_secs,
                hex: hex_marker.clone(),
                base64: {
                    let bytes =
                        scte35::build_splice_insert_bytes(marker_type, channel_pts, break_dur);
                    scte35::base64_encode(&bytes)
                },
                output: "stream".to_string(),
                playlist: None,
            };
            if let Err(e) = scte35::log_scte35_event(&config.channel.logs, id, event).await {
                warn!(target: Target::file_mail(), channel = id;
                    "SCTE-35: Failed to write log event: {e}");
            }
        }

        // Advance the continuous channel PTS by this clip's duration (in 90kHz ticks).
        channel_pts += ((node.out - node.seek) * 90_000.0) as u64;

        drop(decoder_stdout);

        manager.wait(Decoder).await;
        error_decoder_task.await??;

        // At ad break boundaries, inject a short burst of black/silence
        // to keep the encoder's MPEG-TS input buffer fed during the gap
        // between decoder processes. Without this, the encoder can enter
        // error-recovery or freeze when it momentarily receives no data.
        if is_ad_start || is_ad_end {
            if let Err(e) = write_transition_filler(
                &mut enc_writer,
                config.processing.width as u32,
                config.processing.height as u32,
                config.processing.fps,
                config.processing.audio_channels as u32,
            )
            .await
            {
                warn!(target: Target::file_mail(), channel = id;
                    "Transition filler failed: {e}");
            }
        }
    }

    Ok(())
}

/// Player
///
/// Here we create the input file loop, from playlist, or folder source.
/// Then we read the stdout from the reader ffmpeg instance
/// and write it to the stdin from the streamer ffmpeg instance.
/// If it is configured we also fire up a ffmpeg ingest server instance,
/// for getting live feeds.
/// When a live ingest arrive, it stops the current playing and switch to the live source.
/// When ingest stops, it switch back to playlist/folder mode.
pub async fn player(manager: ChannelManager) -> Result<(), ServiceError> {
    let config = manager.config.read().await.clone();
    let config_clone = config.clone();
    let ff_log_format = format!("level+{}", config.logging.ffmpeg_level.to_lowercase());
    let ignore_enc = config.logging.ignore_lines.clone();
    let channel_id = config.general.channel_id;

    // get ffmpeg output instance
    let mut enc_proc = match config.output.mode {
        Desktop => desktop::output(&config, &ff_log_format).await?,
        Null => null::output(&config, &ff_log_format).await?,
        Stream => stream::output(&config, &ff_log_format).await?,
        HLS => {
            hls::writer(&manager, &ff_log_format).await?;
            manager.stop_all(false).await;

            return Ok(());
        }
    };

    let enc_err = BufReader::new(enc_proc.stderr.take().unwrap());
    let enc_writer = enc_proc.stdin.take().unwrap();

    *manager.encoder.lock().await = Some(enc_proc);
    let mgr_clone2 = manager.clone();

    // spawn a task to log ffmpeg output error messages
    let handle_enc_stderr = tokio::spawn(stderr_reader(enc_err, ignore_enc, Encoder, channel_id));

    // spawn a task for a ffmpeg ingest server
    let handle_ingest = if config.ingest.enable {
        Some(tokio::spawn(ingest_server(config_clone, mgr_clone2)))
    } else {
        None
    };

    tokio::select! {
        result = handle_enc_stderr => {
            result??;
        }

        result = async {
            if let Some(f) = handle_ingest {
                f.await?
            } else {
                Ok(())
            }
        }, if handle_ingest.is_some() => {
            result?;
        }

        result = play(manager, enc_writer, &ff_log_format) => {
            result?;
        }
    }

    trace!("Out of source loop");

    Ok(())
}
