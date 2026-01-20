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

## Updating Database Password

If you've reset your database password, you need to re-link the project with the new password. Use single quotes to prevent shell interpretation of special characters:

```bash
supabase link --project-ref nfgpianoyribtvkqbjeq --password 'your-new-password'
```

## Dump

```bash
supabase db dump -f database/dump.sql
```

## Get Types

```bash
supabase gen types typescript --project-id "nfgpianoyribtvkqbjeq" --schema public > core/types/supabase.ts
```
