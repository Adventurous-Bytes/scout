import { IConnectivityWithCoordinates } from "./db";

export type MapDeviceIdToConnectivity = {
  [deviceId: number]: IConnectivityWithCoordinates[];
};


