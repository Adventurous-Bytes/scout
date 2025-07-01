export class GpsData {
  id: number;
  timestamp_utc: number;
  latitude: number;
  longitude: number;
  altitude: number;

  constructor(
    id: number,
    timestamp_utc: number,
    latitude: number,
    longitude: number,
    altitude: number
  ) {
    this.id = id;
    this.timestamp_utc = timestamp_utc;
    this.latitude = latitude;
    this.longitude = longitude;
    this.altitude = altitude;
  }

  // add a method to compose fromm a location object
  static fromLocationObject(location: ILocationObject) {
    return new GpsData(
      location.id,
      0,
      location.latitude,
      location.longitude,
      0
    );
  }
}

export interface IGpsMetadata {
  log_id: number;
  total_distance_km: number;
  max_speed_kph: number;
  average_speed_kph: number;
  max_altitude_m: number;
}

export interface ILocationObject {
  latitude: number;
  longitude: number;
  id: number;
}
