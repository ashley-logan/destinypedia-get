-- Add migration script here
ALTER TABLE images
ADD COLUMN extension TEXT;