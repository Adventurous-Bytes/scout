import { ILocationObject } from "../types/gps";

export default function get_gps_center(gpsList: ILocationObject[]) {
  let latitude = 0;
  let longitude = 0;
  gpsList.forEach((gps) => {
    latitude += gps.latitude;
    longitude += gps.longitude;
  });
  latitude /= gpsList.length;
  longitude /= gpsList.length;
  return { latitude, longitude };
}
