export type Json =
  | string
  | number
  | boolean
  | null
  | { [key: string]: Json | undefined }
  | Json[]

export type Database = {
  // Allows to automatically instantiate createClient with right options
  // instead of createClient<Database, { PostgrestVersion: 'XX' }>(URL, KEY)
  __InternalSupabase: {
    PostgrestVersion: "13.0.5"
  }
  public: {
    Tables: {
      actions: {
        Row: {
          id: number
          inserted_at: string
          opcode: number
          trigger: string[]
          zone_id: number
        }
        Insert: {
          id?: number
          inserted_at?: string
          opcode: number
          trigger: string[]
          zone_id: number
        }
        Update: {
          id?: number
          inserted_at?: string
          opcode?: number
          trigger?: string[]
          zone_id?: number
        }
        Relationships: [
          {
            foreignKeyName: "actions_zone_id_fkey"
            columns: ["zone_id"]
            isOneToOne: false
            referencedRelation: "zones"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "actions_zone_id_fkey"
            columns: ["zone_id"]
            isOneToOne: false
            referencedRelation: "zones_and_actions"
            referencedColumns: ["id"]
          },
        ]
      }
      artifacts: {
        Row: {
          created_at: string
          device_id: number
          file_path: string
          id: number
          modality: string | null
          session_id: number | null
          timestamp_observation: string | null
          timestamp_observation_end: string
          updated_at: string | null
        }
        Insert: {
          created_at?: string
          device_id: number
          file_path: string
          id?: number
          modality?: string | null
          session_id?: number | null
          timestamp_observation?: string | null
          timestamp_observation_end?: string
          updated_at?: string | null
        }
        Update: {
          created_at?: string
          device_id?: number
          file_path?: string
          id?: number
          modality?: string | null
          session_id?: number | null
          timestamp_observation?: string | null
          timestamp_observation_end?: string
          updated_at?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "artifacts_device_id_fkey"
            columns: ["device_id"]
            isOneToOne: false
            referencedRelation: "devices"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "artifacts_session_id_fkey"
            columns: ["session_id"]
            isOneToOne: false
            referencedRelation: "sessions"
            referencedColumns: ["id"]
          },
        ]
      }
      certificates: {
        Row: {
          created_at: string
          expiration: string | null
          id: number
          issuer: string
          tracking_number: string | null
          type: string
          updated_at: string | null
        }
        Insert: {
          created_at?: string
          expiration?: string | null
          id?: number
          issuer: string
          tracking_number?: string | null
          type: string
          updated_at?: string | null
        }
        Update: {
          created_at?: string
          expiration?: string | null
          id?: number
          issuer?: string
          tracking_number?: string | null
          type?: string
          updated_at?: string | null
        }
        Relationships: []
      }
      chat: {
        Row: {
          created_at: string
          herd_id: number
          id: number
          message: string
          sender: string | null
        }
        Insert: {
          created_at?: string
          herd_id: number
          id?: number
          message: string
          sender?: string | null
        }
        Update: {
          created_at?: string
          herd_id?: number
          id?: number
          message?: string
          sender?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "chat_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "chat_sender_fkey"
            columns: ["sender"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      connectivity: {
        Row: {
          altitude: number
          associated_station: string | null
          bandwidth_hz: number | null
          battery_percentage: number | null
          device_id: number | null
          frequency_hz: number | null
          h11_index: string
          h12_index: string
          h13_index: string
          h14_index: string
          heading: number
          id: number
          inserted_at: string
          location: unknown
          mode: string | null
          noise: number
          session_id: number | null
          signal: number
          timestamp_start: string
        }
        Insert: {
          altitude: number
          associated_station?: string | null
          bandwidth_hz?: number | null
          battery_percentage?: number | null
          device_id?: number | null
          frequency_hz?: number | null
          h11_index: string
          h12_index: string
          h13_index: string
          h14_index: string
          heading: number
          id?: number
          inserted_at?: string
          location: unknown
          mode?: string | null
          noise: number
          session_id?: number | null
          signal: number
          timestamp_start: string
        }
        Update: {
          altitude?: number
          associated_station?: string | null
          bandwidth_hz?: number | null
          battery_percentage?: number | null
          device_id?: number | null
          frequency_hz?: number | null
          h11_index?: string
          h12_index?: string
          h13_index?: string
          h14_index?: string
          heading?: number
          id?: number
          inserted_at?: string
          location?: unknown
          mode?: string | null
          noise?: number
          session_id?: number | null
          signal?: number
          timestamp_start?: string
        }
        Relationships: [
          {
            foreignKeyName: "connectivity_device_id_fkey"
            columns: ["device_id"]
            isOneToOne: false
            referencedRelation: "devices"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "connectivity_session_id_fkey"
            columns: ["session_id"]
            isOneToOne: false
            referencedRelation: "sessions"
            referencedColumns: ["id"]
          },
        ]
      }
      devices: {
        Row: {
          altitude: number | null
          created_by: string
          description: string
          device_type: Database["public"]["Enums"]["device_type"]
          domain_name: string | null
          heading: number | null
          herd_id: number
          id: number
          inserted_at: string
          location: unknown
          name: string
          video_publisher_token: string | null
          video_subscriber_token: string | null
        }
        Insert: {
          altitude?: number | null
          created_by: string
          description: string
          device_type?: Database["public"]["Enums"]["device_type"]
          domain_name?: string | null
          heading?: number | null
          herd_id: number
          id?: number
          inserted_at?: string
          location?: unknown
          name: string
          video_publisher_token?: string | null
          video_subscriber_token?: string | null
        }
        Update: {
          altitude?: number | null
          created_by?: string
          description?: string
          device_type?: Database["public"]["Enums"]["device_type"]
          domain_name?: string | null
          heading?: number | null
          herd_id?: number
          id?: number
          inserted_at?: string
          location?: unknown
          name?: string
          video_publisher_token?: string | null
          video_subscriber_token?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "devices_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "devices_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
        ]
      }
      events: {
        Row: {
          altitude: number
          device_id: number
          earthranger_url: string | null
          file_path: string | null
          heading: number
          id: number
          inserted_at: string
          is_public: boolean
          location: unknown
          media_type: Database["public"]["Enums"]["media_type"]
          media_url: string | null
          message: string | null
          session_id: number | null
          timestamp_observation: string
        }
        Insert: {
          altitude?: number
          device_id: number
          earthranger_url?: string | null
          file_path?: string | null
          heading?: number
          id?: number
          inserted_at?: string
          is_public?: boolean
          location?: unknown
          media_type?: Database["public"]["Enums"]["media_type"]
          media_url?: string | null
          message?: string | null
          session_id?: number | null
          timestamp_observation?: string
        }
        Update: {
          altitude?: number
          device_id?: number
          earthranger_url?: string | null
          file_path?: string | null
          heading?: number
          id?: number
          inserted_at?: string
          is_public?: boolean
          location?: unknown
          media_type?: Database["public"]["Enums"]["media_type"]
          media_url?: string | null
          message?: string | null
          session_id?: number | null
          timestamp_observation?: string
        }
        Relationships: [
          {
            foreignKeyName: "events_device_id_fkey"
            columns: ["device_id"]
            isOneToOne: false
            referencedRelation: "devices"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "events_session_id_fkey"
            columns: ["session_id"]
            isOneToOne: false
            referencedRelation: "sessions"
            referencedColumns: ["id"]
          },
        ]
      }
      health_metrics: {
        Row: {
          created_at: string
          device_id: number
          id: number
          metric_name: string
          source: string | null
          timestamp: string
          unit: string | null
          value: number
        }
        Insert: {
          created_at?: string
          device_id: number
          id?: number
          metric_name: string
          source?: string | null
          timestamp: string
          unit?: string | null
          value: number
        }
        Update: {
          created_at?: string
          device_id?: number
          id?: number
          metric_name?: string
          source?: string | null
          timestamp?: string
          unit?: string | null
          value?: number
        }
        Relationships: [
          {
            foreignKeyName: "health_metrics_device_id_fkey"
            columns: ["device_id"]
            isOneToOne: false
            referencedRelation: "devices"
            referencedColumns: ["id"]
          },
        ]
      }
      heartbeats: {
        Row: {
          created_at: string
          device_id: number
          id: number
          timestamp: string
        }
        Insert: {
          created_at?: string
          device_id: number
          id?: number
          timestamp: string
        }
        Update: {
          created_at?: string
          device_id?: number
          id?: number
          timestamp?: string
        }
        Relationships: [
          {
            foreignKeyName: "heartbeats_device_id_fkey"
            columns: ["device_id"]
            isOneToOne: false
            referencedRelation: "devices"
            referencedColumns: ["id"]
          },
        ]
      }
      herds: {
        Row: {
          created_by: string
          description: string
          earthranger_domain: string | null
          earthranger_token: string | null
          id: number
          inserted_at: string
          is_public: boolean
          slug: string
          video_publisher_token: string | null
          video_server_url: string | null
          video_subscriber_token: string | null
        }
        Insert: {
          created_by: string
          description: string
          earthranger_domain?: string | null
          earthranger_token?: string | null
          id?: number
          inserted_at?: string
          is_public?: boolean
          slug: string
          video_publisher_token?: string | null
          video_server_url?: string | null
          video_subscriber_token?: string | null
        }
        Update: {
          created_by?: string
          description?: string
          earthranger_domain?: string | null
          earthranger_token?: string | null
          id?: number
          inserted_at?: string
          is_public?: boolean
          slug?: string
          video_publisher_token?: string | null
          video_server_url?: string | null
          video_subscriber_token?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "herds_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      layers: {
        Row: {
          created_at: string
          features: Json
          herd_id: number
          id: number
        }
        Insert: {
          created_at?: string
          features: Json
          herd_id: number
          id?: number
        }
        Update: {
          created_at?: string
          features?: Json
          herd_id?: number
          id?: number
        }
        Relationships: [
          {
            foreignKeyName: "layers_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
        ]
      }
      operators: {
        Row: {
          action: string | null
          created_at: string
          id: number
          session_id: number | null
          timestamp: string | null
          user_id: string
        }
        Insert: {
          action?: string | null
          created_at?: string
          id?: number
          session_id?: number | null
          timestamp?: string | null
          user_id: string
        }
        Update: {
          action?: string | null
          created_at?: string
          id?: number
          session_id?: number | null
          timestamp?: string | null
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "operators_session_id_fkey"
            columns: ["session_id"]
            isOneToOne: false
            referencedRelation: "sessions"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "operators_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      parts: {
        Row: {
          certificate_id: number | null
          created_at: string
          deleted_at: string | null
          device_id: number
          id: number
          product_number: string
          serial_number: string
          status: Database["public"]["Enums"]["component_status"]
          updated_at: string | null
        }
        Insert: {
          certificate_id?: number | null
          created_at?: string
          deleted_at?: string | null
          device_id: number
          id?: number
          product_number: string
          serial_number: string
          status?: Database["public"]["Enums"]["component_status"]
          updated_at?: string | null
        }
        Update: {
          certificate_id?: number | null
          created_at?: string
          deleted_at?: string | null
          device_id?: number
          id?: number
          product_number?: string
          serial_number?: string
          status?: Database["public"]["Enums"]["component_status"]
          updated_at?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "parts_certificate_id_fkey"
            columns: ["certificate_id"]
            isOneToOne: false
            referencedRelation: "certificates"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "parts_device_id_fkey"
            columns: ["device_id"]
            isOneToOne: false
            referencedRelation: "devices"
            referencedColumns: ["id"]
          },
        ]
      }
      pins: {
        Row: {
          altitude_relative_to_ground: number
          color: string
          created_at: string
          created_by: string | null
          description: string | null
          herd_id: number
          id: number
          location: unknown
          name: string
        }
        Insert: {
          altitude_relative_to_ground: number
          color: string
          created_at?: string
          created_by?: string | null
          description?: string | null
          herd_id: number
          id?: number
          location?: unknown
          name: string
        }
        Update: {
          altitude_relative_to_ground?: number
          color?: string
          created_at?: string
          created_by?: string | null
          description?: string | null
          herd_id?: number
          id?: number
          location?: unknown
          name?: string
        }
        Relationships: [
          {
            foreignKeyName: "pins_created_by_fkey"
            columns: ["created_by"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "pins_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
        ]
      }
      plans: {
        Row: {
          herd_id: number
          id: number
          inserted_at: string | null
          instructions: string
          name: string
          plan_type: Database["public"]["Enums"]["plan_type"]
        }
        Insert: {
          herd_id: number
          id?: number
          inserted_at?: string | null
          instructions: string
          name: string
          plan_type?: Database["public"]["Enums"]["plan_type"]
        }
        Update: {
          herd_id?: number
          id?: number
          inserted_at?: string | null
          instructions?: string
          name?: string
          plan_type?: Database["public"]["Enums"]["plan_type"]
        }
        Relationships: [
          {
            foreignKeyName: "plans_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
        ]
      }
      providers: {
        Row: {
          created_at: string
          herd_id: number
          id: number
          key: string | null
          source: string
          type: string
        }
        Insert: {
          created_at?: string
          herd_id: number
          id?: number
          key?: string | null
          source: string
          type: string
        }
        Update: {
          created_at?: string
          herd_id?: number
          id?: number
          key?: string | null
          source?: string
          type?: string
        }
        Relationships: [
          {
            foreignKeyName: "providers_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
        ]
      }
      sessions: {
        Row: {
          altitude_average: number
          altitude_max: number
          altitude_min: number
          device_id: number
          distance_max_from_start: number
          distance_total: number
          earthranger_url: string | null
          id: number
          inserted_at: string
          locations: unknown
          software_version: string
          timestamp_end: string | null
          timestamp_start: string
          velocity_average: number
          velocity_max: number
          velocity_min: number
        }
        Insert: {
          altitude_average: number
          altitude_max: number
          altitude_min: number
          device_id: number
          distance_max_from_start: number
          distance_total: number
          earthranger_url?: string | null
          id?: number
          inserted_at?: string
          locations?: unknown
          software_version: string
          timestamp_end?: string | null
          timestamp_start: string
          velocity_average: number
          velocity_max: number
          velocity_min: number
        }
        Update: {
          altitude_average?: number
          altitude_max?: number
          altitude_min?: number
          device_id?: number
          distance_max_from_start?: number
          distance_total?: number
          earthranger_url?: string | null
          id?: number
          inserted_at?: string
          locations?: unknown
          software_version?: string
          timestamp_end?: string | null
          timestamp_start?: string
          velocity_average?: number
          velocity_max?: number
          velocity_min?: number
        }
        Relationships: [
          {
            foreignKeyName: "sessions_device_id_fkey"
            columns: ["device_id"]
            isOneToOne: false
            referencedRelation: "devices"
            referencedColumns: ["id"]
          },
        ]
      }
      tags: {
        Row: {
          class_name: string
          conf: number
          event_id: number
          height: number
          id: number
          inserted_at: string
          location: unknown
          observation_type: Database["public"]["Enums"]["tag_observation_type"]
          width: number
          x: number
          y: number
        }
        Insert: {
          class_name: string
          conf: number
          event_id: number
          height?: number
          id?: number
          inserted_at?: string
          location?: unknown
          observation_type: Database["public"]["Enums"]["tag_observation_type"]
          width: number
          x: number
          y: number
        }
        Update: {
          class_name?: string
          conf?: number
          event_id?: number
          height?: number
          id?: number
          inserted_at?: string
          location?: unknown
          observation_type?: Database["public"]["Enums"]["tag_observation_type"]
          width?: number
          x?: number
          y?: number
        }
        Relationships: [
          {
            foreignKeyName: "tags_event_id_fkey"
            columns: ["event_id"]
            isOneToOne: false
            referencedRelation: "events"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "tags_event_id_fkey"
            columns: ["event_id"]
            isOneToOne: false
            referencedRelation: "events_with_tags"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "tags_event_id_fkey"
            columns: ["event_id"]
            isOneToOne: false
            referencedRelation: "events_with_tags_by_session"
            referencedColumns: ["id"]
          },
        ]
      }
      users: {
        Row: {
          id: string
          username: string | null
        }
        Insert: {
          id: string
          username?: string | null
        }
        Update: {
          id?: string
          username?: string | null
        }
        Relationships: []
      }
      users_roles_per_herd: {
        Row: {
          herd_id: number
          id: number
          inserted_at: string
          role: Database["public"]["Enums"]["role"]
          user_id: string
        }
        Insert: {
          herd_id: number
          id?: number
          inserted_at?: string
          role: Database["public"]["Enums"]["role"]
          user_id: string
        }
        Update: {
          herd_id?: number
          id?: number
          inserted_at?: string
          role?: Database["public"]["Enums"]["role"]
          user_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "users_roles_per_herd_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "users_roles_per_herd_user_id_fkey"
            columns: ["user_id"]
            isOneToOne: false
            referencedRelation: "users"
            referencedColumns: ["id"]
          },
        ]
      }
      versions_software: {
        Row: {
          broken: boolean
          commit_hash: string | null
          created_at: string
          description: string
          hyperlink: string | null
          id: number
          min: boolean
          pre: boolean
          stable: boolean
          system: string
          title: string | null
          updated_at: string | null
          version: string
        }
        Insert: {
          broken?: boolean
          commit_hash?: string | null
          created_at?: string
          description: string
          hyperlink?: string | null
          id?: number
          min?: boolean
          pre?: boolean
          stable?: boolean
          system: string
          title?: string | null
          updated_at?: string | null
          version: string
        }
        Update: {
          broken?: boolean
          commit_hash?: string | null
          created_at?: string
          description?: string
          hyperlink?: string | null
          id?: number
          min?: boolean
          pre?: boolean
          stable?: boolean
          system?: string
          title?: string | null
          updated_at?: string | null
          version?: string
        }
        Relationships: []
      }
      zones: {
        Row: {
          herd_id: number
          id: number
          inserted_at: string
          region: unknown
        }
        Insert: {
          herd_id: number
          id?: number
          inserted_at?: string
          region: unknown
        }
        Update: {
          herd_id?: number
          id?: number
          inserted_at?: string
          region?: unknown
        }
        Relationships: [
          {
            foreignKeyName: "zones_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
        ]
      }
    }
    Views: {
      events_with_tags: {
        Row: {
          altitude: number | null
          device_id: number | null
          earthranger_url: string | null
          file_path: string | null
          heading: number | null
          herd_id: number | null
          id: number | null
          inserted_at: string | null
          is_public: boolean | null
          location: unknown
          media_type: Database["public"]["Enums"]["media_type"] | null
          media_url: string | null
          message: string | null
          session_id: number | null
          tags: Database["public"]["Tables"]["tags"]["Row"][] | null
          timestamp_observation: string | null
        }
        Relationships: [
          {
            foreignKeyName: "devices_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "events_device_id_fkey"
            columns: ["device_id"]
            isOneToOne: false
            referencedRelation: "devices"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "events_session_id_fkey"
            columns: ["session_id"]
            isOneToOne: false
            referencedRelation: "sessions"
            referencedColumns: ["id"]
          },
        ]
      }
      events_with_tags_by_session: {
        Row: {
          altitude: number | null
          device_id: number | null
          earthranger_url: string | null
          file_path: string | null
          heading: number | null
          herd_id: number | null
          id: number | null
          inserted_at: string | null
          is_public: boolean | null
          location: unknown
          media_type: Database["public"]["Enums"]["media_type"] | null
          media_url: string | null
          message: string | null
          session_id: number | null
          tags: Database["public"]["Tables"]["tags"]["Row"][] | null
          timestamp_observation: string | null
        }
        Relationships: [
          {
            foreignKeyName: "devices_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "events_device_id_fkey"
            columns: ["device_id"]
            isOneToOne: false
            referencedRelation: "devices"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "events_session_id_fkey"
            columns: ["session_id"]
            isOneToOne: false
            referencedRelation: "sessions"
            referencedColumns: ["id"]
          },
        ]
      }
      zones_and_actions: {
        Row: {
          actions: Database["public"]["Tables"]["actions"]["Row"][] | null
          herd_id: number | null
          id: number | null
          inserted_at: string | null
          region: unknown
        }
        Relationships: [
          {
            foreignKeyName: "zones_herd_id_fkey"
            columns: ["herd_id"]
            isOneToOne: false
            referencedRelation: "herds"
            referencedColumns: ["id"]
          },
        ]
      }
    }
    Functions: {
      analyze_device_heartbeats: {
        Args: {
          p_device_id: number
          p_lookback_minutes?: number
          p_window_minutes?: number
        }
        Returns: Database["public"]["CompositeTypes"]["device_heartbeat_analysis"]
        SetofOptions: {
          from: "*"
          to: "device_heartbeat_analysis"
          isOneToOne: true
          isSetofReturn: false
        }
      }
      analyze_herd_device_heartbeats: {
        Args: {
          p_device_types?: Database["public"]["Enums"]["device_type"][]
          p_herd_id: number
          p_lookback_minutes?: number
          p_window_minutes?: number
        }
        Returns: Database["public"]["CompositeTypes"]["device_heartbeat_analysis"][]
        SetofOptions: {
          from: "*"
          to: "device_heartbeat_analysis"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      check_realtime_schema_status: {
        Args: never
        Returns: {
          check_type: string
          details: string
          schema_name: string
          status: string
          table_name: string
        }[]
      }
      delete_all_orphaned_sessions: {
        Args: { min_age_seconds?: number }
        Returns: {
          age_seconds: number
          device_id: number
          session_id: number
          status: string
          timestamp_start: string
        }[]
      }
      delete_orphaned_session: {
        Args: { min_age_seconds?: number; session_id_param: number }
        Returns: {
          age_seconds: number
          connectivity_count: number
          device_id: number
          session_id: number
          status: string
          timestamp_start: string
        }[]
      }
      fix_all_sessions_missing_end_timestamps: {
        Args: never
        Returns: {
          device_id: number
          new_timestamp_end: string
          old_timestamp_end: string
          session_id: number
          status: string
        }[]
      }
      fix_session_end_timestamp: {
        Args: { session_id_param: number }
        Returns: {
          new_timestamp_end: string
          old_timestamp_end: string
          session_id: number
          status: string
        }[]
      }
      get_artifacts_for_device: {
        Args: {
          device_id_caller: number
          limit_caller?: number
          offset_caller?: number
        }
        Returns: {
          created_at: string
          device_id: number
          file_path: string
          id: number
          modality: string | null
          session_id: number | null
          timestamp_observation: string | null
          timestamp_observation_end: string
          updated_at: string | null
        }[]
        SetofOptions: {
          from: "*"
          to: "artifacts"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_artifacts_for_devices_batch: {
        Args: { device_ids: number[]; limit_per_device?: number }
        Returns: {
          created_at: string
          device_id: number
          file_path: string
          id: number
          modality: string | null
          session_id: number | null
          timestamp_observation: string | null
          timestamp_observation_end: string
          updated_at: string | null
        }[]
        SetofOptions: {
          from: "*"
          to: "artifacts"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_artifacts_for_herd: {
        Args: {
          herd_id_caller: number
          limit_caller?: number
          offset_caller?: number
        }
        Returns: {
          created_at: string
          device_id: number
          file_path: string
          id: number
          modality: string | null
          session_id: number | null
          timestamp_observation: string | null
          timestamp_observation_end: string
          updated_at: string | null
        }[]
        SetofOptions: {
          from: "*"
          to: "artifacts"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_artifacts_infinite_by_device: {
        Args: {
          cursor_id?: number
          cursor_timestamp?: string
          device_id_caller: number
          limit_caller?: number
        }
        Returns: {
          created_at: string
          device_id: number
          file_path: string
          id: number
          modality: string | null
          session_id: number | null
          timestamp_observation: string | null
          timestamp_observation_end: string
          updated_at: string | null
        }[]
        SetofOptions: {
          from: "*"
          to: "artifacts"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_artifacts_infinite_by_herd: {
        Args: {
          cursor_id?: number
          cursor_timestamp?: string
          herd_id_caller: number
          limit_caller?: number
        }
        Returns: {
          created_at: string
          device_id: number
          file_path: string
          id: number
          modality: string | null
          session_id: number | null
          timestamp_observation: string | null
          timestamp_observation_end: string
          updated_at: string | null
        }[]
        SetofOptions: {
          from: "*"
          to: "artifacts"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_connectivity_with_coordinates: {
        Args: { session_id_caller: number }
        Returns: Database["public"]["CompositeTypes"]["connectivity_with_coordinates"][]
        SetofOptions: {
          from: "*"
          to: "connectivity_with_coordinates"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_connectivity_with_coordinates_by_device_and_timestamp: {
        Args: { device_id_caller: number; timestamp_filter: string }
        Returns: Database["public"]["CompositeTypes"]["connectivity_with_coordinates"][]
        SetofOptions: {
          from: "*"
          to: "connectivity_with_coordinates"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_device_by_api_key: {
        Args: { device_api_key: string }
        Returns: Database["public"]["CompositeTypes"]["device_pretty_location"]
        SetofOptions: {
          from: "*"
          to: "device_pretty_location"
          isOneToOne: true
          isSetofReturn: false
        }
      }
      get_device_by_id: {
        Args: { device_id_caller: number }
        Returns: Database["public"]["CompositeTypes"]["device_pretty_location"]
        SetofOptions: {
          from: "*"
          to: "device_pretty_location"
          isOneToOne: true
          isSetofReturn: false
        }
      }
      get_device_id_from_key: {
        Args: { device_api_key: string }
        Returns: number
      }
      get_devices_for_herd: {
        Args: { herd_id_caller: number }
        Returns: Database["public"]["CompositeTypes"]["device_pretty_location"][]
        SetofOptions: {
          from: "*"
          to: "device_pretty_location"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_events_and_tags_for_device: {
        Args: { device_id_caller: number; limit_caller: number }
        Returns: Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"][]
        SetofOptions: {
          from: "*"
          to: "event_and_tags_pretty_location"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_events_and_tags_for_devices_batch: {
        Args: { device_ids: number[]; limit_per_device?: number }
        Returns: Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"][]
        SetofOptions: {
          from: "*"
          to: "event_and_tags_pretty_location"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_events_and_tags_for_herd: {
        Args: {
          herd_id_caller: number
          limit_caller: number
          offset_caller: number
        }
        Returns: Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"][]
        SetofOptions: {
          from: "*"
          to: "event_and_tags_pretty_location"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_events_and_tags_for_session: {
        Args: {
          limit_caller: number
          offset_caller: number
          session_id_caller: number
        }
        Returns: Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"][]
        SetofOptions: {
          from: "*"
          to: "event_and_tags_pretty_location"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_events_infinite_by_device: {
        Args: {
          cursor_id?: number
          cursor_timestamp?: string
          device_id_caller: number
          limit_caller?: number
        }
        Returns: Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"][]
        SetofOptions: {
          from: "*"
          to: "event_and_tags_pretty_location"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_events_infinite_by_herd: {
        Args: {
          cursor_id?: number
          cursor_timestamp?: string
          herd_id_caller: number
          limit_caller?: number
        }
        Returns: Database["public"]["CompositeTypes"]["event_and_tags_pretty_location"][]
        SetofOptions: {
          from: "*"
          to: "event_and_tags_pretty_location"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_events_with_tags_for_herd: {
        Args: {
          herd_id_caller: number
          limit_caller: number
          offset_caller: number
        }
        Returns: Database["public"]["CompositeTypes"]["event_with_tags"][]
        SetofOptions: {
          from: "*"
          to: "event_with_tags"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_health_metrics_summary: {
        Args: { p_device_id: number; p_lookback_minutes?: number }
        Returns: {
          avg_value: number
          count: number
          max_value: number
          metric_name: string
          min_value: number
        }[]
      }
      get_herd_uptime_summary: {
        Args: {
          p_device_types?: Database["public"]["Enums"]["device_type"][]
          p_herd_id: number
          p_lookback_minutes?: number
          p_window_minutes?: number
        }
        Returns: {
          average_heartbeat_interval: number
          offline_devices: number
          online_devices: number
          overall_uptime_percentage: number
          total_devices: number
          total_heartbeats: number
        }[]
      }
      get_pins_for_herd: {
        Args: { herd_id_caller: number }
        Returns: Database["public"]["CompositeTypes"]["pins_pretty_location"][]
        SetofOptions: {
          from: "*"
          to: "pins_pretty_location"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_session_by_id: {
        Args: { session_id_caller: number }
        Returns: Database["public"]["CompositeTypes"]["session_with_coordinates"][]
        SetofOptions: {
          from: "*"
          to: "session_with_coordinates"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_session_summaries: {
        Args: {
          device_id_caller?: number
          end_date_caller?: string
          herd_id_caller?: number
          start_date_caller?: string
        }
        Returns: Json
      }
      get_session_usage_over_time: {
        Args: {
          device_id_caller?: number
          end_date_caller?: string
          herd_id_caller?: number
          start_date_caller?: string
        }
        Returns: Json
      }
      get_sessions_infinite_by_device: {
        Args: {
          cursor_id?: number
          cursor_timestamp?: string
          device_id_caller: number
          limit_caller?: number
        }
        Returns: Database["public"]["CompositeTypes"]["session_with_coordinates"][]
        SetofOptions: {
          from: "*"
          to: "session_with_coordinates"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_sessions_infinite_by_herd: {
        Args: {
          cursor_id?: number
          cursor_timestamp?: string
          herd_id_caller: number
          limit_caller?: number
        }
        Returns: Database["public"]["CompositeTypes"]["session_with_coordinates"][]
        SetofOptions: {
          from: "*"
          to: "session_with_coordinates"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_sessions_with_coordinates: {
        Args: { herd_id_caller: number }
        Returns: Database["public"]["CompositeTypes"]["session_with_coordinates"][]
        SetofOptions: {
          from: "*"
          to: "session_with_coordinates"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_sessions_with_coordinates_by_device: {
        Args: { device_id_caller: number }
        Returns: Database["public"]["CompositeTypes"]["session_with_coordinates"][]
        SetofOptions: {
          from: "*"
          to: "session_with_coordinates"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      get_total_artifacts_for_herd: {
        Args: { herd_id_caller: number }
        Returns: number
      }
      get_total_events_for_herd_with_session_filter: {
        Args: { exclude_session_events: boolean; herd_id_caller: number }
        Returns: number
      }
      get_total_events_for_session: {
        Args: { session_id_caller: number }
        Returns: number
      }
      get_zones_and_actions_for_herd: {
        Args: {
          herd_id_caller: number
          limit_caller: number
          offset_caller: number
        }
        Returns: Database["public"]["CompositeTypes"]["zones_and_actions_pretty_location"][]
        SetofOptions: {
          from: "*"
          to: "zones_and_actions_pretty_location"
          isOneToOne: false
          isSetofReturn: true
        }
      }
      load_api_keys:
        | {
            Args: { id_of_device: number }
            Returns: {
              error: true
            } & "Could not choose the best candidate function between: public.load_api_keys(id_of_device => int8), public.load_api_keys(id_of_device => text). Try renaming the parameters or the function itself in the database so function overloading can be resolved"
          }
        | {
            Args: { id_of_device: string }
            Returns: {
              error: true
            } & "Could not choose the best candidate function between: public.load_api_keys(id_of_device => int8), public.load_api_keys(id_of_device => text). Try renaming the parameters or the function itself in the database so function overloading can be resolved"
          }
      load_api_keys_batch: {
        Args: { device_ids: number[] }
        Returns: {
          api_key_id: string
          api_key_key: string
          device_id: number
        }[]
      }
      load_api_keys_old: { Args: { id_of_device: string }; Returns: string[] }
      remove_rls_broadcast_triggers: { Args: never; Returns: undefined }
    }
    Enums: {
      app_permission: "herds.delete" | "events.delete"
      component_status: "active" | "inactive"
      device_type:
        | "trail_camera"
        | "drone_fixed_wing"
        | "drone_quad"
        | "gps_tracker"
        | "sentry_tower"
        | "smart_buoy"
        | "radio_mesh_base_station"
        | "radio_mesh_repeater"
        | "unknown"
        | "gps_tracker_vehicle"
        | "gps_tracker_person"
        | "radio_mesh_base_station_gateway"
      media_type: "image" | "video" | "audio" | "text"
      plan_type: "mission" | "fence" | "rally" | "markov"
      role: "admin" | "viewer" | "editor" | "operator"
      tag_observation_type: "manual" | "auto"
      user_status: "ONLINE" | "OFFLINE"
    }
    CompositeTypes: {
      connectivity_with_coordinates: {
        id: number | null
        session_id: number | null
        device_id: number | null
        inserted_at: string | null
        timestamp_start: string | null
        signal: number | null
        noise: number | null
        altitude: number | null
        heading: number | null
        latitude: number | null
        longitude: number | null
        h14_index: string | null
        h13_index: string | null
        h12_index: string | null
        h11_index: string | null
        battery_percentage: number | null
        frequency_hz: number | null
        bandwidth_hz: number | null
        associated_station: string | null
        mode: string | null
      }
      device_heartbeat_analysis: {
        device_id: number | null
        is_online: boolean | null
        last_heartbeat_time: string | null
        minutes_since_last_heartbeat: number | null
        heartbeat_history: boolean[] | null
        uptime_percentage: number | null
        heartbeat_intervals: number[] | null
        average_heartbeat_interval: number | null
        total_heartbeats: number | null
        analysis_window_start: string | null
        analysis_window_end: string | null
      }
      device_pretty_location: {
        id: number | null
        inserted_at: string | null
        created_by: string | null
        herd_id: number | null
        device_type: Database["public"]["Enums"]["device_type"] | null
        domain_name: string | null
        location: string | null
        altitude: number | null
        heading: number | null
        name: string | null
        description: string | null
        latitude: number | null
        longitude: number | null
      }
      event_and_tags: {
        id: number | null
        inserted_at: string | null
        message: string | null
        media_url: string | null
        latitude: number | null
        longitude: number | null
        altitude: number | null
        heading: number | null
        media_type: Database["public"]["Enums"]["media_type"] | null
        device_id: number | null
        timestamp_observation: string | null
        is_public: boolean | null
        tags: Database["public"]["Tables"]["tags"]["Row"][] | null
        herd_id: number | null
      }
      event_and_tags_pretty_location: {
        id: number | null
        inserted_at: string | null
        message: string | null
        media_url: string | null
        file_path: string | null
        latitude: number | null
        longitude: number | null
        earthranger_url: string | null
        altitude: number | null
        heading: number | null
        media_type: Database["public"]["Enums"]["media_type"] | null
        device_id: number | null
        timestamp_observation: string | null
        is_public: boolean | null
        tags:
          | Database["public"]["CompositeTypes"]["tags_pretty_location"][]
          | null
        herd_id: number | null
      }
      event_plus_tags: {
        id: number | null
        inserted_at: string | null
        message: string | null
        media_url: string | null
        location: unknown
        earthranger_url: string | null
        altitude: number | null
        heading: number | null
        media_type: Database["public"]["Enums"]["media_type"] | null
        device_id: number | null
        timestamp_observation: string | null
        is_public: boolean | null
        tags: Database["public"]["Tables"]["tags"]["Row"][] | null
        herd_id: number | null
      }
      event_with_tags: {
        id: number | null
        inserted_at: string | null
        message: string | null
        media_url: string | null
        latitude: number | null
        longitude: number | null
        altitude: number | null
        heading: number | null
        media_type: Database["public"]["Enums"]["media_type"] | null
        device_id: number | null
        timestamp_observation: string | null
        is_public: boolean | null
        tags: Database["public"]["Tables"]["tags"]["Row"][] | null
      }
      pins_pretty_location: {
        id: number | null
        created_at: string | null
        location: unknown
        altitude_relative_to_ground: number | null
        color: string | null
        name: string | null
        description: string | null
        herd_id: number | null
        created_by: string | null
        latitude: number | null
        longitude: number | null
      }
      session_with_coordinates: {
        id: number | null
        device_id: number | null
        timestamp_start: string | null
        timestamp_end: string | null
        inserted_at: string | null
        software_version: string | null
        locations_geojson: Json | null
        altitude_max: number | null
        altitude_min: number | null
        altitude_average: number | null
        velocity_max: number | null
        velocity_min: number | null
        velocity_average: number | null
        distance_total: number | null
        distance_max_from_start: number | null
      }
      tags_pretty_location: {
        id: number | null
        inserted_at: string | null
        x: number | null
        y: number | null
        width: number | null
        conf: number | null
        observation_type:
          | Database["public"]["Enums"]["tag_observation_type"]
          | null
        event_id: number | null
        class_name: string | null
        height: number | null
        location: unknown
        latitude: number | null
        longitude: number | null
      }
      zones_and_actions_pretty_location: {
        id: number | null
        inserted_at: string | null
        region: string | null
        herd_id: number | null
        actions: Database["public"]["Tables"]["actions"]["Row"][] | null
      }
    }
  }
}

type DatabaseWithoutInternals = Omit<Database, "__InternalSupabase">

type DefaultSchema = DatabaseWithoutInternals[Extract<keyof Database, "public">]

export type Tables<
  DefaultSchemaTableNameOrOptions extends
    | keyof (DefaultSchema["Tables"] & DefaultSchema["Views"])
    | { schema: keyof DatabaseWithoutInternals },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof (DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"] &
        DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Views"])
    : never = never,
> = DefaultSchemaTableNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? (DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"] &
      DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Views"])[TableName] extends {
      Row: infer R
    }
    ? R
    : never
  : DefaultSchemaTableNameOrOptions extends keyof (DefaultSchema["Tables"] &
        DefaultSchema["Views"])
    ? (DefaultSchema["Tables"] &
        DefaultSchema["Views"])[DefaultSchemaTableNameOrOptions] extends {
        Row: infer R
      }
      ? R
      : never
    : never

export type TablesInsert<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof DatabaseWithoutInternals },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = DefaultSchemaTableNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Insert: infer I
    }
    ? I
    : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
    ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
        Insert: infer I
      }
      ? I
      : never
    : never

export type TablesUpdate<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof DatabaseWithoutInternals },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = DefaultSchemaTableNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? DatabaseWithoutInternals[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Update: infer U
    }
    ? U
    : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
    ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
        Update: infer U
      }
      ? U
      : never
    : never

export type Enums<
  DefaultSchemaEnumNameOrOptions extends
    | keyof DefaultSchema["Enums"]
    | { schema: keyof DatabaseWithoutInternals },
  EnumName extends DefaultSchemaEnumNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof DatabaseWithoutInternals[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"]
    : never = never,
> = DefaultSchemaEnumNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? DatabaseWithoutInternals[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"][EnumName]
  : DefaultSchemaEnumNameOrOptions extends keyof DefaultSchema["Enums"]
    ? DefaultSchema["Enums"][DefaultSchemaEnumNameOrOptions]
    : never

export type CompositeTypes<
  PublicCompositeTypeNameOrOptions extends
    | keyof DefaultSchema["CompositeTypes"]
    | { schema: keyof DatabaseWithoutInternals },
  CompositeTypeName extends PublicCompositeTypeNameOrOptions extends {
    schema: keyof DatabaseWithoutInternals
  }
    ? keyof DatabaseWithoutInternals[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"]
    : never = never,
> = PublicCompositeTypeNameOrOptions extends {
  schema: keyof DatabaseWithoutInternals
}
  ? DatabaseWithoutInternals[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"][CompositeTypeName]
  : PublicCompositeTypeNameOrOptions extends keyof DefaultSchema["CompositeTypes"]
    ? DefaultSchema["CompositeTypes"][PublicCompositeTypeNameOrOptions]
    : never

export const Constants = {
  public: {
    Enums: {
      app_permission: ["herds.delete", "events.delete"],
      component_status: ["active", "inactive"],
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
        "gps_tracker_vehicle",
        "gps_tracker_person",
        "radio_mesh_base_station_gateway",
      ],
      media_type: ["image", "video", "audio", "text"],
      plan_type: ["mission", "fence", "rally", "markov"],
      role: ["admin", "viewer", "editor", "operator"],
      tag_observation_type: ["manual", "auto"],
      user_status: ["ONLINE", "OFFLINE"],
    },
  },
} as const
