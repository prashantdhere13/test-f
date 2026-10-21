use log::*;

/// SCTE-35 splice_insert command types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scte35Command {
    SpliceStart,
    SpliceEnd,
}

/// An SCTE-35 event record written to the JSON log file.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Scte35Event {
    /// ISO-8601 timestamp of the event
    pub timestamp: String,
    /// "cue_out" or "cue_in"
    pub event: String,
    /// PTS in seconds (continuous channel PTS)
    pub pts: f64,
    /// Break duration in seconds (only for cue_out)
    pub break_duration: Option<f64>,
    /// Hex-encoded splice_insert bytes
    pub hex: String,
    /// Base64-encoded splice_insert bytes
    pub base64: String,
    /// Output target: "hls" or "stream"
    pub output: String,
    /// m3u8 path (HLS only)
    pub playlist: Option<String>,
}

/// Build a valid SCTE-35 splice_info_section (ANSI/SCTE 35) as raw bytes.
///
/// Wire format (all multi-byte fields are big-endian):
///
/// ```text
/// splice_info_section() {
///     table_id                        8   = 0xFC
///     section_syntax_indicator        1   = 0
///     private_indicator               1   = 0
///     sap_type                        2   = 0
///     section_length                  12  (bytes after this field, incl. CRC32)
///     protocol_version                8   = 0
///     encrypted_packet                1   = 0
///     reserved                        7   = 0x7F
///     pts_adjustment                  33  = <pts value>
///     cw_index                        8   = 0
///     tier                            12  = 0x7E0
///     splice_command_length           13  (bytes of command data that follow)
///     splice_command_type             8   = 0x00 (splice_insert)
///     splice_insert() {
///         out_of_network_indicator    1
///         auto_return                 1
///         duration_flag               1
///         pts_time_specified          1
///         reserved                    4   = 0
///         if (pts_time_specified) {
///             time_specified_flag     1   = 1
///             reserved                6   = 0x7F
///             pts_time                33
///         }
///         if (duration_flag) {
///             auto_return             1
///             reserved                6   = 0x3F
///             out_of_network_indicator 1
///             break_duration          33
///         }
///     }
///     descriptor_loop_length          16  = 0
///     CRC32                           32
/// }
/// ```
pub fn build_splice_insert_bytes(
    command: Scte35Command,
    pts: u64,
    break_duration_90k: Option<u64>,
) -> Vec<u8> {
    let out_of_network = match command {
        Scte35Command::SpliceStart => 1u8,
        Scte35Command::SpliceEnd => 0u8,
    };

    let has_break = break_duration_90k.is_some();
    let auto_return = has_break as u8;

    // --- Build the splice_insert command data (no length prefix) ---

    let mut cmd: Vec<u8> = Vec::new();

    // Flags byte: OON(1) | auto_return(1) | duration_flag(1) | pts_time_specified(1) | reserved(4)
    let flags: u8 =
        (out_of_network << 7) | (auto_return << 6) | ((has_break as u8) << 5) | (1 << 4); // pts_time_specified always present

    cmd.push(flags);

    // splice_time: time_specified_flag(1) | reserved(6) | pts_time(33)
    let pts_33 = pts & 0x1FFFFFFFF_u64;
    let pts_word: u64 = (1u64 << 39) | pts_33;
    cmd.push(((pts_word >> 32) & 0xFF) as u8);
    cmd.push(((pts_word >> 24) & 0xFF) as u8);
    cmd.push(((pts_word >> 16) & 0xFF) as u8);
    cmd.push(((pts_word >> 8) & 0xFF) as u8);
    cmd.push((pts_word & 0xFF) as u8);

    // break_duration (conditional, 40 bits): auto_return(1) | reserved(6) | OON(1) | duration(33)
    if let Some(dur) = break_duration_90k {
        let dur_33 = dur & 0x1FFFFFFFF_u64;
        let bw: u64 = ((auto_return as u64) << 39)
            | (0x3F_u64 << 33) // reserved
            | ((out_of_network as u64) << 32)
            | dur_33;
        cmd.push(((bw >> 32) & 0xFF) as u8);
        cmd.push(((bw >> 24) & 0xFF) as u8);
        cmd.push(((bw >> 16) & 0xFF) as u8);
        cmd.push(((bw >> 8) & 0xFF) as u8);
        cmd.push((bw & 0xFF) as u8);
    }

    // --- Build splice_info_section body (before CRC32) ---

    let mut body: Vec<u8> = Vec::new();

    body.push(0x00); // protocol_version = 0

    // encrypted_packet(1) + reserved(7) + pts_adjustment(33) = 5 bytes
    // encrypted_packet = 0, reserved = 0x7F, pts_adjustment = pts
    let pts_40: u64 = (0u64 << 39) | (0x7Fu64 << 33) | (pts & 0x1FFFFFFFF_u64);
    body.push(((pts_40 >> 32) & 0xFF) as u8);
    body.push(((pts_40 >> 24) & 0xFF) as u8);
    body.push(((pts_40 >> 16) & 0xFF) as u8);
    body.push(((pts_40 >> 8) & 0xFF) as u8);
    body.push((pts_40 & 0xFF) as u8);

    body.push(0x00); // cw_index = 0

    // tier(12) = 0x7E0, splice_command_length(13) = cmd.len()
    let tier_and_len: u16 = (0x7E0u16 << 13) | ((cmd.len() as u16) & 0x1FFF);
    body.push(((tier_and_len >> 8) & 0xFF) as u8);
    body.push((tier_and_len & 0xFF) as u8);

    body.push(0x00); // splice_command_type = 0x00 (splice_insert)
    body.extend_from_slice(&cmd);

    body.push(0x00); // descriptor_loop_length high
    body.push(0x00); // descriptor_loop_length low

    // --- Compute section_length and build the full section ---

    let section_length = body.len() + 4; // +4 for CRC32

    let mut section: Vec<u8> = Vec::with_capacity(3 + body.len() + 4);
    section.push(0xFC); // table_id

    // section_syntax_indicator(1)=0, private_indicator(1)=0, sap_type(2)=0, section_length(12)
    let hdr: u16 = (section_length as u16) & 0x0FFF;
    section.push(((hdr >> 8) & 0xFF) as u8);
    section.push((hdr & 0xFF) as u8);

    section.extend_from_slice(&body);

    let crc = crc32(&section);
    section.extend_from_slice(&crc.to_be_bytes());
    section
}

