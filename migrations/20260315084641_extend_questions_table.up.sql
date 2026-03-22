-- Add up migration script here
alter table questions
add column account_id serial;