# @adventurelabs/scout-core

Core utilities and helpers for Adventure Labs Scout applications.

## Installation

```bash
npm install @adventurelabs/scout-core
# or
yarn add @adventurelabs/scout-core
```

## Usage

```typescript
import "../../app/globals.css";
import StoreProvider from "../../components/Store/StoreProvider";
import { ScoutRefreshProvider } from "@adventurelabs/scout-core";

export default function ScoutLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    {/* Store provider for state management */}
    <StoreProvider>
      {/* Listen for updates and refresh data in background */}
      <ScoutRefreshProvider>
        <div className="">{children}</div>
      </ScoutRefreshProvider>

    </StoreProvider>
  );
}
```

## Available Modules

### Types

- Database types from Supabase
- Herd, Device, Event, User interfaces
- Request/Response types

### Helpers

- Authentication utilities
- Database operations
- Email validation
- GPS and location helpers
- Device and event management
- Tag and annotation utilities

### Hooks

- `useScoutDbListener` - Real-time database listening for plans, devices, and tags
- `useScoutRefresh` - Data refresh utilities

### Store

- Zustand-based state management for Scout applications

### Supabase

- Client, server, and middleware utilities for Supabase integration

### API Keys

- API key management utilities

## Development

```bash
# Install dependencies
yarn install

# Build the package
yarn build

# Watch for changes
yarn dev

# Clean build artifacts
yarn clean
```

## License

GPL-3.0
