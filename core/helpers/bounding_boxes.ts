import {
  BoundingBox,
  EnumSourceBoundingBox,
  SelectionStatus,
} from "../types/bounding_boxes";
import { Tag } from "../types/db";

export function convertManualBoundingBoxToTag(
  boundingBox: BoundingBox,
  event_id: number
): Tag {
  const newClassName = boundingBox.label;
  // try to convert if nan, make it 0
  let newId = Number(boundingBox.id);
  if (isNaN(newId)) {
    newId = 0;
  }
  const newTag: Tag = {
    id: newId,
    x: boundingBox.xCenterPercentage,
    y: boundingBox.yCenterPercentage,
    width: boundingBox.widthPercentage,
    height: boundingBox.heightPercentage,
    inserted_at: new Date().toISOString(),
    conf: 1,
    observation_type: "manual",
    class_name: newClassName,
    event_id: event_id,
    location: null,
  };
  return newTag;
}

export function convertTagToBoundingBox(
  tag: Tag,
  source: EnumSourceBoundingBox
): BoundingBox {
  const newBoundingBox: BoundingBox = {
    xCenterPercentage: tag.x,
    yCenterPercentage: tag.y,
    widthPercentage: tag.width,
    heightPercentage: tag.height,
    label: tag.class_name,
    id: tag.id ? tag.id.toString() : "0",
    left: 0,
    top: 0,
    width: 0,
    height: 0,
    anchorPoint: { x: 0, y: 0 },
    status: SelectionStatus.ARCHIVED,
    source: source,
  };
  return newBoundingBox;
}

export function formatBoundingBoxLabel(label: string): string {
  return label;
}
