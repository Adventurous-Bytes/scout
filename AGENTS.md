This project contains systems that combine to persist data across dirtibuted systems. 

## Project Structure

# Docs
[Docs](docs) contains documentation for the project. We have three distinct sections of our documentation: scout-rs, scout-core, and scout-db. The content is stored in [content/docs](docs/content/docs). 


# Scout RS

[Scout RS](scout_rs) is a rust client for direct interactions with our database. The client exposes a [sync engine](scout_rs/src/sync.rs) that maintains a tree of data for synchronizing local state to our remote database. Scout RS use [native db](https://github.com/vincent-herlemont/native_db) for an embeeded database with explicit model versions. Rust schema definitions are found in the [models directory](scout_rs/src/models). 

# Scout Core

Scout core is a web library that exposes convenient methods for loading and updating state. This project assumes conumers are using NextJs and Redux. [useScoutRefresh](core/hooks/useScoutRefresh.ts) is responsible for loading state from indexdb, while fetching recent data from our database. [middleware](core/supabase/middleware.ts) exposes methods for validating user sessions via NextJs middleware/proxy.

Types are stored in [core/types](core/types) and are based on automatically generated types from the supabase cli.

# Editing Guidelines
- Ensure your edits are concise and follow existing patterns
- Don't create testing files
- Don't add example files
- Delete and simplify as much as possible
