-- Your SQL goes here
create table config
(
    id int identity(1,1) primary key,
    refresh_minutes int DEFAULT 60,
    allow_nsfw int DEFAULT false
)