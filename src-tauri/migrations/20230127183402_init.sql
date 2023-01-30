CREATE TABLE "queue" (
	"id"	TEXT NOT NULL,
	"name"	TEXT NOT NULL,
	"data_url"	TEXT NOT NULL,
	"info_url"	TEXT,
	"date"	DATETIME NOT NULL,
	"source"	TEXT NOT NULL,
	"was_set"	BOOLEAN NOT NULL DEFAULT 0,
	PRIMARY KEY("id")
);