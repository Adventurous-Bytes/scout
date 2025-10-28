export function convertSecondsSinceEpochToDate(
  secondsSinceEpoch: number,
): Date {
  return new Date(secondsSinceEpoch * 1000);
}

// convert iso ISO 8601 string to date
export function convertIsoStringToDate(isoString: string): Date {
  return new Date(isoString);
}

// convert date to time string of format "HH:MM:SS, DD/MM/YYYY"
export function convertDateToTimeString(date: Date): string {
  return `${date.toLocaleTimeString()}, ${date.toLocaleDateString()} UTC`;
}

// Format a Date object as a PostgreSQL-compatible timestamp string
// Returns format: "YYYY-MM-DDTHH:MM:SS.SSSZ" (ISO 8601 with timezone)
export function formatTimestampForDatabase(date: Date): string {
  return date.toISOString();
}

// Get a timestamp for N days ago, formatted for database queries
export function getDaysAgoTimestamp(daysAgo: number): string {
  const date = new Date();
  date.setDate(date.getDate() - daysAgo);
  return formatTimestampForDatabase(date);
}

// Get a timestamp for N hours ago, formatted for database queries
export function getHoursAgoTimestamp(hoursAgo: number): string {
  const date = new Date();
  date.setHours(date.getHours() - hoursAgo);
  return formatTimestampForDatabase(date);
}

// Get a timestamp for N minutes ago, formatted for database queries
export function getMinutesAgoTimestamp(minutesAgo: number): string {
  const date = new Date();
  date.setMinutes(date.getMinutes() - minutesAgo);
  return formatTimestampForDatabase(date);
}
