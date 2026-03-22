-- Add up migration script here

create table if not exists accounts (
    id serial not null,
    email varchar(255) not null primary key,
    password varchar(255) not null
);