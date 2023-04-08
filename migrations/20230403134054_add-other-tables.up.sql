create table channels (
  id serial primary key,
  title text not null,
  url text not null,
  thumbnail text not null
);

create table videos (
  id serial primary key,
  channel_id int not null,
  title text not null,
  url text not null,
  upload_datetime timestamp not null,
  views bigint not null,
  length int not null,
  foreign key (channel_id) references channels(id)
);

create table captions (
  id serial primary key,
  video_id int not null,
  raw_text text not null,
  foreign key (video_id) references videos(id)
);

create table submissions (
  id serial primary key,
  user_id int not null,
  video_id int not null,
  submitted_datetime timestamp not null,
  foreign key (user_id) references users(id),
  foreign key (video_id) references videos(id)
);