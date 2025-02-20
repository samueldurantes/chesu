CREATE TABLE games
(
  id               uuid primary  key        default uuid_generate_v1mc(),
  white_player     uuid not null references "users" (id),
  black_player     uuid not null references "users" (id),
  moves            text[]        not null default array[]::text[],
  state            text not null check (state in  ('waiting', 'running', 'draw', 'white_win', 'black_win')) default 'waiting',
  bet_value        int           not null,
  created_at       timestamptz   not null default now(),
  updated_at       timestamptz
);

SELECT trigger_updated_at('games');
