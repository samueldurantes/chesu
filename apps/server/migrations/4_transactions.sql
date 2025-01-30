CREATE TABLE transactions (
    id            uuid primary key default uuid_generate_v1mc(),
    user_id       uuid not null references users(id) on delete cascade,
    type          text not null check (type in  ('input', 'output')),
    invoice       text,
    amount        int not null check (amount >= 0),
    created_at    timestamptz not null default now(),
    last_balance  int not null check (last_balance >= 0) 
);
