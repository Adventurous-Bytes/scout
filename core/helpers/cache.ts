import { IHerdModule } from "../types/herd_module";

const DB_NAME = "ScoutCache";
const DB_VERSION = 1;
const HERD_MODULES_STORE = "herd_modules";
const CACHE_METADATA_STORE = "cache_metadata";
const PROVIDERS_STORE = "providers";

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
        reject(request.error);
      };

      request.onsuccess = () => {
        this.db = request.result;
        console.log("[ScoutCache] IndexedDB initialized successfully");
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Create herd modules store
        if (!db.objectStoreNames.contains(HERD_MODULES_STORE)) {
          const herdModulesStore = db.createObjectStore(HERD_MODULES_STORE, {
            keyPath: "herdId",
          });
          herdModulesStore.createIndex("timestamp", "timestamp", {
            unique: false,
          });
        }

        // Create cache metadata store
        if (!db.objectStoreNames.contains(CACHE_METADATA_STORE)) {
          const metadataStore = db.createObjectStore(CACHE_METADATA_STORE, {
            keyPath: "key",
          });
        }

        // Create providers store
        if (!db.objectStoreNames.contains(PROVIDERS_STORE)) {
          const providersStore = db.createObjectStore(PROVIDERS_STORE, {
            keyPath: "herdId",
          });
          providersStore.createIndex("timestamp", "timestamp", {
            unique: false,
          });
        }

        console.log("[ScoutCache] Database schema upgraded");
      };
    });

    return this.initPromise;
  }

  async setHerdModules(
    herdModules: IHerdModule[],
    ttlMs: number = DEFAULT_TTL_MS,
    etag?: string,
  ): Promise<void> {
    await this.init();
    if (!this.db) throw new Error("Database not initialized");

    const transaction = this.db.transaction(
      [HERD_MODULES_STORE, CACHE_METADATA_STORE, PROVIDERS_STORE],
      "readwrite",
    );

    return new Promise((resolve, reject) => {
      transaction.onerror = () => reject(transaction.error);
      transaction.oncomplete = () => resolve();

      const herdModulesStore = transaction.objectStore(HERD_MODULES_STORE);
      const metadataStore = transaction.objectStore(CACHE_METADATA_STORE);
      const providersStore = transaction.objectStore(PROVIDERS_STORE);

      const timestamp = Date.now();
      const version = "1.0.0";

      // Store each herd module and its providers
      herdModules.forEach((herdModule) => {
        const cacheEntry = {
          herdId: herdModule.herd.id.toString(),
          data: herdModule,
          timestamp,
        };
        herdModulesStore.put(cacheEntry);

        // Store providers separately for easier querying
        if (herdModule.providers && herdModule.providers.length > 0) {
          const providersCacheEntry = {
            herdId: herdModule.herd.id.toString(),
            data: herdModule.providers,
            timestamp,
          };
          providersStore.put(providersCacheEntry);
        }
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

      // Store providers metadata
      const providersMetadata: CacheMetadata = {
        key: "providers",
        timestamp,
        ttl: ttlMs,
        version,
        etag,
        lastModified: timestamp,
      };
      metadataStore.put(providersMetadata);
    });
  }

  async getHerdModules(): Promise<CacheResult<IHerdModule[]>> {
    await this.init();
    if (!this.db) throw new Error("Database not initialized");

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

    const transaction = this.db.transaction(
      [HERD_MODULES_STORE, CACHE_METADATA_STORE, PROVIDERS_STORE],
      "readwrite",
    );

    return new Promise((resolve, reject) => {
      transaction.onerror = () => reject(transaction.error);
      transaction.oncomplete = () => resolve();

      const herdModulesStore = transaction.objectStore(HERD_MODULES_STORE);
      const metadataStore = transaction.objectStore(CACHE_METADATA_STORE);
      const providersStore = transaction.objectStore(PROVIDERS_STORE);

      herdModulesStore.clear();
      providersStore.clear();
      metadataStore.delete("herd_modules");
      metadataStore.delete("providers");
    });
  }

  async invalidateHerdModules(): Promise<void> {
    await this.init();
    if (!this.db) throw new Error("Database not initialized");

    const transaction = this.db.transaction(
      [CACHE_METADATA_STORE],
      "readwrite",
    );

    return new Promise((resolve, reject) => {
      transaction.onerror = () => reject(transaction.error);
      transaction.oncomplete = () => resolve();

      const metadataStore = transaction.objectStore(CACHE_METADATA_STORE);
      metadataStore.delete("herd_modules");
      metadataStore.delete("providers");
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

  // Method to get providers for a specific herd from cache
  async getProvidersForHerd(herdId: string): Promise<CacheResult<any[]>> {
    await this.init();
    if (!this.db) throw new Error("Database not initialized");

    const transaction = this.db.transaction(
      [PROVIDERS_STORE, CACHE_METADATA_STORE],
      "readonly",
    );

    return new Promise((resolve, reject) => {
      transaction.onerror = () => reject(transaction.error);

      const providersStore = transaction.objectStore(PROVIDERS_STORE);
      const metadataStore = transaction.objectStore(CACHE_METADATA_STORE);

      // Get metadata first
      const metadataRequest = metadataStore.get("providers");
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

        // Get providers for specific herd
        const getRequest = providersStore.get(herdId);
        getRequest.onsuccess = () => {
          const cacheEntry = getRequest.result;
          const providers = cacheEntry?.data || [];

          // Update stats
          if (providers.length > 0) {
            this.stats.hits++;
          } else {
            this.stats.misses++;
          }

          resolve({
            data: providers,
            isStale,
            age,
            metadata,
          });
        };
      };
    });
  }

  // Method to set providers for a specific herd
  async setProvidersForHerd(
    herdId: string,
    providers: any[],
    ttlMs: number = DEFAULT_TTL_MS,
  ): Promise<void> {
    await this.init();
    if (!this.db) throw new Error("Database not initialized");

    const transaction = this.db.transaction(
      [PROVIDERS_STORE, CACHE_METADATA_STORE],
      "readwrite",
    );

    return new Promise((resolve, reject) => {
      transaction.onerror = () => reject(transaction.error);
      transaction.oncomplete = () => resolve();

      const providersStore = transaction.objectStore(PROVIDERS_STORE);
      const metadataStore = transaction.objectStore(CACHE_METADATA_STORE);

      const timestamp = Date.now();
      const version = "1.0.0";

      // Store providers
      const providersCacheEntry = {
        herdId,
        data: providers,
        timestamp,
      };
      providersStore.put(providersCacheEntry);

      // Update providers metadata
      const metadata: CacheMetadata = {
        key: "providers",
        timestamp,
        ttl: ttlMs,
        version,
        lastModified: timestamp,
      };
      metadataStore.put(metadata);
    });
  }

  // Get the default TTL value
  getDefaultTtl(): number {
    return DEFAULT_TTL_MS;
  }
}

// Singleton instance
export const scoutCache = new ScoutCache();
