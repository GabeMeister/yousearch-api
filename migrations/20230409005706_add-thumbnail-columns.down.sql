ALTER TABLE
  videos DROP COLUMN thumbnail;

ALTER TABLE
  videos
ALTER COLUMN
  upload_datetime type timestamp;