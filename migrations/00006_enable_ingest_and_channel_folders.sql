-- Enable live stream ingest (works in playlist mode: live input interrupts
-- the playlist and switches back when the stream ends) for all channels.
UPDATE configurations SET ingest_enable = 1;

-- Give each channel its own SRT ingest port to avoid conflicts in multi channel setups.
-- Channel 1 keeps the default port 9936, every next channel gets the next port.
UPDATE configurations SET ingest_param =
    '-f mpegts -listen 1 -i srt://127.0.0.1:' || (9935 + channel_id) || '?mode=listener';

-- Enable the channel logo overlay for all channels.
UPDATE configurations SET processing_add_logo = 1;

-- Point logo, font and subtitle defaults to the per channel asset folders.
UPDATE configurations SET processing_logo = 'logo/logo.png'
    WHERE processing_logo = '00-assets/logo.png';
UPDATE configurations SET text_font = 'text/DejaVuSans.ttf'
    WHERE text_font = '00-assets/DejaVuSans.ttf';
UPDATE configurations SET processing_vtt_dummy = 'subtitles/dummy.vtt'
    WHERE processing_vtt_dummy = '00-assets/dummy.vtt';
