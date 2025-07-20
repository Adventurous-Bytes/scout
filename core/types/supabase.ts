export type Json =
  | string
  | number
  | boolean
  | null
  | { [key: string]: Json | undefined }
  | Json[];

export type Database = {
  // Allows to automatically instanciate createClient with right options
  // instead of createClient<Database, { PostgrestVersion: 'XX' }>(URL, KEY)
  __InternalSupabase: {
    PostgrestVersion: "12.2.3 (519615d)";
  };
  graphql_public: {
    Tables: {
      [_ in never]: never;
    };
    Views: {
      [_ in never]: never;
    };
    Functions: {
      graphql: {
        Args: {
          operationName?: string;
          query?: string;
          variables?: Json;
          extensions?: Json;
        };
        Returns: Json;
      };
    };
    Enums: {
      [_ in never]: never;
    };
    CompositeTypes: {
      [_ in never]: never;
    };
  };
  public: {
    Tables: {
      actions: {
        Row: {
          id: number;
          inserted_at: string;
          opcode: number;
          trigger: string[];
          zone_id: number;
        };
        Insert: {
          id?: number;
          inserted_at?: string;
          opcode: number;
          trigger: string[];
          zone_id: number;
        };
        Update: {
          id?: number;
          inserted_at?: string;
          opcode?: number;
          trigger?: string[];
          zone_id?: number;
        };
        Relationships: [
          {
            foreignKeyName: "actions_zone_id_fkey";
            columns: ["zone_id"];
            isOneToOne: false;
            referencedRelation: "zones";
            referencedColumns: ["id"];
          },
          {
            foreignKeyName: "actions_zone_id_fkey";
            columns: ["zone_id"];
            isOneToOne: false;
            referencedRelation: "zones_and_actions";
            referencedColumns: ["id"];
          }
        ];
      };
      connectivity: {
        Row: {
          altitude: number;
          h11_index: string;
          h12_index: string;
          h13_index: string;
          h14_index: string;
          heading: number;
          id: number;
          inserted_at: string;
          location: unknown;
          noise: number;
          session_id: number;
          signal: number;
          timestamp_start: string;
        };
        Insert: {
          altitude: number;
          h11_index: string;
          h12_index: string;
          h13_index: string;
          h14_index: string;
          heading: number;
          id?: number;
          inserted_at?: string;
          location: unknown;
          noise: number;
          session_id: number;
          signal: number;
          timestamp_start: string;
        };
        Update: {
          altitude?: number;
          h11_index?: string;
          h12_index?: string;
          h13_index?: string;
          h14_index?: string;
          heading?: number;
          id?: number;
          inserted_at?: string;
          location?: unknown;
          noise?: number;
          session_id?: number;
          signal?: number;
          timestamp_start?: string;
        };
        Relationships: [
          {
            foreignKeyName: "connectivity_session_id_fkey";
            columns: ["session_id"];
            isOneToOne: false;
            referencedRelation: "sessions";
            referencedColumns: ["id"];
          }
        ];
      };
      devices: {
        Row: {
          altitude: number | null;
          created_by: string;
          description: string;
          device_type: Database["public"]["Enums"]["device_type"];
          domain_name: string | null;
          heading: number | null;
          herd_id: number;
          id: number;
          inserted_at: string;
          location: unknown | null;
          name: string;
          video_publisher_token: string | null;
          video_subscriber_token: string | null;
        };
        Insert: {
          altitude?: number | null;
          created_by: string;
          description: string;
          device_type?: Database["public"]["Enums"]["device_type"];
          domain_name?: string | null;
          heading?: number | null;
          herd_id: number;
          id?: number;
          inserted_at?: string;
          location?: unknown | null;
          name: string;
          video_publisher_token?: string | null;
          video_subscriber_token?: string | null;
        };
        Update: {
          altitude?: number | null;
          created_by?: string;
          description?: string;
          device_type?: Database["public"]["Enums"]["device_type"];
          domain_name?: string | null;
          heading?: number | null;
          herd_id?: number;
          id?: number;
          inserted_at?: string;
          location?: unknown | null;
          name?: string;
          video_publisher_token?: string | null;
          video_subscriber_token?: string | null;
        };
        Relationships: [
          {
            foreignKeyName: "devices_created_by_fkey";
            columns: ["created_by"];
            isOneToOne: false;
            referencedRelation: "users";
            referencedColumns: ["id"];
          },
          {
            foreignKeyName: "devices_herd_id_fkey";
            columns: ["herd_id"];
            isOneToOne: false;
            referencedRelation: "herds";
            referencedColumns: ["id"];
          }
        ];
      };
      events: {
        Row: {
          altitude: number;
          device_id: number;
          earthranger_url: string | null;
          file_path: string | null;
          heading: number;
          id: number;
          inserted_at: string;
          is_public: boolean;
          location: unknown | null;
          media_type: Database["public"]["Enums"]["media_type"];
          media_url: string | null;
          message: string | null;
          session_id: number | null;
          timestamp_observation: string;
        };
        Insert: {
          altitude?: number;
          device_id: number;
          earthranger_url?: string | null;
          file_path?: string | null;
          heading?: number;
          id?: number;
          inserted_at?: string;
          is_public?: boolean;
          location?: unknown | null;
          media_type?: Database["public"]["Enums"]["media_type"];
          media_url?: string | null;
          message?: string | null;
          session_id?: number | null;
          timestamp_observation?: string;
        };
        Update: {
          altitude?: number;
          device_id?: number;
          earthranger_url?: string | null;
          file_path?: string | null;
          heading?: number;
          id?: number;
          inserted_at?: string;
          is_public?: boolean;
          location?: unknown | null;
          media_type?: Database["public"]["Enums"]["media_type"];
          media_url?: string | null;
          message?: string | null;
          session_id?: number | null;
          timestamp_observation?: string;
        };
        Relationships: [
          {
            foreignKeyName: "events_device_id_fkey";
            columns: ["device_id"];
            isOneToOne: false;
            referencedRelation: "devices";
            referencedColumns: ["id"];
          },
          {
            foreignKeyName: "events_session_id_fkey";
            columns: ["session_id"];
            isOneToOne: false;
            referencedRelation: "sessions";
            referencedColumns: ["id"];
          }
        ];
      };
      herds: {
        Row: {
          created_by: string;
          description: string;
          earthranger_domain: string | null;
          earthranger_token: string | null;
          id: number;
          inserted_at: string;
          is_public: boolean;
          slug: string;
          video_publisher_token: string | null;
          video_server_url: string | null;
          video_subscriber_token: string | null;
        };
        Insert: {
          created_by: string;
          description: string;
          earthranger_domain?: string | null;
          earthranger_token?: string | null;
          id?: number;
          inserted_at?: string;
          is_public?: boolean;
          slug: string;
          video_publisher_token?: string | null;
          video_server_url?: string | null;
          video_subscriber_token?: string | null;
        };
        Update: {
          created_by?: string;
          description?: string;
          earthranger_domain?: string | null;
          earthranger_token?: string | null;
          id?: number;
          inserted_at?: string;
          is_public?: boolean;
          slug?: string;
          video_publisher_token?: string | null;
          video_server_url?: string | null;
          video_subscriber_token?: string | null;
        };
        Relationships: [
          {
            foreignKeyName: "herds_created_by_fkey";
            columns: ["created_by"];
            isOneToOne: false;
            referencedRelation: "users";
            referencedColumns: ["id"];
          }
        ];
      };
      plans: {
        Row: {
          herd_id: number;
          id: number;
          inserted_at: string;
          instructions: string;
          name: string;
        };
        Insert: {
          herd_id: number;
          id?: number;
          inserted_at?: string;
          instructions: string;
          name: string;
        };
        Update: {
          herd_id?: number;
          id?: number;
          inserted_at?: string;
          instructions?: string;
          name?: string;
        };
        Relationships: [
          {
            foreignKeyName: "plans_herd_id_fkey";
            columns: ["herd_id"];
            isOneToOne: false;
            referencedRelation: "herds";
            referencedColumns: ["id"];
          }
        ];
      };
      sessions: {
        Row: {
          altitude_average: number;
          altitude_max: number;
          altitude_min: number;
          device_id: number;
          distance_max_from_start: number;
          distance_total: number;
          id: number;
          inserted_at: string;
          locations: unknown;
          software_version: string;
          timestamp_end: string;
          timestamp_start: string;
          velocity_average: number;
          velocity_max: number;
          velocity_min: number;
        };
        Insert: {
          altitude_average: number;
          altitude_max: number;
          altitude_min: number;
          device_id: number;
          distance_max_from_start: number;
          distance_total: number;
          id?: number;
          inserted_at?: string;
          locations: unknown;
          software_version: string;
          timestamp_end: string;
          timestamp_start: string;
          velocity_average: number;
          velocity_max: number;
          velocity_min: number;
        };
        Update: {
          altitude_average?: number;
          altitude_max?: number;
          altitude_min?: number;
          device_id?: number;
          distance_max_from_start?: number;
          distance_total?: number;
          id?: number;
          inserted_at?: string;
          locations?: unknown;
          software_version?: string;
          timestamp_end?: string;
          timestamp_start?: string;
          velocity_average?: number;
          velocity_max?: number;
          velocity_min?: number;
        };
        Relationships: [
          {
            foreignKeyName: "sessions_device_id_fkey";
            columns: ["device_id"];
            isOneToOne: false;
            referencedRelation: "devices";
            referencedColumns: ["id"];
          }
        ];
      };
      tags: {
        Row: {
          class_name: string;
          conf: number;
          event_id: number;
          height: number;
          id: number;
          inserted_at: string;
          observation_type: Database["public"]["Enums"]["tag_observation_type"];
          width: number;
          x: number;
          y: number;
        };
        Insert: {
          class_name: string;
          conf: number;
          event_id: number;
          height?: number;
          id?: number;
          inserted_at?: string;
          observation_type: Database["public"]["Enums"]["tag_observation_type"];
          width: number;
          x: number;
          y: number;
        };
        Update: {
          class_name?: string;
          conf?: number;
          event_id?: number;
          height?: number;
          id?: number;
          inserted_at?: string;
          observation_type?: Database["public"]["Enums"]["tag_observation_type"];
          width?: number;
          x?: number;
          y?: number;
        };
        Relationships: [
          {
            foreignKeyName: "tags_event_id_fkey";
            columns: ["event_id"];
            isOneToOne: false;
            referencedRelation: "events";
            referencedColumns: ["id"];
          },
          {
            foreignKeyName: "tags_event_id_fkey";
            columns: ["event_id"];
            isOneToOne: false;
            referencedRelation: "events_with_tags";
            referencedColumns: ["id"];
          }
        ];
      };
      users: {
        Row: {
          id: string;
          username: string | null;
        };
        Insert: {
          id: string;
          username?: string | null;
        };
        Update: {
          id?: string;
          username?: string | null;
        };
        Relationships: [];
      };
      users_roles_per_herd: {
        Row: {
          herd_id: number;
          id: number;
          inserted_at: string;
          role: Database["public"]["Enums"]["role"];
          user_id: string;
        };
        Insert: {
          herd_id: number;
          id?: number;
          inserted_at?: string;
          role: Database["public"]["Enums"]["role"];
          user_id: string;
        };
        Update: {
          herd_id?: number;
          id?: number;
          inserted_at?: string;
          role?: Database["public"]["Enums"]["role"];
          user_id?: string;
        };
        Relationships: [
          {
            foreignKeyName: "users_roles_per_herd_herd_id_fkey";
            columns: ["herd_id"];
            isOneToOne: false;
            referencedRelation: "herds";
            referencedColumns: ["id"];
          },
          {
            foreignKeyName: "users_roles_per_herd_user_id_fkey";
            columns: ["user_id"];
            isOneToOne: false;
            referencedRelation: "users";
            referencedColumns: ["id"];
          }
        ];
      };
      zones: {
        Row: {
          herd_id: number;
          id: number;
          inserted_at: string;
          region: unknown;
        };
        Insert: {
          herd_id: number;
          id?: number;
          inserted_at?: string;
          region: unknown;
        };
        Update: {
          herd_id?: number;
          id?: number;
          inserted_at?: string;
          region?: unknown;
        };
        Relationships: [
          {
            foreignKeyName: "zones_herd_id_fkey";
            columns: ["herd_id"];
            isOneToOne: false;
            referencedRelation: "herds";
            referencedColumns: ["id"];
          }
        ];
      };
    };
    Views: {
      events_with_tags: {
        Row: {
          altitude: number | null;
          device_id: number | null;
          earthranger_url: string | null;
          file_path: string | null;
          heading: number | null;
          herd_id: number | null;
          id: number | null;
          inserted_at: string | null;
          is_public: boolean | null;
          location: unknown | null;
          media_type: Database["public"]["Enums"]["media_type"] | null;
          media_url: string | null;
          message: string | null;
          tags: Database["public"]["Tables"]["tags"]["Row"][] | null;
          timestamp_observation: string | null;
        };
        Relationships: [
          {
            foreignKeyName: "devices_herd_id_fkey";
            columns: ["herd_id"];
            isOneToOne: false;
            referencedRelation: "herds";
            referencedColumns: ["id"];
          },
          {
            foreignKeyName: "events_device_id_fkey";
            columns: ["device_id"];
            isOneToOne: false;
            referencedRelation: "devices";
            referencedColumns: ["id"];
          }
        ];
      };
      zones_and_actions: {
        Row: {
          actions: Database["public"]["Tables"]["actions"]["Row"][] | null;
          herd_id: number | null;
          id: number | null;
          inserted_at: string | null;
          region: unknown | null;
        };
        Relationships: [
          {
            foreignKeyName: "zones_herd_id_fkey";
            columns: ["herd_id"];
            isOneToOne: false;
            referencedRelation: "herds";
            referencedColumns: ["id"];
          }
        ];
      };
    };
    Functions: {
      authorize: {
        Args: {
          requested_permission: Database["public"]["Enums"]["app_permission"];
        };
        Returns: boolean;
      };
      create_api_key: {
        Args: { id_of_device: number };
        Returns: undefined;
      };
      create_user: {
        Args: { email: string };
        Returns: string;
      };
      custom_access_token_hook: {
        Args: { event: Json };
        Returns: Json;
      };
      get_connectivity_with_coordinates: {
        Args: { session_id_caller: number };
        Returns: Database["public"]["CompositeTypes"]["connectivity_with_coordinates"][];
      };
      get_device_by_id: {
        Args: { device_id_caller: number };
        Returns: Database["public"]["CompositeTypes"]["device_pretty_location"];
      };
      get_device_from_api_key: {
        Args: { device_api_key: string };
        Returns: {
          altitude: number | null;
          created_by: string;
          description: string;
          device_type: Database["public"]["Enums"]["device_type"];
          domain_name: string | null;
          heading: number | null;
          herd_id: number;
          id: number;
          inserted_at: string;
          location: unknown | null;
          name: string;
          video_publisher_token: string | null;
          video_subscriber_token: string | null;
        };
      };
      get_device_id_from_key: {
        Args: { device_api_key: string };
        Returns: number;
      };
      get_devices_for_herd: {
        Args: { herd_id_caller: number };
        Returns: Database["public"]["CompositeTypes"]["device_pretty_location"][];
      };
      get_events_and_tags_for_device: {
        Args: { device_id_caller: number; limit_caller: number };
        Returns: Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"][];
      };
      get_events_and_tags_for_herd: {
        Args: {
          herd_id_caller: number;
          limit_caller: number;
          offset_caller: number;
        };
        Returns: Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"][];
      };
      get_events_for_herd: {
        Args: { herd_id_in: number };
        Returns: {
          altitude: number;
          device_id: number;
          earthranger_url: string | null;
          file_path: string | null;
          heading: number;
          id: number;
          inserted_at: string;
          is_public: boolean;
          location: unknown | null;
          media_type: Database["public"]["Enums"]["media_type"];
          media_url: string | null;
          message: string | null;
          session_id: number | null;
          timestamp_observation: string;
        }[];
      };
      get_events_with_tags_by_id: {
        Args: { event_id_caller: number };
        Returns: Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"];
      };
      get_events_with_tags_for_herd: {
        Args: {
          herd_id_caller: number;
          offset_caller: number;
          limit_caller: number;
        };
        Returns: Database["public"]["CompositeTypes"]["event_with_tags"][];
      };
      get_sessions_with_coordinates: {
        Args: { herd_id_caller: number };
        Returns: Database["public"]["CompositeTypes"]["session_with_coordinates"][];
      };
      get_sessions_with_coordinates_by_device: {
        Args: { device_id_caller: number };
        Returns: Database["public"]["CompositeTypes"]["session_with_coordinates"][];
      };
      get_total_events_for_herd: {
        Args: { herd_id_caller: number };
        Returns: number;
      };
      get_zones_and_actions_for_herd: {
        Args: {
          herd_id_caller: number;
          limit_caller: number;
          offset_caller: number;
        };
        Returns: Database["public"]["CompositeTypes"]["zones_and_actions_pretty_location"][];
      };
      load_api_keys: {
        Args: { id_of_device: string };
        Returns: string[];
      };
    };
    Enums: {
      app_permission: "herds.delete" | "events.delete";
      device_type:
        | "trail_camera"
        | "drone_fixed_wing"
        | "drone_quad"
        | "gps_tracker"
        | "sentry_tower"
        | "smart_buoy"
        | "radio_mesh_base_station"
        | "radio_mesh_repeater"
        | "unknown";
      media_type: "image" | "video" | "audio" | "text";
      role: "admin" | "viewer" | "editor";
      tag_observation_type: "manual" | "auto";
      user_status: "ONLINE" | "OFFLINE";
    };
    CompositeTypes: {
      connectivity_with_coordinates: {
        id: number | null;
        session_id: number | null;
        inserted_at: string | null;
        timestamp_start: string | null;
        signal: number | null;
        noise: number | null;
        altitude: number | null;
        heading: number | null;
        latitude: number | null;
        longitude: number | null;
        h14_index: string | null;
        h13_index: string | null;
        h12_index: string | null;
        h11_index: string | null;
      };
      device_pretty_location: {
        id: number | null;
        inserted_at: string | null;
        created_by: string | null;
        herd_id: number | null;
        device_type: Database["public"]["Enums"]["device_type"] | null;
        domain_name: string | null;
        location: string | null;
        altitude: number | null;
        heading: number | null;
        name: string | null;
        description: string | null;
        latitude: number | null;
        longitude: number | null;
      };
      event_and_tags: {
        id: number | null;
        inserted_at: string | null;
        message: string | null;
        media_url: string | null;
        latitude: number | null;
        longitude: number | null;
        altitude: number | null;
        heading: number | null;
        media_type: Database["public"]["Enums"]["media_type"] | null;
        device_id: number | null;
        timestamp_observation: string | null;
        is_public: boolean | null;
        tags: Database["public"]["Tables"]["tags"]["Row"][] | null;
        herd_id: number | null;
      };
      event_and_tags_pretty_location: {
        id: number | null;
        inserted_at: string | null;
        message: string | null;
        media_url: string | null;
        file_path: string | null;
        latitude: number | null;
        longitude: number | null;
        earthranger_url: string | null;
        altitude: number | null;
        heading: number | null;
        media_type: Database["public"]["Enums"]["media_type"] | null;
        device_id: number | null;
        timestamp_observation: string | null;
        is_public: boolean | null;
        tags: Database["public"]["Tables"]["tags"]["Row"][] | null;
        herd_id: number | null;
      };
      event_plus_tags: {
        id: number | null;
        inserted_at: string | null;
        message: string | null;
        media_url: string | null;
        location: unknown | null;
        earthranger_url: string | null;
        altitude: number | null;
        heading: number | null;
        media_type: Database["public"]["Enums"]["media_type"] | null;
        device_id: number | null;
        timestamp_observation: string | null;
        is_public: boolean | null;
        tags: Database["public"]["Tables"]["tags"]["Row"][] | null;
        herd_id: number | null;
      };
      event_with_tags: {
        id: number | null;
        inserted_at: string | null;
        message: string | null;
        media_url: string | null;
        latitude: number | null;
        longitude: number | null;
        altitude: number | null;
        heading: number | null;
        media_type: Database["public"]["Enums"]["media_type"] | null;
        device_id: number | null;
        timestamp_observation: string | null;
        is_public: boolean | null;
        tags: Database["public"]["Tables"]["tags"]["Row"][] | null;
      };
      session_with_coordinates: {
        id: number | null;
        device_id: number | null;
        timestamp_start: string | null;
        timestamp_end: string | null;
        inserted_at: string | null;
        software_version: string | null;
        locations_geojson: Json | null;
        altitude_max: number | null;
        altitude_min: number | null;
        altitude_average: number | null;
        velocity_max: number | null;
        velocity_min: number | null;
        velocity_average: number | null;
        distance_total: number | null;
        distance_max_from_start: number | null;
      };
      zones_and_actions_pretty_location: {
        id: number | null;
        inserted_at: string | null;
        region: string | null;
        herd_id: number | null;
        actions: Database["public"]["Tables"]["actions"]["Row"][] | null;
      };
    };
  };
};

