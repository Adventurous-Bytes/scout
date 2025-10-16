import { IHerdModule } from "../types/herd_module";

const DB_NAME = "ScoutCache";
const DB_VERSION = 1;
const HERD_MODULES_STORE = "herd_modules";
const CACHE_METADATA_STORE = "cache_metadata";

// Default TTL: 24 hours (1 day)
const DEFAULT_TTL_MS = 24 * 60 * 60 * 1000;

export interface CacheMetadata {
  key: string;
  timestamp: number;
  ttl: number; // Time to live in milliseconds
  version: string;
  etag?: string; // For conditional requests
  lastModified?: number; // For conditional requests
}

export interface CacheResult<T> {
  data: T | null;
  isStale: boolean;
  age: number;
  metadata: CacheMetadata | null;
}

export interface CacheStats {
  size: number;
  lastUpdated: number;
  isStale: boolean;
  hitRate: number;
  totalHits: number;
  totalMisses: number;
}

export interface TimingStats {
  totalDuration: number;
  cacheLoad: number;
  herdModulesApi: number;
  userApi: number;
  cacheSave: number;
  dataProcessing: number;
  localStorage: number;
}

export interface DatabaseHealth {
  healthy: boolean;
  issues: string[];
}

export class ScoutCache {
  private db: IDBDatabase | null = null;
  private initPromise: Promise<void> | null = null;
  private stats = {
    hits: 0,
    misses: 0,
  };

  private async init(): Promise<void> {
    if (this.db) return;
    if (this.initPromise) return this.initPromise;

    this.initPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        console.error("[ScoutCache] Failed to open IndexedDB:", request.error);
        this.db = null;
        this.initPromise = null;
        reject(request.error);
      };

