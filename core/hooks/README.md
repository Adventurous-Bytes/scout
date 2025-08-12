# Scout Hooks

This directory contains React hooks for the Scout application.

## useScoutRefresh

A hook for refreshing scout data with detailed timing measurements for performance monitoring and debugging.

### Features

- **Automatic refresh**: Automatically refreshes data on component mount
- **Manual refresh**: Provides a function to manually trigger refreshes
- **Detailed timing**: Measures the duration of each portion of the loading process
- **Concurrent protection**: Prevents multiple simultaneous refresh operations
- **Error handling**: Graceful error handling with state consistency

### Timing Measurements

The hook tracks the duration of several key operations:

1. **Herd Modules API Call** (`herd_modules_api_duration_ms`): Time taken to fetch herd modules from the server
2. **User API Call** (`user_api_duration_ms`): Time taken to fetch user data from the server
3. **Data Processing** (`data_processing_duration_ms`): Time taken to process and dispatch data to the store
4. **LocalStorage Operations** (`localStorage_duration_ms`): Time taken for localStorage read/write operations
5. **Total Duration** (`herd_modules_loaded_in_ms`): Overall time from start to completion

### Usage

```tsx
import { useScoutRefresh } from "../hooks";

function MyComponent() {
  const { handleRefresh, getTimingStats } = useScoutRefresh({
    autoRefresh: true,
    onRefreshComplete: () => {
      console.log("Refresh completed!");
    },
  });

  const handleManualRefresh = async () => {
    await handleRefresh();

    // Get timing statistics
    const stats = getTimingStats();
    console.log("Performance breakdown:");
    console.log(`- Herd modules API: ${stats.herdModulesApi}ms`);
    console.log(`- User API: ${stats.userApi}ms`);
    console.log(`- Data processing: ${stats.dataProcessing}ms`);
    console.log(`- LocalStorage: ${stats.localStorage}ms`);
    console.log(`- Total: ${stats.totalDuration}ms`);
  };

  return (
    <div>
      <button onClick={handleManualRefresh}>Refresh Data</button>
    </div>
  );
}
```

### Redux State

The hook automatically updates the Redux store with timing information:

```tsx
// Access timing data from the store
const timingData = useSelector((state) => ({
  herdModulesApi: state.scout.herd_modules_api_duration_ms,
  userApi: state.scout.user_api_duration_ms,
  dataProcessing: state.scout.data_processing_duration_ms,
  localStorage: state.scout.localStorage_duration_ms,
  total: state.scout.herd_modules_loaded_in_ms,
}));
```

### Performance Monitoring

Use these timing measurements to:

- Identify performance bottlenecks in the loading process
- Monitor API response times
- Track data processing efficiency
- Debug localStorage performance issues
- Set performance budgets and alerts

### Error Handling

The hook includes comprehensive error handling:

- API response validation
- Graceful fallbacks for localStorage failures
- Consistent state updates even on errors
- Detailed error logging for debugging