type DatabaseWithoutInternals = Omit<Database, "__InternalSupabase">;

type DefaultSchema = DatabaseWithoutInternals[Extract<
  keyof Database,
  "public"
>];

export type Tables<
  DefaultSchemaTableNameOrOptions extends
    | keyof (DefaultSchema["Tables"] & DefaultSchema["Views"])
    | { schema: keyof DatabaseWithoutInternals },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals;
  }
    ? keyof (DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"] &
        DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Views"])
    : never = never
> = DefaultSchemaTableNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals;
}
  ? (DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"] &
      DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Views"])[TableName] extends {
      Row: infer R;
    }
    ? R
    : never
  : DefaultSchemaTableNameOrOptions extends keyof (DefaultSchema["Tables"] &
      DefaultSchema["Views"])
  ? (DefaultSchema["Tables"] &
      DefaultSchema["Views"])[DefaultSchemaTableNameOrOptions] extends {
      Row: infer R;
    }
    ? R
    : never
  : never;

export type TablesInsert<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof DatabaseWithoutInternals },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals;
  }
    ? keyof DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never
> = DefaultSchemaTableNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals;
}
  ? DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Insert: infer I;
    }
    ? I
    : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
  ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
      Insert: infer I;
    }
    ? I
    : never
  : never;

export type TablesUpdate<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof DatabaseWithoutInternals },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals;
  }
    ? keyof DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never
> = DefaultSchemaTableNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals;
}
  ? DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Update: infer U;
    }
    ? U
    : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
  ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
      Update: infer U;
    }
    ? U
    : never
  : never;

export type Enums<
  DefaultSchemaEnumNameOrOptions extends
    | keyof DefaultSchema["Enums"]
    | { schema: keyof DatabaseWithoutInternals },
  EnumName extends DefaultSchemaEnumNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals;
  }
    ? keyof DatabaseWithoutInternals[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"]
    : never = never
> = DefaultSchemaEnumNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals;
}
  ? DatabaseWithoutInternals[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"][EnumName]
  : DefaultSchemaEnumNameOrOptions extends keyof DefaultSchema["Enums"]
  ? DefaultSchema["Enums"][DefaultSchemaEnumNameOrOptions]
  : never;

export type CompositeTypes<
  PublicCompositeTypeNameOrOptions extends
    | keyof DefaultSchema["CompositeTypes"]
    | { schema: keyof DatabaseWithoutInternals },
  CompositeTypeName extends PublicCompositeTypeNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals;
  }
    ? keyof DatabaseWithoutInternals[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"]
    : never = never
> = PublicCompositeTypeNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals;
}
  ? DatabaseWithoutInternals[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"][CompositeTypeName]
  : PublicCompositeTypeNameOrOptions extends keyof DefaultSchema["CompositeTypes"]
  ? DefaultSchema["CompositeTypes"][PublicCompositeTypeNameOrOptions]
  : never;

export const Constants = {
  graphql_public: {
    Enums: {},
  },
  public: {
    Enums: {
      app_permission: ["herds.delete", "events.delete"],
      device_type: [
        "trail_camera",
        "drone_fixed_wing",
        "drone_quad",
        "gps_tracker",
        "sentry_tower",
        "smart_buoy",
        "radio_mesh_base_station",
        "radio_mesh_repeater",
        "unknown",
      ],
      media_type: ["image", "video", "audio", "text"],
      role: ["admin", "viewer", "editor"],
      tag_observation_type: ["manual", "auto"],
      user_status: ["ONLINE", "OFFLINE"],
    },
  },
} as const;
