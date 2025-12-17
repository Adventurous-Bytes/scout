import { SupabaseClient, User } from "@supabase/supabase-js";
import { Database } from "./supabase";

export type ScoutDatabaseClient = SupabaseClient<Database, "public">;
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
  };
export type IPin = Database["public"]["CompositeTypes"]["pins_pretty_location"];
export type IEvent = Database["public"]["Tables"]["events"]["Row"];
export type ITag = Database["public"]["Tables"]["tags"]["Row"];
export type ITagPrettyLocation =
  Database["public"]["CompositeTypes"]["tags_pretty_location"];
export type IPlan = Database["public"]["Tables"]["plans"]["Row"];
export type ILayer = Database["public"]["Tables"]["layers"]["Row"];
export type IAction = Database["public"]["Tables"]["actions"]["Row"];
export type IZone = Database["public"]["Tables"]["zones"]["Row"];
export type IUserRolePerHerd =
  Database["public"]["Tables"]["users_roles_per_herd"]["Row"];
export type IHerd = Database["public"]["Tables"]["herds"]["Row"];
export type ISession = Database["public"]["Tables"]["sessions"]["Row"];
export type IConnectivity = Database["public"]["Tables"]["connectivity"]["Row"];
export type IHeartbeat = Database["public"]["Tables"]["heartbeats"]["Row"];
export type IOperator = Database["public"]["Tables"]["operators"]["Row"];

export type IProvider = Database["public"]["Tables"]["providers"]["Row"];
export type IComponent = Database["public"]["Tables"]["components"]["Row"];
export type IVersionsSoftware =
  Database["public"]["Tables"]["versions_software"]["Row"];
export type IArtifact = Database["public"]["Tables"]["artifacts"]["Row"];

// Compound type for artifacts with signed media URL
export type IArtifactWithMediaUrl = IArtifact & {
  media_url?: string | null;
};

// Insert types
export type ComponentInsert =
  Database["public"]["Tables"]["components"]["Insert"];
export type VersionsSoftwareInsert =
  Database["public"]["Tables"]["versions_software"]["Insert"];
export type ArtifactInsert =
  Database["public"]["Tables"]["artifacts"]["Insert"];
export type PinInsert = Database["public"]["Tables"]["pins"]["Insert"];

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

// RPC function result types
export type ISessionWithCoordinates =
  Database["public"]["CompositeTypes"]["session_with_coordinates"];
export type IConnectivityWithCoordinates =
  Database["public"]["CompositeTypes"]["connectivity_with_coordinates"];

// Heartbeat analysis function return types
export type IDeviceHeartbeatAnalysis =
  Database["public"]["CompositeTypes"]["device_heartbeat_analysis"];
export type IHerdUptimeSummary =
  Database["public"]["Functions"]["get_herd_uptime_summary"]["Returns"][0];

// Custom types that extend Supabase types
export interface IZoneWithActions extends IZone {
  actions: IAction[];
}

export interface ISessionWithConnectivity extends ISession {
  connectivity: IConnectivity[];
}

export interface ISessionWithEvents extends ISession {
  events: IEvent[];
}

export interface ISessionWithConnectivityAndEvents extends ISession {
  connectivity: IConnectivity[];
  events: IEvent[];
}

export interface IConnectivityWithSession extends IConnectivity {
  session: ISession;
}

export interface IEventWithSession extends IEvent {
  session: ISession | null;
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
  key: string;
}

// Alias for ITag to maintain backward compatibility
export type Tag = ITag;

// Type for tag class names
export type TagClassName = string;

// Device-specific types for database operations
export type DeviceInsert = Database["public"]["Tables"]["devices"]["Insert"];
export type DeviceUpdate = Database["public"]["Tables"]["devices"]["Update"];