/// Build SCTE-35 splice_insert as hex string (for MPEG-TS metadata).
pub fn build_splice_insert_hex(
    command: Scte35Command,
    pts: u64,
    break_duration_90k: Option<u64>,
) -> String {
    build_splice_insert_bytes(command, pts, break_duration_90k)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= u32::from(byte) << 24;
        for _ in 0..8 {
            if crc & 0x80000000 != 0 {
                crc = (crc << 1) ^ 0x04C11DB7;
            } else {
                crc <<= 1;
            }
        }
    }
    crc ^ 0xFFFFFFFF
}

/// Encode bytes to base64 (no external dependency).
pub fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Build an #EXT-X-CUE-OUT tag for HLS.
///
/// Format: `#EXT-X-CUE-OUT:Duration=<secs>`
pub fn build_cue_out_tag(duration_secs: f64, _pts_90k: u64, _break_duration_90k: u64) -> String {
    format!("#EXT-X-CUE-OUT:Duration={duration_secs:.3}")
}

/// Build an #EXT-X-CUE-IN tag for HLS.
///
/// CUE-IN is a **separate** SCTE-35 event from CUE-OUT — it signals the end of
/// the ad break and return to program content. It must be injected as its own
/// tag after all ad segments have been written.
///
/// Format: `#EXT-X-CUE-IN`
pub fn build_cue_in_tag(_pts_90k: u64, _duration_secs: f64) -> String {
    "#EXT-X-CUE-IN".to_string()
}

/// Write an SCTE-35 event to the channel's scte log folder as a JSON file.
///
/// File: `<logs>/<channel_id>/scte/<YYYY-MM-DD>.jsonl`
/// Each event is appended as one JSON line (JSONL format).
pub async fn log_scte35_event(
    logs_path: &std::path::Path,
    channel_id: i32,
    mut event: Scte35Event,
) -> Result<(), std::io::Error> {
    use tokio::io::AsyncWriteExt;

    // Auto-fill timestamp if empty
    if event.timestamp.is_empty() {
        event.timestamp = chrono::Utc::now().to_rfc3339();
    }

    let scte_dir = logs_path.join(channel_id.to_string()).join("scte");
    tokio::fs::create_dir_all(&scte_dir).await?;

    let date = if event.timestamp.len() >= 10 {
        &event.timestamp[..10]
    } else {
        "unknown"
    };
    let file_path = scte_dir.join(format!("{date}.jsonl"));

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .await?;

    let mut json = serde_json::to_string(&event)?;
    json.push('\n');
    file.write_all(json.as_bytes()).await?;
    file.flush().await?;

    Ok(())
}
///
/// Reads the entire m3u8, checks for duplicate tags, inserts the line
/// either before `before_pattern` (if provided) or before `#EXT-X-ENDLIST`,
/// then writes atomically via temp file + rename.
pub async fn inject_tag_atomic(
    playlist_path: &std::path::Path,
    line: &str,
) -> Result<(), std::io::Error> {
    inject_tag_at(playlist_path, line, None).await
}

