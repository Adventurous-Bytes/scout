# Database

## Setup

1. Install a container runtime compatible with Docker APIs.
2. Download the supabase cli from [here](https://supabase.com/docs/guides/cli).
3. Associate Your Hosted Database
```bash
supabase init
supabase login
# select your database instance
supabase link
```

## Dump

```bash
supabase db dump -f database/dump.sql
```

## Get Types

```bash
supabase gen types typescript --project-id "nfgpianoyribtvkqbjeq" --schema public > core/types/supabase.ts
```
