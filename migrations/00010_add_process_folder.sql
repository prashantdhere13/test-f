-- Add 'process' path and 'auto_preprocess' flag to channels for preprocessed media files.
-- Preprocessed files are .ts with consistent resolution, fps, and logo baked in.
-- auto_preprocess: when true, preprocessing runs automatically on channel start.
ALTER TABLE channels ADD COLUMN process TEXT NOT NULL DEFAULT '';
ALTER TABLE channels ADD COLUMN auto_preprocess INTEGER NOT NULL DEFAULT 0;

-- Add 'storage_ad_pattern' to configurations for identifying ad files in folder mode.
-- Files matching this regex get category='advertisement', enabling SCTE-35 CUE-OUT/CUE-IN.
ALTER TABLE configurations ADD COLUMN storage_ad_pattern TEXT NOT NULL DEFAULT '';
