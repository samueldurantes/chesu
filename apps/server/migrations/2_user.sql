CREATE TABLE users
(
  id         uuid primary key default               uuid_generate_v1mc(),
  username   text collate "case_insensitive" unique not null,
  email      text collate "case_insensitive" unique not null,
  password   text                                   not null,
  created_at timestamptz                            not null default now(),
  updated_at timestamptz
);

SELECT trigger_updated_at('users');