      request.onsuccess = () => {
        this.db = request.result;

        // Validate that required object stores exist
        if (!this.validateDatabaseSchema()) {
          console.error("[ScoutCache] Database schema validation failed");
          this.db.close();
          this.db = null;
          this.initPromise = null;
          reject(new Error("Database schema validation failed"));
          return;
        }

        console.log("[ScoutCache] IndexedDB initialized successfully");
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        try {
          // Create herd modules store
          if (!db.objectStoreNames.contains(HERD_MODULES_STORE)) {
            const herdModulesStore = db.createObjectStore(HERD_MODULES_STORE, {
              keyPath: "herdId",
            });
            herdModulesStore.createIndex("timestamp", "timestamp", {
              unique: false,
            });
            console.log("[ScoutCache] Created herd_modules object store");
          }

          // Create cache metadata store
          if (!db.objectStoreNames.contains(CACHE_METADATA_STORE)) {
            const metadataStore = db.createObjectStore(CACHE_METADATA_STORE, {
              keyPath: "key",
            });
            console.log("[ScoutCache] Created cache_metadata object store");
          }

          console.log("[ScoutCache] Database schema upgraded");
        } catch (error) {
          console.error("[ScoutCache] Error during database upgrade:", error);
          reject(error);
        }
      };
    });

    return this.initPromise;
  }

  private validateDatabaseSchema(): boolean {
    if (!this.db) return false;

    const hasHerdModulesStore =
      this.db.objectStoreNames.contains(HERD_MODULES_STORE);
    const hasMetadataStore =
      this.db.objectStoreNames.contains(CACHE_METADATA_STORE);

    if (!hasHerdModulesStore) {
      console.error("[ScoutCache] Missing herd_modules object store");
    }
    if (!hasMetadataStore) {
      console.error("[ScoutCache] Missing cache_metadata object store");
    }

    return hasHerdModulesStore && hasMetadataStore;
  }

  async setHerdModules(
    herdModules: IHerdModule[],
    ttlMs: number = DEFAULT_TTL_MS,
    etag?: string,
  ): Promise<void> {
    await this.init();
    if (!this.db) throw new Error("Database not initialized");

    // Validate schema before creating transaction
    if (!this.validateDatabaseSchema()) {
      throw new Error(
        "Database schema validation failed - required object stores not found",
      );
    }

    const transaction = this.db.transaction(
      [HERD_MODULES_STORE, CACHE_METADATA_STORE],
      "readwrite",
    );

    return new Promise((resolve, reject) => {
      transaction.onerror = () => reject(transaction.error);
      transaction.oncomplete = () => resolve();

      const herdModulesStore = transaction.objectStore(HERD_MODULES_STORE);
      const metadataStore = transaction.objectStore(CACHE_METADATA_STORE);

      const timestamp = Date.now();
      const version = "1.0.0";

      // Store each herd module
      herdModules.forEach((herdModule) => {
        const cacheEntry = {
          herdId: herdModule.herd.id.toString(),
          data: herdModule,
          timestamp,
        };
        herdModulesStore.put(cacheEntry);
      });

      // Store cache metadata
      const metadata: CacheMetadata = {
        key: "herd_modules",
        timestamp,
        ttl: ttlMs,
        version,
        etag,
        lastModified: timestamp,
      };
      metadataStore.put(metadata);
    });
  }

  async getHerdModules(): Promise<CacheResult<IHerdModule[]>> {
    await this.init();
    if (!this.db) throw new Error("Database not initialized");

    // Validate schema before creating transaction
    if (!this.validateDatabaseSchema()) {
      throw new Error(
        "Database schema validation failed - required object stores not found",
      );
    }

    const transaction = this.db.transaction(
      [HERD_MODULES_STORE, CACHE_METADATA_STORE],
      "readonly",
    );

    return new Promise((resolve, reject) => {
      transaction.onerror = () => reject(transaction.error);

      const herdModulesStore = transaction.objectStore(HERD_MODULES_STORE);
      const metadataStore = transaction.objectStore(CACHE_METADATA_STORE);

      // Get metadata first
      const metadataRequest = metadataStore.get("herd_modules");
      metadataRequest.onsuccess = () => {
        const metadata: CacheMetadata | undefined = metadataRequest.result;
        const now = Date.now();

        if (!metadata) {
          this.stats.misses++;
          resolve({ data: null, isStale: true, age: 0, metadata: null });
          return;
        }

        const age = now - metadata.timestamp;
        const isStale = age > metadata.ttl;

        // Get all herd modules
        const getAllRequest = herdModulesStore.getAll();
        getAllRequest.onsuccess = () => {
          const cacheEntries = getAllRequest.result;
          const herdModules = cacheEntries
            .filter(
              (entry) => entry.data && entry.data.herd && entry.data.herd.slug,
            )
            .map((entry) => entry.data)
            .sort((a, b) =>
              (a.herd?.slug || "").localeCompare(b.herd?.slug || ""),
            );

          // Update stats
          if (herdModules.length > 0) {
            this.stats.hits++;
          } else {
            this.stats.misses++;
          }

          resolve({
            data: herdModules,
            isStale,
            age,
            metadata,
          });
        };
      };
    });
  }

  async clearHerdModules(): Promise<void> {
    await this.init();
    if (!this.db) throw new Error("Database not initialized");

    // Validate schema before creating transaction
    if (!this.validateDatabaseSchema()) {
      throw new Error(
        "Database schema validation failed - required object stores not found",
      );
    }

    const transaction = this.db.transaction(
      [HERD_MODULES_STORE, CACHE_METADATA_STORE],
      "readwrite",
    );

    return new Promise((resolve, reject) => {
      transaction.onerror = () => reject(transaction.error);
      transaction.oncomplete = () => resolve();

      const herdModulesStore = transaction.objectStore(HERD_MODULES_STORE);
      const metadataStore = transaction.objectStore(CACHE_METADATA_STORE);

      herdModulesStore.clear();
      metadataStore.delete("herd_modules");
    });
  }

  async invalidateHerdModules(): Promise<void> {
    await this.init();
    if (!this.db) throw new Error("Database not initialized");

    // Validate schema before creating transaction
    if (!this.validateDatabaseSchema()) {
      throw new Error(
        "Database schema validation failed - required object stores not found",
      );
    }

    const transaction = this.db.transaction(
      [CACHE_METADATA_STORE],
      "readwrite",
    );

    return new Promise((resolve, reject) => {
      transaction.onerror = () => reject(transaction.error);
      transaction.oncomplete = () => resolve();

      const metadataStore = transaction.objectStore(CACHE_METADATA_STORE);
      metadataStore.delete("herd_modules");
    });
  }

  async getCacheStats(): Promise<CacheStats> {
    const result = await this.getHerdModules();
    const totalRequests = this.stats.hits + this.stats.misses;
    const hitRate = totalRequests > 0 ? this.stats.hits / totalRequests : 0;

    return {
      size: result.data?.length || 0,
      lastUpdated: result.data ? Date.now() - result.age : 0,
      isStale: result.isStale,
      hitRate: Math.round(hitRate * 100) / 100,
      totalHits: this.stats.hits,
      totalMisses: this.stats.misses,
    };
  }

  async isCacheValid(ttlMs?: number): Promise<boolean> {
    const result = await this.getHerdModules();
    if (!result.data || !result.metadata) return false;

    const effectiveTtl = ttlMs || result.metadata.ttl;
    return !result.isStale && result.age < effectiveTtl;
  }

  async getCacheAge(): Promise<number> {
    const result = await this.getHerdModules();
    return result.age;
  }

  // Method to check if we should refresh based on various conditions
  async shouldRefresh(
    maxAgeMs?: number,
    forceRefresh?: boolean,
  ): Promise<{ shouldRefresh: boolean; reason: string }> {
    if (forceRefresh) {
      return { shouldRefresh: true, reason: "Force refresh requested" };
    }

    const result = await this.getHerdModules();

    if (!result.data || result.data.length === 0) {
      return { shouldRefresh: true, reason: "No cached data" };
    }

    if (result.isStale) {
      return { shouldRefresh: true, reason: "Cache is stale" };
    }

    if (maxAgeMs && result.age > maxAgeMs) {
      return {
        shouldRefresh: true,
        reason: `Cache age (${Math.round(
          result.age / 1000,
        )}s) exceeds max age (${Math.round(maxAgeMs / 1000)}s)`,
      };
    }

    return { shouldRefresh: false, reason: "Cache is valid and fresh" };
  }

  // Method to preload cache with background refresh
  async preloadCache(
    loadFunction: () => Promise<IHerdModule[]>,
    ttlMs: number = DEFAULT_TTL_MS,
  ): Promise<void> {
    try {
      console.log("[ScoutCache] Starting background cache preload...");
      const startTime = Date.now();

      const herdModules = await loadFunction();
      await this.setHerdModules(herdModules, ttlMs);

      const duration = Date.now() - startTime;
      console.log(`[ScoutCache] Background preload completed in ${duration}ms`);
    } catch (error) {
      console.warn("[ScoutCache] Background preload failed:", error);
    }
  }

  // Get the default TTL value
  getDefaultTtl(): number {
    return DEFAULT_TTL_MS;
  }

  // Method to reset the database in case of corruption
  async resetDatabase(): Promise<void> {
    console.log("[ScoutCache] Resetting database...");

    // Close existing connection
    if (this.db) {
      this.db.close();
      this.db = null;
    }
    this.initPromise = null;

    // Delete the database
    return new Promise((resolve, reject) => {
      const deleteRequest = indexedDB.deleteDatabase(DB_NAME);

      deleteRequest.onsuccess = () => {
        console.log("[ScoutCache] Database reset successfully");
        resolve();
      };

      deleteRequest.onerror = () => {
        console.error(
          "[ScoutCache] Failed to reset database:",
          deleteRequest.error,
        );
        reject(deleteRequest.error);
      };

      deleteRequest.onblocked = () => {
        console.warn(
          "[ScoutCache] Database reset blocked - close all other tabs",
        );
        // Continue anyway, it will resolve when unblocked
      };
    });
  }

  // Method to check database health
  async checkDatabaseHealth(): Promise<DatabaseHealth> {
    const issues: string[] = [];

    try {
      await this.init();

      if (!this.db) {
        issues.push("Database connection not established");
        return { healthy: false, issues };
      }

      if (!this.validateDatabaseSchema()) {
        issues.push("Database schema validation failed");
      }

      // Try a simple read operation
      try {
        await this.getHerdModules();
      } catch (error) {
        issues.push(`Read operation failed: ${error}`);
      }
    } catch (error) {
      issues.push(`Database initialization failed: ${error}`);
    }

    return {
      healthy: issues.length === 0,
      issues,
    };
  }
}

// Singleton instance
export const scoutCache = new ScoutCache();
