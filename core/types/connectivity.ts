import { HistoricalData } from "./data";
import { IConnectivityWithCoordinates } from "./db";

export type MapDeviceIdToConnectivity = {
  [deviceId: number]: HistoricalData<IConnectivityWithCoordinates>;
};
