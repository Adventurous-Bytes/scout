# Scout Hooks

This directory contains React hooks for the Scout application.

## useScoutRefresh

A hook for refreshing scout data with detailed timing measurements, caching capabilities, and database health monitoring for performance optimization and debugging.

### Features

- **Cache-first loading**: Loads data from IndexedDB cache first, then updates with fresh data
- **Automatic refresh**: Automatically refreshes data on component mount
- **Manual refresh**: Provides a function to manually trigger refreshes
- **Detailed timing**: Measures the duration of each portion of the loading process
- **Concurrent protection**: Prevents multiple simultaneous refresh operations
- **Database health monitoring**: Check and repair IndexedDB database issues
- **Auto-recovery**: Automatically recovers from database corruption
- **Error handling**: Graceful error handling with state consistency

### Timing Measurements

The hook tracks the duration of several key operations:

1. **Cache Load** (`cacheLoad`): Time taken to load data from IndexedDB cache
2. **Herd Modules API Call** (`herdModulesApi`): Time taken to fetch herd modules from the server
3. **User API Call** (`userApi`): Time taken to fetch user data from the server
4. **Cache Save** (`cacheSave`): Time taken to save data to IndexedDB cache
5. **Data Processing** (`dataProcessing`): Time taken to process and dispatch data to the store
6. **LocalStorage Operations** (`localStorage`): Time taken for localStorage read/write operations
7. **Total Duration** (`totalDuration`): Overall time from start to completion

### Usage

```tsx
import { useScoutRefresh } from "../hooks";

function MyComponent() {
  const { 
    handleRefresh, 
    getTimingStats, 
    getCacheStats,
    checkDatabaseHealth,
    resetDatabase,
    clearCache 
  } = useScoutRefresh({
    autoRefresh: true,
    cacheFirst: true,
    cacheTtlMs: 10 * 60 * 1000, // 10 minutes cache TTL
    onRefreshComplete: () => {
      console.log("Refresh completed!");
    },
  });

  const handleManualRefresh = async () => {
    await handleRefresh();

    // Get timing statistics
    const stats = getTimingStats();
    console.log("Performance breakdown:");
    console.log(`- Cache load: ${stats.cacheLoad}ms`);
    console.log(`- Herd modules API: ${stats.herdModulesApi}ms`);
    console.log(`- User API: ${stats.userApi}ms`);
    console.log(`- Cache save: ${stats.cacheSave}ms`);
    console.log(`- Data processing: ${stats.dataProcessing}ms`);
    console.log(`- LocalStorage: ${stats.localStorage}ms`);
    console.log(`- Total: ${stats.totalDuration}ms`);
  };

  const handleCacheStats = async () => {
    const cacheStats = await getCacheStats();
    console.log("Cache statistics:");
    console.log(`- Cache size: ${cacheStats.size} items`);
    console.log(`- Hit rate: ${cacheStats.hitRate * 100}%`);
    console.log(`- Is stale: ${cacheStats.isStale}`);
  };

  const handleDatabaseHealth = async () => {
    const health = await checkDatabaseHealth();
    if (!health.healthy) {
      console.warn("Database issues found:", health.issues);
      await resetDatabase();
      console.log("Database reset completed");
    }
  };

  return (
    <div>
      <button onClick={handleManualRefresh}>Refresh Data</button>
      <button onClick={handleCacheStats}>Check Cache Stats</button>
      <button onClick={handleDatabaseHealth}>Check Database Health</button>
      <button onClick={clearCache}>Clear Cache</button>
    </div>
  );
}
```

### Cache-First Loading

The hook supports cache-first loading for improved perceived performance:

- Data is loaded from IndexedDB cache immediately
- Fresh data is fetched from API in the background
- Cache is updated with fresh data for future requests
- Configurable cache TTL (Time To Live)

### Database Health & Recovery

The hook includes robust database health monitoring and auto-recovery:

- **Automatic Detection**: Detects IndexedDB corruption and schema issues
- **Auto-Recovery**: Automatically resets corrupted databases
- **Health Checks**: Manual database health validation
- **Graceful Fallback**: Falls back to API-only mode if cache fails

### Redux State

The hook automatically updates the Redux store with timing information:

```tsx
// Access timing data from the store
const timingData = useSelector((state) => ({
  total: state.scout.herd_modules_loaded_in_ms,
  // Additional timing data available through getTimingStats()
}));
```

### Performance Monitoring

Use these timing measurements to:

- Identify performance bottlenecks in the loading process
- Monitor API response times vs cache performance
- Track data processing efficiency
- Debug localStorage performance issues
- Monitor cache hit/miss ratios
- Set performance budgets and alerts

### Error Handling

The hook includes comprehensive error handling:

- **API Response Validation**: Validates API responses before processing
- **Cache Failure Recovery**: Automatically recovers from IndexedDB errors
- **Database Corruption**: Detects and repairs corrupted databases
- **Graceful Fallbacks**: Falls back to localStorage and API-only modes
- **Consistent State**: Maintains consistent Redux state even on errors
- **Detailed Logging**: Comprehensive error logging for debugging

### Cache Management

Available cache management functions:

- `getCacheStats()`: Get cache size, hit rate, and staleness info
- `clearCache()`: Clear all cached data
- `checkDatabaseHealth()`: Validate database schema and connectivity
- `resetDatabase()`: Reset corrupted database (auto-triggered when needed)
