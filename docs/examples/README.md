# Playlist Import Templates

Import these files via the Player page **Import** button (toolbar, file-text icon)
or `PUT /api/file/{channel_id}/import/?file=<name>&date=<YYYY-MM-DD>`.

Supported formats:

| Format | Files | Notes |
|--------|-------|-------|
| Excel  | `.xlsx` | see `playlist_import_secondary_events.xlsx` |
| CSV    | `.csv` (comma/semicolon/tab) | see `playlist_import_template.csv` |
| JSON   | `.json` | ffplayout program export, bare array or single object |
| M3U    | `.m3u`, `.m3u8` | `#EXTINF:dur,title` + path; optional `#EXT-X-ASRUN:` graphics tags |
| As-run | `.asrun`, `.txt` | tab-separated automation log with HH:MM:SS:FF timecodes |

Every upload is archived per channel/date under
`demo_data/playlists/imports/<channel>/<date>/<time>_<filename>` - as-run logs
are broadcast evidence of what actually aired and are kept verbatim.

## Main columns (sheets)

| Column   | Required | Description |
|----------|----------|-------------|
| Start    | no       | `HH:MM:SS(.ms)`, `HH:MM:SS:FF` timecode or seconds. Omitted on all rows = cumulative sequencing from 00:00. |
| Title    | yes*     | Event title. *Used as source fallback when Source is empty. |
| Source   | yes*     | File name (searched in channel storage by exact name then stem match), storage-relative path or absolute path. |
| Duration | no       | Falls back to `Out - In`, then probed file duration, then 10 s. |
| In / Out | no       | Mark-in offset / mark-out position inside the file (seconds). |
| Type     | no       | Values containing `ad`, `promo` or `commerc` mark the row as advertisement. |

Without a recognizable header row the positional order
`Start, Title, Source, Duration, In, Out, Type` is assumed.

## Secondary graphic events (overlays)

Optional columns attach timed graphics overlays to a clip; they appear in the
web UI playlist table as a violet layer-count badge (hover for details) and are
rendered over the output by the playout engine:

- `se_template` - NOW / NEXT / LATER / Current-Movie / custom
- `se_in`, `se_out` - show/hide times relative to clip start (`00:00:00:00` format)
- `se_text` - overlay text
- more overlays per clip: duplicate the group as `se1_*`, `se2_*`, ...

A row with only se-columns filled (no Source) attaches its overlays to the
previous clip.

## Notes

- Rows whose Source cannot be resolved are imported anyway and are shown
  highlighted in red in the web UI (file missing).
- Excel files must be `.xlsx`; legacy `.xls` is not supported - re-save as
  `.xlsx` or `.csv`.
- Import replaces the whole playlist for the target date.
