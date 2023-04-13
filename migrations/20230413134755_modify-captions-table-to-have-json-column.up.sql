ALTER TABLE
  captions
ADD
  COLUMN caption_timestamps json not null default '{}' :: json;