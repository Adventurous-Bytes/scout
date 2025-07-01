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
import {
  HerdModule,
  IDevice,
  IEvent,
  isEmailValidForLogin,
  useScoutDbListener,
  useScoutRefresh,
} from "@adventurelabs/scout-core";

// Use the HerdModule class
const herdModule = new HerdModule(herd, devices, events, Date.now());

// Use helper functions
const isValidEmail = isEmailValidForLogin("user@adventurelabs.earth");

// Use React hooks for real-time database listening
useScoutDbListener(); // Automatically listens for changes to plans, devices, and tags

// Use refresh hook
const { handleRefresh } = useScoutRefresh({ autoRefresh: true });
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
