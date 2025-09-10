/**
 * Enum representing the source of data in the Scout application
 */
export enum EnumDataSource {
  CACHE = "CACHE",
  DATABASE = "DATABASE",
  UNKNOWN = "UNKNOWN",
}

/**
 * Interface for data source information
 */
export interface IDataSourceInfo {
  source: EnumDataSource;
  timestamp: number;
  cacheAge?: number; // Age of cached data in milliseconds
  isStale?: boolean; // Whether the cached data is stale
}
