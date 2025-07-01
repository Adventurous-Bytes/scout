import { User } from "@supabase/supabase-js";
import { Database } from "./supabase";

// Re-export all types from Supabase
export type Role = Database["public"]["Enums"]["role"];
export type DeviceType = Database["public"]["Enums"]["device_type"];
export type MediaType = Database["public"]["Enums"]["media_type"];
export type TagObservationType =
  Database["public"]["Enums"]["tag_observation_type"];

// Re-export table types
export type IUser = User;
export type IDevice =
  Database["public"]["CompositeTypes"]["device_pretty_location"] & {
    api_keys_scout?: IApiKeyScout[];
    recent_events?: IEventWithTags[];
  };
export type IEvent = Database["public"]["Tables"]["events"]["Row"];
export type ITag = Database["public"]["Tables"]["tags"]["Row"];
export type IPlan = Database["public"]["Tables"]["plans"]["Row"];
export type IAction = Database["public"]["Tables"]["actions"]["Row"];
export type IZone = Database["public"]["Tables"]["zones"]["Row"];
export type IUserRolePerHerd =
  Database["public"]["Tables"]["users_roles_per_herd"]["Row"];
export type IHerd = Database["public"]["Tables"]["herds"]["Row"];

// Re-export composite types
export type IEventWithTags =
  Database["public"]["CompositeTypes"]["event_with_tags"] & {
    earthranger_url: string | null;
    file_path: string | null;
  };

export type IDevicePrettyLocation =
  Database["public"]["CompositeTypes"]["device_pretty_location"];
export type IEventAndTagsPrettyLocation =
  Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"];
export type IZonesAndActionsPrettyLocation =
  Database["public"]["CompositeTypes"]["zones_and_actions_pretty_location"];
// Custom types that extend Supabase types
export interface IZoneWithActions extends IZone {
  actions: IAction[];
}

export type IUserAndRole = {
  user: {
    id: string;
    username: string | null;
  } | null;
  role: Role;
};
export interface IApiKeyScout {
  id: string;
  description: string;
  key: string;
}

// Alias for ITag to maintain backward compatibility
export type Tag = ITag;

// Type for tag class names
export type TagClassName = string;

// Dummy event for testing/development
export const DUMMY_EVENT: IEvent = {
  id: 0,
  inserted_at: new Date().toISOString(),
  device_id: 0,
  message: "Dummy event",
  media_url: "",
  media_type: "image",
  location: null,
  altitude: 0,
  heading: 0,
  is_public: true,
  timestamp_observation: new Date().toISOString(),
  earthranger_url: null,
  file_path: null,
};
