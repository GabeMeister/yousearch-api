ALTER TABLE
  videos
ADD
  COLUMN thumbnail text not null default '';

ALTER TABLE
  videos
ALTER COLUMN
  upload_datetime type timestamp with time zone;