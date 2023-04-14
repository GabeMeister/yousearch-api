alter table
  captions rename column caption_timestamps TO caption_json;

create table caption_timestamps (
  id serial primary key,
  video_id int not null,
  caption_id int not null,
  caption_text text not null,
  start float not null,
  duration float not null,
  foreign key (video_id) references videos(id),
  foreign key (caption_id) references captions(id)
);

create index caption_text_index on caption_timestamps using gin(to_tsvector('english', caption_text));