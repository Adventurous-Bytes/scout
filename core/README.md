# Scout Core

Core utilities and helpers for Adventure Labs Scout applications.

## Features

- **Herd Management**: Comprehensive herd and device management
- **Event Tracking**: Wildlife event monitoring and tagging
- **Real-time Updates**: Supabase-powered real-time data synchronization
- **State Management**: Redux-based state management with loading states

## Herd Modules Loading State

The core provides a global loading state for herd modules, which are essential for many consuming applications. This state tracks whether herd modules are currently loading, have loaded successfully, or failed to load.

### Loading State Enum

```typescript
import { EnumHerdModulesLoadingState } from "@adventurelabs/scout-core";

enum EnumHerdModulesLoadingState {
  NOT_LOADING = "NOT_LOADING",
  LOADING = "LOADING",
  SUCCESSFULLY_LOADED = "SUCCESSFULLY_LOADED",
  UNSUCCESSFULLY_LOADED = "UNSUCCESSFULLY_LOADED",
}

**Available Hooks:**
- `useHerdModulesLoadingState()` - Get current loading state
- `useIsHerdModulesLoading()` - Check if currently loading
- `useIsHerdModulesLoaded()` - Check if successfully loaded
- `useIsHerdModulesFailed()` - Check if loading failed
- `useHerdModulesLoadedAt()` - Get how long the last loading took (in milliseconds)
- `useHerdModulesLoadingDuration()` - Get loading duration in milliseconds
- `useHerdModulesLoadingTimeAgo()` - Get formatted time ago since last loaded (e.g., "2.5s ago")
```

### Usage in Components

```typescript
import {
  useHerdModulesLoadingState,
  useIsHerdModulesLoading,
  useIsHerdModulesLoaded,
  useIsHerdModulesFailed,
  useHerdModulesLoadedAt,
  useHerdModulesLoadingTimeAgo,
  useHerdModulesLoadingDuration,
} from "@adventurelabs/scout-core";

function MyComponent() {
  const loadingState = useHerdModulesLoadingState();
  const isLoading = useIsHerdModulesLoading();
  const isLoaded = useIsHerdModulesLoaded();
  const isFailed = useIsHerdModulesFailed();

  if (isLoading) {
    return <div>Loading herd modules...</div>;
  }

  if (isFailed) {
    return <div>Failed to load herd modules</div>;
  }

  if (isLoaded) {
    return <div>Herd modules loaded successfully!</div>;
  }

  return <div>Not loading</div>;
}

// Example with loading duration information
function HerdModulesStatus() {
  const loadingState = useHerdModulesLoadingState();
  const loadingTimeMs = useHerdModulesLoadedAt();
  const timeAgo = useHerdModulesLoadingTimeAgo();
  const loadingDuration = useHerdModulesLoadingDuration();

  return (
    <div>
      <div>Status: {loadingState}</div>
      {loadingTimeMs && (
        <>
          <div>Last loading took: {loadingTimeMs}ms</div>
          <div>Loaded: {timeAgo}</div>
          <div>Loading duration: {loadingDuration}ms</div>
        </>
      )}
    </div>
  );
}
```

### Manual Refresh

```typescript
import { useScoutRefresh } from "@adventurelabs/scout-core";

function RefreshButton() {
  const { handleRefresh } = useScoutRefresh({ autoRefresh: false });

  return <button onClick={handleRefresh}>Refresh Data</button>;
}
```

## Installation

```bash
npm install @adventurelabs/scout-core
# or
yarn add @adventurelabs/scout-core
```

## Setup

Wrap your app with the ScoutRefreshProvider:

```typescript
import { ScoutRefreshProvider } from "@adventurelabs/scout-core";

function App() {
  return (
    <ScoutRefreshProvider>{/* Your app components */}</ScoutRefreshProvider>
  );
}
```

## Recent Updates

- **v1.0.58**: Added global herd modules loading state tracking with timestamps
- Fixed repeat Supabase client creation logs
- Enhanced loading state management for better UX
- Added loading duration and time-ago tracking
- Added comprehensive edge case handling and race condition prevention

## Usage

````typescript
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

## Available Modules

### Types

- Database types from Supabase
- Herd, Device, Event, User interfaces
- Request/Response types
- Herd module loading state enums (`EnumHerdModulesLoadingState`)

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
````

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

**New Hooks** (in `core/store/hooks.ts`):

- `useHerdModulesLoadingState()` - Get current loading state
- `useIsHerdModulesLoading()` - Check if currently loading
- `useIsHerdModulesLoaded()` - Check if successfully loaded
- `
