alter table "queue" rename column "name" to "name_old";

alter table "queue" add column "name" text default null;

update "queue" set name = name_old;

alter table "queue" drop column "name_old";