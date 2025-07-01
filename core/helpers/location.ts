export function format_coordinates_for_db(lat: number, long: number): string {
  return `POINT(${long} ${lat})`;
}
