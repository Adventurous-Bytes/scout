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

- `useScoutDbListener` - Real-time database listening for plans, devices, and tags with robust disconnect handling
- `useScoutRefresh` - Data refresh utilities
- `useConnectionStatus` - Connection status monitoring and manual reconnection controls

#### Robust Connection Features

The `useScoutDbListener` hook includes several features to handle network disconnections and connection issues:

- **Automatic Reconnection**: Automatically attempts to reconnect when the connection is lost
- **Exponential Backoff**: Uses exponential backoff with jitter to avoid overwhelming the server
- **Connection State Tracking**: Provides real-time connection status (connected, connecting, disconnected, error)
- **Error Handling**: Comprehensive error handling with detailed error messages
- **Manual Reconnection**: Allows manual reconnection attempts via the `reconnect()` function
- **Retry Limits**: Configurable maximum retry attempts to prevent infinite reconnection loops
- **Graceful Cleanup**: Proper cleanup of resources when the component unmounts

Example usage:

```tsx
import { useConnectionStatus } from "@adventurelabs/scout-core";

function ConnectionStatus() {
  const { isConnected, isConnecting, lastError, retryCount, reconnect } =
    useConnectionStatus();

  if (isConnecting) {
    return <div>Connecting to database...</div>;
  }

  if (lastError) {
    return (
      <div>
        <p>Connection error: {lastError}</p>
        <p>Retry attempts: {retryCount}</p>
        <button onClick={reconnect}>Reconnect</button>
      </div>
    );
  }

  return <div>Status: {isConnected ? "Connected" : "Disconnected"}</div>;
}
```

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
