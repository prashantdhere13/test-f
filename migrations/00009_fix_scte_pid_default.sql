-- Fix SCTE-35 default: reset scte_pid to 0 for all existing outputs.
-- The previous migration (00008) defaulted scte_pid to 528, which caused
-- SCTE-35 encoder flags to be injected unconditionally, breaking the
-- ffmpeg output commands and causing filler playback.

UPDATE outputs SET scte_pid = 0 WHERE scte_pid != 0;
