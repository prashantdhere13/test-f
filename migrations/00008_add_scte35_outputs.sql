-- Add SCTE-35 and MPEG-TS PID configuration to outputs table.
-- Each output preset can now specify PID mappings and SCTE-35 marker settings.
-- SCTE-35 is disabled by default (scte_pid=0) to avoid breaking existing outputs.

ALTER TABLE outputs ADD COLUMN video_pid INTEGER NOT NULL DEFAULT 256;
ALTER TABLE outputs ADD COLUMN audio_pid INTEGER NOT NULL DEFAULT 257;
ALTER TABLE outputs ADD COLUMN pmt_pid INTEGER NOT NULL DEFAULT 258;
ALTER TABLE outputs ADD COLUMN service_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE outputs ADD COLUMN scte_pid INTEGER NOT NULL DEFAULT 0;
ALTER TABLE outputs ADD COLUMN scte_marker_type TEXT NOT NULL DEFAULT 'splice_insert';

-- Ensure existing outputs have SCTE-35 disabled (scte_pid=0) so they are not
-- broken by the SCTE-35 encoder flag injection. Users must explicitly enable
-- SCTE-35 by setting scte_pid > 0 in the config UI.
UPDATE outputs SET scte_pid = 0 WHERE scte_pid = 528;