/// Inject a tag line into an HLS m3u8, optionally before a specific pattern.
///
/// If `before_pattern` is Some, the tag is inserted before the last line
/// matching that prefix (e.g. `"#EXTINF"` to place before a segment).
/// If None, the tag is inserted before `#EXT-X-ENDLIST` or at the end.
///
/// Uses a retry loop: if the m3u8 is modified between our read and write
/// (by FFmpeg's HLS muxer), we re-read and re-inject to avoid losing
/// segment entries.
pub async fn inject_tag_at(
    playlist_path: &std::path::Path,
    line: &str,
    before_pattern: Option<&str>,
) -> Result<(), std::io::Error> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    const MAX_RETRIES: u32 = 5;

    for attempt in 0..MAX_RETRIES {
        let content = match tokio::fs::read_to_string(playlist_path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        };

        // Record mtime before we read, so we can detect concurrent writes
        let mtime_before = tokio::fs::metadata(playlist_path)
            .await
            .and_then(|m| m.modified())
            .ok();

        // Avoid duplicate injection — check by tag prefix (e.g. "#EXT-X-CUE-OUT").
        let tag_prefix = line.split(':').next().unwrap_or("");
        if content.contains(tag_prefix) {
            return Ok(());
        }

        let mut lines: Vec<String> = content.lines().map(String::from).collect();

        let insert_pos = if let Some(pattern) = before_pattern {
            // Insert before the first line starting with the pattern.
            lines
                .iter()
                .position(|l| l.trim_start().starts_with(pattern))
                .unwrap_or(lines.len())
        } else {
            // Insert before #EXT-X-ENDLIST if present, otherwise at the end.
            lines
                .iter()
                .rposition(|l| l.trim() == "#EXT-X-ENDLIST")
                .unwrap_or(lines.len())
        };

        lines.insert(insert_pos, line.to_string());

        let new_content = lines.join("\n") + "\n";

        let parent = playlist_path.parent().unwrap_or(std::path::Path::new("."));
        let tmp_path = parent.join(format!(
            ".{}.tmp",
            playlist_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("playlist")
        ));

        tokio::fs::write(&tmp_path, new_content).await?;
        tokio::fs::rename(&tmp_path, playlist_path).await?;

        // Check if FFmpeg modified the m3u8 between our read and write.
        // If so, our write may have overwritten FFmpeg's segment entries.
        let mtime_after = tokio::fs::metadata(playlist_path)
            .await
            .and_then(|m| m.modified())
            .ok();

        let unchanged = match (mtime_before, mtime_after) {
            (Some(before), Some(after)) => before == after,
            _ => true,
        };

        if unchanged {
            return Ok(());
        }

        // mtime changed — FFmpeg wrote concurrently. Re-read and retry.
        warn!(
            "SCTE-35 m3u8 modified during injection (attempt {}/{MAX_RETRIES}), retrying",
            attempt + 1
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // If we exhausted retries, the last write still succeeded — the tag is in the file.
    // FFmpeg may overwrite it on its next m3u8 write, but that's handled by the
    // re-inject logic in hls.rs.
    Ok(())
}

/// Delete SCTE log files older than `retain_days` days.
///
/// Scans `<logs>/<channel_id>/scte/` for `YYYY-MM-DD.jsonl` files
/// and removes any whose date prefix is older than the retention window.
pub async fn cleanup_scte_logs(
    logs_path: &std::path::Path,
    channel_id: i32,
    retain_days: u32,
) -> Result<usize, std::io::Error> {
    let scte_dir = logs_path.join(channel_id.to_string()).join("scte");
    if !scte_dir.is_dir() {
        return Ok(0);
    }

    let cutoff = chrono::Utc::now() - chrono::Duration::days(retain_days as i64);
    let cutoff_date = cutoff.format("%Y-%m-%d").to_string();
    let mut removed = 0;

    let mut entries = tokio::fs::read_dir(&scte_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();

        // Expect format: YYYY-MM-DD.jsonl
        if name.len() == 10 && name.ends_with(".jsonl") {
            let date_part = &name[..10];
            if date_part < cutoff_date.as_str() {
                if let Err(e) = tokio::fs::remove_file(entry.path()).await {
                    warn!("Failed to remove old SCTE log {name}: {e}");
                } else {
                    removed += 1;
                }
            }
        }
    }

    if removed > 0 {
        info!("Cleaned up {removed} old SCTE log file(s) for channel {channel_id}");
    }

    Ok(removed)
}

/// Find the m3u8 playlist path from the output command parameters.
pub fn find_m3u8_path(output_cmd: &[String]) -> Option<String> {
    output_cmd
        .iter()
        .find(|s| s.ends_with(".m3u8") && !s.contains("master"))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splice_start_no_break() {
        let bytes = build_splice_insert_bytes(Scte35Command::SpliceStart, 0, None);
        assert_eq!(bytes[0], 0xFC, "table_id must be 0xFC");
        // section_syntax_indicator=0, private_indicator=0, sap_type=0
        assert_eq!(bytes[1] & 0xF0, 0x00, "header nibble must be 0x0_");
        assert_eq!(bytes[3], 0x00, "protocol_version = 0");
        // encrypted_packet(1)=0 + reserved(7)=0x7F -> byte4 = 0x7F
        assert_eq!(bytes[4], 0x7F, "encrypted_packet=0 + reserved=0x7F");
        assert_eq!(bytes[9], 0x00, "cw_index = 0");
    }

    #[test]
    fn test_splice_start_with_break() {
        let bytes = build_splice_insert_bytes(Scte35Command::SpliceStart, 0, Some(900_000));
        let bytes_no = build_splice_insert_bytes(Scte35Command::SpliceStart, 0, None);
        // With break_duration: +5 bytes in cmd data -> section is 5 bytes larger
        assert_eq!(bytes.len(), bytes_no.len() + 5);
    }

    #[test]
    fn test_splice_end_no_break() {
        let bytes = build_splice_insert_bytes(Scte35Command::SpliceEnd, 90_000, None);
        assert_eq!(bytes[0], 0xFC);
        // Flags: OON=0, auto_return=0, duration_flag=0, pts_time_specified=1
        // After section header (3) + body before cmd (1+5+1+2+1=10) = offset 13 is flags byte
        let flags = bytes[13];
        assert_eq!(
            flags & 0xF0,
            0x10,
            "SpliceEnd flags: OON=0, auto=0, dur=0, pts=1 -> 0x10"
        );
    }

    #[test]
    fn test_crc32_is_valid() {
        let bytes = build_splice_insert_bytes(Scte35Command::SpliceStart, 0, None);
        let data = &bytes[..bytes.len() - 4];
        let crc_bytes = &bytes[bytes.len() - 4..];
        let crc_read = u32::from_be_bytes([crc_bytes[0], crc_bytes[1], crc_bytes[2], crc_bytes[3]]);
        assert_eq!(crc32(data), crc_read, "CRC32 must match");
    }

    #[test]
    fn test_section_length_consistency() {
        let bytes = build_splice_insert_bytes(Scte35Command::SpliceStart, 0, Some(900_000));
        // section_length is in bytes[1..3] (12 bits), total after first 3 bytes
        let sl = ((bytes[1] as usize) << 8 | bytes[2] as usize) & 0x0FFF;
        assert_eq!(sl, bytes.len() - 3, "section_length must equal total - 3");
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
    }

    #[test]
    fn test_cue_out_tag_format() {
        let tag = build_cue_out_tag(300.0, 0, 300 * 90_000);
        assert_eq!(tag, "#EXT-X-CUE-OUT:Duration=300.000");
    }

    #[test]
    fn test_cue_in_tag_format() {
        let tag = build_cue_in_tag(90_000, 30.0);
        assert_eq!(tag, "#EXT-X-CUE-IN");
    }

    #[test]
    fn test_find_m3u8_path() {
        let cmd = vec![
            "-f".to_string(),
            "hls".to_string(),
            "live/stream.m3u8".to_string(),
        ];
        assert_eq!(find_m3u8_path(&cmd), Some("live/stream.m3u8".to_string()));
    }

    #[test]
    fn test_find_m3u8_path_excludes_master() {
        let cmd = vec![
            "-f".to_string(),
            "hls".to_string(),
            "live/master.m3u8".to_string(),
        ];
        assert_eq!(find_m3u8_path(&cmd), None);
    }
}
