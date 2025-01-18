# Chesu
Play chess, bet money, and win in a secure platform.

## Getting Started
### Prerequisites
- Node.js >= 20
- pnpm >= 9
- rust >= 1.78

### Installation
1. Clone the repo:
```sh
git clone https://github.com/username/repo.git
```

2. Set .env: 
```sh
# In ./apps/server/ dir
cp .env.example .env

# Fill environment variables
vim .env
```

3. Install client and server dependencies:
```sh
# In ./apps/server/ dir
cargo build

# In ./apps/client/ dir
pnpm i
```

4. Run application:
```sh
# In ./ dir
pnpm run dev
```
