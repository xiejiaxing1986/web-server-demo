-- Add down migration script here
alter table questions
drop column account_id;