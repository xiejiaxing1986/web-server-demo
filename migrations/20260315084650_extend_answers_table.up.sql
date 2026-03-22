-- Add up migration script here
alter table answers
add column account_id serial;