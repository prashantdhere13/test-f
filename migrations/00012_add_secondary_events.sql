-- Add secondary_events (graphic overlay) settings to configurations.
ALTER TABLE configurations ADD COLUMN se_enable INTEGER NOT NULL DEFAULT 1;
ALTER TABLE configurations ADD COLUMN se_graphic_enable INTEGER NOT NULL DEFAULT 0;
ALTER TABLE configurations ADD COLUMN se_graphic TEXT NOT NULL DEFAULT '';
ALTER TABLE configurations ADD COLUMN se_graphic_position TEXT NOT NULL DEFAULT 'bottom right';
ALTER TABLE configurations ADD COLUMN se_graphic_scale INTEGER NOT NULL DEFAULT 20;
ALTER TABLE configurations ADD COLUMN se_nnl_position TEXT NOT NULL DEFAULT 'top right';
ALTER TABLE configurations ADD COLUMN se_nnl_size INTEGER NOT NULL DEFAULT 100;
