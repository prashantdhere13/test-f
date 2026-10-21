-- Add 'processing_scan_type' to configurations for interlace/progressive output control.
-- 'progressive' (default): deinterlace interlaced sources, keep progressive sources as-is.
-- 'interlace': force interlaced output (for broadcast/capture card compatibility).
ALTER TABLE configurations ADD COLUMN processing_scan_type TEXT NOT NULL DEFAULT 'progressive';
