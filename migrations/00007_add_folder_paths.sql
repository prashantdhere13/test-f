-- Add per-channel folder path configuration for storage sub-folders.
-- Each channel can now customize the relative paths for media, subtitles,
-- and graphics folders (relative to the channel storage root).

ALTER TABLE configurations ADD COLUMN storage_media TEXT NOT NULL DEFAULT 'media';
ALTER TABLE configurations ADD COLUMN storage_subtitles TEXT NOT NULL DEFAULT 'subtitles';
ALTER TABLE configurations ADD COLUMN storage_graphics TEXT NOT NULL DEFAULT 'graphics';
