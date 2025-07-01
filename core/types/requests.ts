export enum EnumWebResponse {
  SUCCESS = "success",
  ERROR = "error",
}

export class IWebResponse<T> {
  status: EnumWebResponse;
  data: T | null;
  msg: string | null;

  constructor(status: EnumWebResponse, data: T | null, msg: string | null) {
    this.status = status;
    this.data = data;
    this.msg = msg;
  }

  static success<T>(data: T): IWebResponse<T> {
    return new IWebResponse<T>(EnumWebResponse.SUCCESS, data, null);
  }

  static error<T>(msg: string): IWebResponse<T> {
    return new IWebResponse<T>(EnumWebResponse.ERROR, null, msg);
  }

  to_compatible(): IWebResponseCompatible<T> {
    return {
      status: this.status,
      data: this.data,
      msg: this.msg,
    };
  }
}

export interface IWebResponseCompatible<T> {
  status: EnumWebResponse;
  data: T | null;
  msg: string | null;
}
