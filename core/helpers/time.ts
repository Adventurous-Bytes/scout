export function convertSecondsSinceEpochToDate(
  secondsSinceEpoch: number
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
