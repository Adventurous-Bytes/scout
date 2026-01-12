/** 
 * This helper checks multiple possible environment variable names to support
 * different frameworks and build tools:
 * - NEXT_PUBLIC_* (Next.js convention, also works in other frameworks)
 * - VITE_* (Vite convention)
 * - REACT_APP_* (Create React App convention)
 * - Direct access (for runtime configuration)
 */

/**
 * Get an environment variable, checking multiple possible names
 * 
 * @param name - The base name of the environment variable (without prefix)
 * @param defaultValue - Optional default value if not found
 * @returns The environment variable value or default
 */
export function getEnvVar(
  name: string,
  defaultValue: string = "",
): string {
  // Check multiple possible prefixes
  const possibleNames = [
    `NEXT_PUBLIC_${name}`, // Next.js (also works in other frameworks)
    `VITE_${name}`, // Vite
    `REACT_APP_${name}`, // Create React App
    name, // Direct access (for runtime configuration)
  ];

  // Try each possible name
  for (const envName of possibleNames) {
    if (typeof process !== "undefined" && process.env?.[envName]) {
      return process.env[envName];
    }
    // Also check window object for runtime configuration (PWA use case)
    if (typeof window !== "undefined" && (window as any).__SCOUT_CONFIG__?.[name]) {
      return (window as any).__SCOUT_CONFIG__[name];
    }
  }

  return defaultValue;
}

/**
 * Get Supabase URL from environment variables
 */
export function getSupabaseUrl(): string {
  return getEnvVar("SUPABASE_URL", "");
}

/**
 * Get Supabase anonymous key from environment variables
 */
export function getSupabaseAnonKey(): string {
  return getEnvVar("SUPABASE_ANON_KEY", "");
}
