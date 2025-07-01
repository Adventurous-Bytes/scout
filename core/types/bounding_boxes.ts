export interface BoundingBox {
  xCenterPercentage: number;
  yCenterPercentage: number;
  widthPercentage: number;
  heightPercentage: number;
  label: string;
  id: string;
  left: number;
  top: number;
  width: number;
  height: number;
  anchorPoint: { x: number; y: number };
  status: SelectionStatus;
  source: EnumSourceBoundingBox;
}

export enum SelectionStatus {
  INACTIVE = "inactive",
  DRAG = "drag",
  INPUT_TAG = "input",
  FINALIZED = "finalized",
  ARCHIVED = "archived",
}

export enum EnumSourceBoundingBox {
  MANUAL = "manual",
  AI = "ai",
  ARCHIVES = "archives",
  LOCAL = "local",
}
