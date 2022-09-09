-- Your SQL goes here

create table sources
(
    id int identity(1,1) primary key NOT NULL,
    subreddit varchar(50) null
)