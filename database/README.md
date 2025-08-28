# Database

## Setup

Install a container runtime compatible with Docker APIs.
Download the supabase cli from [here](https://supabase.com/docs/guides/cli).

## Dump

```bash
supabase db dump -f database/dump.sql
```

## Get Types

```bash
supabase gen types typescript --project-id "nfgpianoyribtvkqbjeq" --schema public > core/types/supabase.ts
```
