import { HistoricalData } from "./historical";
import { IConnectivityWithCoordinates } from "./db";

export type MapDeviceIdToConnectivity = {
  [deviceId: number]: HistoricalData<IConnectivityWithCoordinates>;
};
