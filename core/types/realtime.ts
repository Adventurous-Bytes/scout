export enum EnumRealtimeOperation {
  INSERT,
  UPDATE,
  DELETE,
}

export type RealtimeData<T> = {
  data: T;
  operation: EnumRealtimeOperation;
};
