


SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;


CREATE EXTENSION IF NOT EXISTS "pg_cron" WITH SCHEMA "pg_catalog";






CREATE SCHEMA IF NOT EXISTS "gis";


ALTER SCHEMA "gis" OWNER TO "postgres";


CREATE EXTENSION IF NOT EXISTS "pg_net" WITH SCHEMA "extensions";






CREATE EXTENSION IF NOT EXISTS "pgsodium";






CREATE SCHEMA IF NOT EXISTS "private";


ALTER SCHEMA "private" OWNER TO "postgres";


COMMENT ON SCHEMA "public" IS 'standard public schema';



CREATE EXTENSION IF NOT EXISTS "pg_graphql" WITH SCHEMA "graphql";






CREATE EXTENSION IF NOT EXISTS "pg_stat_statements" WITH SCHEMA "extensions";






CREATE EXTENSION IF NOT EXISTS "pgcrypto" WITH SCHEMA "extensions";






CREATE EXTENSION IF NOT EXISTS "pgjwt" WITH SCHEMA "extensions";






CREATE EXTENSION IF NOT EXISTS "postgis" WITH SCHEMA "extensions";






CREATE EXTENSION IF NOT EXISTS "supabase_vault" WITH SCHEMA "vault";






CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA "extensions";






CREATE TYPE "public"."app_permission" AS ENUM (
    'herds.delete',
    'events.delete'
);


ALTER TYPE "public"."app_permission" OWNER TO "postgres";


CREATE TYPE "public"."component_status" AS ENUM (
    'active',
    'inactive'
);


ALTER TYPE "public"."component_status" OWNER TO "postgres";


CREATE TYPE "public"."connectivity_with_coordinates" AS (
	"id" bigint,
	"session_id" bigint,
	"device_id" bigint,
	"inserted_at" timestamp with time zone,
	"timestamp_start" timestamp with time zone,
	"signal" double precision,
	"noise" double precision,
	"altitude" double precision,
	"heading" double precision,
	"latitude" double precision,
	"longitude" double precision,
	"h14_index" "text",
	"h13_index" "text",
	"h12_index" "text",
	"h11_index" "text",
	"battery_percentage" real,
	"frequency_hz" real,
	"bandwidth_hz" real,
	"associated_station" "text",
	"mode" "text"
);


ALTER TYPE "public"."connectivity_with_coordinates" OWNER TO "postgres";


CREATE TYPE "public"."device_heartbeat_analysis" AS (
	"device_id" bigint,
	"is_online" boolean,
	"last_heartbeat_time" timestamp with time zone,
	"minutes_since_last_heartbeat" numeric,
	"heartbeat_history" boolean[],
	"uptime_percentage" integer,
	"heartbeat_intervals" numeric[],
	"average_heartbeat_interval" numeric,
	"total_heartbeats" integer,
	"analysis_window_start" timestamp with time zone,
	"analysis_window_end" timestamp with time zone
);


ALTER TYPE "public"."device_heartbeat_analysis" OWNER TO "postgres";


COMMENT ON TYPE "public"."device_heartbeat_analysis" IS 'Custom type containing comprehensive device heartbeat analysis data including:
- device_id: The analyzed device ID
- is_online: Whether device is currently considered online
- last_heartbeat_time: Timestamp of most recent heartbeat
- minutes_since_last_heartbeat: Minutes elapsed since last heartbeat
- heartbeat_history: Boolean array showing presence/absence in time windows
- uptime_percentage: Percentage of windows with heartbeats
- heartbeat_intervals: Array of intervals between consecutive heartbeats
- average_heartbeat_interval: Mean interval between heartbeats
- total_heartbeats: Total heartbeat count in analysis window';



CREATE TYPE "public"."device_type" AS ENUM (
    'trail_camera',
    'drone_fixed_wing',
    'drone_quad',
    'gps_tracker',
    'sentry_tower',
    'smart_buoy',
    'radio_mesh_base_station',
    'radio_mesh_repeater',
    'unknown',
    'gps_tracker_vehicle',
    'gps_tracker_person',
    'radio_mesh_base_station_gateway'
);


ALTER TYPE "public"."device_type" OWNER TO "postgres";


CREATE TYPE "public"."device_pretty_location" AS (
	"id" bigint,
	"inserted_at" timestamp with time zone,
	"created_by" "uuid",
	"herd_id" bigint,
	"device_type" "public"."device_type",
	"domain_name" "text",
	"location" "text",
	"altitude" double precision,
	"heading" double precision,
	"name" "text",
	"description" "text",
	"latitude" double precision,
	"longitude" double precision
);


ALTER TYPE "public"."device_pretty_location" OWNER TO "postgres";


CREATE TYPE "public"."media_type" AS ENUM (
    'image',
    'video',
    'audio',
    'text'
);


ALTER TYPE "public"."media_type" OWNER TO "postgres";


CREATE TYPE "public"."tag_observation_type" AS ENUM (
    'manual',
    'auto'
);


ALTER TYPE "public"."tag_observation_type" OWNER TO "postgres";

SET default_tablespace = '';

SET default_table_access_method = "heap";


CREATE TABLE IF NOT EXISTS "public"."tags" (
    "id" bigint NOT NULL,
    "inserted_at" timestamp with time zone DEFAULT "timezone"('utc'::"text", "now"()) NOT NULL,
    "x" double precision NOT NULL,
    "y" double precision NOT NULL,
    "width" double precision NOT NULL,
    "conf" double precision NOT NULL,
    "observation_type" "public"."tag_observation_type" NOT NULL,
    "event_id" bigint NOT NULL,
    "class_name" "text" NOT NULL,
    "height" double precision DEFAULT '0'::double precision NOT NULL,
    "location" "extensions"."geography"
);


ALTER TABLE "public"."tags" OWNER TO "postgres";


CREATE TYPE "public"."event_and_tags" AS (
	"id" bigint,
	"inserted_at" timestamp with time zone,
	"message" "text",
	"media_url" "text",
	"latitude" double precision,
	"longitude" double precision,
	"altitude" double precision,
	"heading" double precision,
	"media_type" "public"."media_type",
	"device_id" bigint,
	"timestamp_observation" timestamp with time zone,
	"is_public" boolean,
	"tags" "public"."tags"[],
	"herd_id" bigint
);


ALTER TYPE "public"."event_and_tags" OWNER TO "postgres";


CREATE TYPE "public"."tags_pretty_location" AS (
	"id" bigint,
	"inserted_at" timestamp with time zone,
	"x" double precision,
	"y" double precision,
	"width" double precision,
	"conf" double precision,
	"observation_type" "public"."tag_observation_type",
	"event_id" bigint,
	"class_name" "text",
	"height" double precision,
	"location" "extensions"."geography",
	"latitude" double precision,
	"longitude" double precision
);


ALTER TYPE "public"."tags_pretty_location" OWNER TO "postgres";


CREATE TYPE "public"."event_and_tags_pretty_location" AS (
	"id" bigint,
	"inserted_at" timestamp with time zone,
	"message" "text",
	"media_url" "text",
	"file_path" "text",
	"latitude" double precision,
	"longitude" double precision,
	"earthranger_url" "text",
	"altitude" double precision,
	"heading" double precision,
	"media_type" "public"."media_type",
	"device_id" bigint,
	"timestamp_observation" timestamp with time zone,
	"is_public" boolean,
	"tags" "public"."tags_pretty_location"[],
	"herd_id" bigint
);


ALTER TYPE "public"."event_and_tags_pretty_location" OWNER TO "postgres";


CREATE TYPE "public"."event_plus_tags" AS (
	"id" bigint,
	"inserted_at" timestamp with time zone,
	"message" "text",
	"media_url" "text",
	"location" "extensions"."geography"(Point,4326),
	"earthranger_url" "text",
	"altitude" double precision,
	"heading" double precision,
	"media_type" "public"."media_type",
	"device_id" bigint,
	"timestamp_observation" timestamp with time zone,
	"is_public" boolean,
	"tags" "public"."tags"[],
	"herd_id" bigint
);


ALTER TYPE "public"."event_plus_tags" OWNER TO "postgres";


CREATE TYPE "public"."event_with_tags" AS (
	"id" bigint,
	"inserted_at" timestamp with time zone,
	"message" "text",
	"media_url" "text",
	"latitude" double precision,
	"longitude" double precision,
	"altitude" double precision,
	"heading" double precision,
	"media_type" "public"."media_type",
	"device_id" bigint,
	"timestamp_observation" timestamp with time zone,
	"is_public" boolean,
	"tags" "public"."tags"[]
);


ALTER TYPE "public"."event_with_tags" OWNER TO "postgres";


CREATE TYPE "public"."pins_pretty_location" AS (
	"id" bigint,
	"created_at" timestamp with time zone,
	"location" "extensions"."geography",
	"altitude_relative_to_ground" bigint,
	"color" "text",
	"name" "text",
	"description" "text",
	"herd_id" bigint,
	"created_by" "uuid",
	"latitude" double precision,
	"longitude" double precision
);


ALTER TYPE "public"."pins_pretty_location" OWNER TO "postgres";


COMMENT ON TYPE "public"."pins_pretty_location" IS 'Type for pins with extracted coordinate values for easier frontend consumption. Includes both raw geography field and extracted lat/lng coordinates.';



CREATE TYPE "public"."plan_type" AS ENUM (
    'mission',
    'fence',
    'rally',
    'markov'
);


ALTER TYPE "public"."plan_type" OWNER TO "postgres";


CREATE TYPE "public"."role" AS ENUM (
    'admin',
    'viewer',
    'editor',
    'operator'
);


ALTER TYPE "public"."role" OWNER TO "postgres";


CREATE TYPE "public"."session_with_coordinates" AS (
	"id" bigint,
	"device_id" bigint,
	"timestamp_start" timestamp with time zone,
	"timestamp_end" timestamp with time zone,
	"inserted_at" timestamp with time zone,
	"software_version" "text",
	"locations_geojson" "jsonb",
	"altitude_max" double precision,
	"altitude_min" double precision,
	"altitude_average" double precision,
	"velocity_max" double precision,
	"velocity_min" double precision,
	"velocity_average" double precision,
	"distance_total" double precision,
	"distance_max_from_start" double precision
);


ALTER TYPE "public"."session_with_coordinates" OWNER TO "postgres";


CREATE TYPE "public"."user_status" AS ENUM (
    'ONLINE',
    'OFFLINE'
);


ALTER TYPE "public"."user_status" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."actions" (
    "id" bigint NOT NULL,
    "inserted_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "zone_id" bigint NOT NULL,
    "trigger" "text"[] NOT NULL,
    "opcode" integer NOT NULL
);


ALTER TABLE "public"."actions" OWNER TO "postgres";


CREATE TYPE "public"."zones_and_actions_pretty_location" AS (
	"id" bigint,
	"inserted_at" timestamp with time zone,
	"region" "text",
	"herd_id" bigint,
	"actions" "public"."actions"[]
);


ALTER TYPE "public"."zones_and_actions_pretty_location" OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."add_device_api_secret_and_keys"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
begin
  perform private.create_device_api_key_secret(new.id);
  perform private.create_api_key(new.id);
  return new;
end;
$$;


ALTER FUNCTION "private"."add_device_api_secret_and_keys"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."broadcast_parts_changes"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
    herd_id_val BIGINT;
    part_record JSONB;
BEGIN
    -- Get herd_id via device_id for all operations
    IF TG_OP = 'DELETE' THEN
        SELECT d.herd_id INTO herd_id_val
        FROM public.devices d
        WHERE d.id = OLD.device_id;
    ELSE
        SELECT d.herd_id INTO herd_id_val
        FROM public.devices d
        WHERE d.id = NEW.device_id;
    END IF;

    -- Only proceed if we found a valid herd_id
    IF herd_id_val IS NULL THEN
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        ELSE
            RETURN NEW;
        END IF;
    END IF;

    -- Build the payload based on operation type
    IF TG_OP = 'INSERT' THEN
        -- Build part record for INSERT
        part_record := row_to_json(NEW)::jsonb || jsonb_build_object('herd_id', herd_id_val);

        -- Send INSERT broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'parts',
                'schema', 'public',
                'operation', 'INSERT',
                'record', part_record,
                'old_record', NULL
            ),
            'INSERT',
            herd_id_val::text || '-parts',
            TRUE
        );

    ELSIF TG_OP = 'UPDATE' THEN
        -- Build part record for UPDATE (NEW record)
        part_record := row_to_json(NEW)::jsonb || jsonb_build_object('herd_id', herd_id_val);

        -- Send UPDATE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'parts',
                'schema', 'public',
                'operation', 'UPDATE',
                'record', part_record,
                'old_record', row_to_json(OLD)::jsonb || jsonb_build_object('herd_id', herd_id_val)
            ),
            'UPDATE',
            herd_id_val::text || '-parts',
            TRUE
        );

    ELSIF TG_OP = 'DELETE' THEN
        -- Build part record for DELETE (OLD record)
        part_record := row_to_json(OLD)::jsonb || jsonb_build_object('herd_id', herd_id_val);

        -- Send DELETE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'parts',
                'schema', 'public',
                'operation', 'DELETE',
                'record', NULL,
                'old_record', part_record
            ),
            'DELETE',
            herd_id_val::text || '-parts',
            TRUE
        );
    END IF;

    -- Return the appropriate record
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$;


ALTER FUNCTION "private"."broadcast_parts_changes"() OWNER TO "postgres";


COMMENT ON FUNCTION "private"."broadcast_parts_changes"() IS 'Broadcasts changes to the parts table via Supabase realtime. Triggers on INSERT, UPDATE, and DELETE operations. Includes herd_id for proper channel routing.';



CREATE OR REPLACE FUNCTION "private"."create_api_key"("id_of_device" bigint) RETURNS "void"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$declare
  api_key text;
  expires bigint;
  jti uuid := gen_random_uuid();
  jwt text;
  jwt_body jsonb;
  project_hash text;
  project_jwt_secret text;
  project_api_key_secret text;
  secret_uuid uuid;
  time_stamp bigint;
  device_api_key_secret text;
begin
    select into time_stamp trunc(extract(epoch from now()), 0);
    select into expires time_stamp + trunc(extract(epoch from interval '100 years'), 0);
    jwt_body := jsonb_build_object(
      'role', 'authenticated', 
      'aud', 'authenticated', 
      'iss', 'supabase', 
      'sub', to_jsonb(auth.uid()::text), 
      'iat', to_jsonb(time_stamp), 
      'exp', to_jsonb(expires), 
      'jti', to_jsonb(jti));
    select decrypted_secret into device_api_key_secret from vault.decrypted_secrets where name=id_of_device::text;
    select decrypted_secret into project_api_key_secret from vault.decrypted_secrets where name='project_api_key_secret';
    select decrypted_secret into project_jwt_secret from vault.decrypted_secrets where name='project_jwt_secret';
    select into jwt sign(jwt_body::json, project_jwt_secret);
    api_key := encode(hmac(jwt, device_api_key_secret, 'sha512'), 'hex');
    project_hash := encode(hmac(api_key, project_api_key_secret, 'sha512'), 'hex');
    
    -- Using vault.create_secret instead of direct insertion
    secret_uuid := vault.create_secret(jwt, project_hash);
    
    insert into private.jwts (secret_id, device_id) values (secret_uuid, id_of_device::bigint);
end;$$;


ALTER FUNCTION "private"."create_api_key"("id_of_device" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."create_device_api_key_secret"("id_of_device" bigint) RETURNS "void"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$declare
  rand_bytes bytea := gen_random_bytes(32);
  device_api_key_secret text := encode(digest(rand_bytes, 'sha512'), 'hex');
begin
  perform vault.create_secret(device_api_key_secret, id_of_device::text);
  -- insert into vault.secrets (secret, name) values (device_api_key_secret, id_of_device);
end;$$;


ALTER FUNCTION "private"."create_device_api_key_secret"("id_of_device" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."create_new_device_api_key"("id_of_device" bigint) RETURNS "void"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$declare
  device_api_key_secret text;
  project_api_key_secret text;
  project_jwt_secret text;
  jwt text;
  api_key text;
  project_hash text;
  secret_uuid uuid;
  jwt_body jsonb;
  time_stamp bigint;
  expires bigint;
  jti uuid;
begin
    if not exists (select 1 from public.devices where id = id_of_device) then
      raise exception 'Device % does not exist', id_of_device;
    end if;
    select into jti gen_random_uuid();
    select into time_stamp trunc(extract(epoch from now()), 0);
    select into expires time_stamp + trunc(extract(epoch from interval '100 years'), 0);
    jwt_body := jsonb_build_object(
      'role', 'authenticated',
      'aud', 'authenticated',
      'iss', 'supabase',
      'sub', to_jsonb(auth.uid()::text),
      'iat', to_jsonb(time_stamp),
      'exp', to_jsonb(expires),
      'jti', to_jsonb(jti));
    select decrypted_secret into device_api_key_secret from vault.decrypted_secrets where name=id_of_device::text;
    select decrypted_secret into project_api_key_secret from vault.decrypted_secrets where name='project_api_key_secret';
    select decrypted_secret into project_jwt_secret from vault.decrypted_secrets where name='project_jwt_secret';
    select into jwt sign(jwt_body::json, project_jwt_secret);
    api_key := encode(extensions.hmac(jwt, device_api_key_secret, 'sha512'), 'hex');
    project_hash := encode(extensions.hmac(api_key, project_api_key_secret, 'sha512'), 'hex');

    -- Using vault.create_secret instead of direct insertion
    secret_uuid := vault.create_secret(jwt, project_hash);

    insert into private.jwts (secret_id, device_id) values (secret_uuid, id_of_device::bigint);
end;$$;


ALTER FUNCTION "private"."create_new_device_api_key"("id_of_device" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."event_belongs_to_device"("device_id_caller" bigint, "event_id_caller" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$begin
  return exists (
    select 1 from public.events where id = event_id_caller and device_id = device_id_caller
  );
end;$$;


ALTER FUNCTION "private"."event_belongs_to_device"("device_id_caller" bigint, "event_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."extract_herd_id_from_topic"("topic_name" "text") RETURNS bigint
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
  herd_id_part TEXT;
  herd_id BIGINT;
BEGIN
  -- Split on the first dash and take the first part
  herd_id_part := split_part(topic_name, '-', 1);

  -- Try to convert to bigint, return NULL if invalid
  BEGIN
    herd_id := herd_id_part::BIGINT;
    RETURN herd_id;
  EXCEPTION WHEN others THEN
    RETURN NULL;
  END;
END;
$$;


ALTER FUNCTION "private"."extract_herd_id_from_topic"("topic_name" "text") OWNER TO "postgres";


COMMENT ON FUNCTION "private"."extract_herd_id_from_topic"("topic_name" "text") IS 'Extracts herd_id from realtime topic names with format "herd_id-topic"';



CREATE OR REPLACE FUNCTION "private"."fill_event_location"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO ''
    AS $$
begin
  if new.location is null then
    select into new.location location from public.devices where id = new.device_id;
  end if;
  if new.altitude is null then
    select into new.altitude altitude from public.devices where id = new.device_id;
  end if;
  if new.heading is null then
    select into new.heading heading from public.devices where id = new.device_id;
  end if;
  return new;
end;
$$;


ALTER FUNCTION "private"."fill_event_location"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."get_device_id_by_api_key"("device_api_key" "text") RETURNS bigint
    LANGUAGE "plpgsql"
    AS $$declare
  project_hash text;
  project_api_key_secret text;
  secret_uuid uuid;
begin
  select decrypted_secret into project_api_key_secret from vault.decrypted_secrets where name='project_api_key_secret';
  project_hash := encode(extensions.hmac(device_api_key, project_api_key_secret, 'sha512'), 'hex');
  select id into secret_uuid from vault.secrets where name=project_hash;
  if secret_uuid is not null then
      return (select private.jwts.device_id from private.jwts where secret_id=secret_uuid);
  end if;
end;$$;


ALTER FUNCTION "private"."get_device_id_by_api_key"("device_api_key" "text") OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."get_device_id_from_path"("object_name" "text") RETURNS bigint
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
BEGIN
  -- Extract device_id from path: {herd_id}/{device_id}/filename
  -- Split by '/' and get the second element (index 2)
  RETURN (string_to_array(object_name, '/'))[2]::bigint;
EXCEPTION
  WHEN OTHERS THEN
    RETURN NULL;
END;
$$;


ALTER FUNCTION "private"."get_device_id_from_path"("object_name" "text") OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."herds" (
    "id" bigint NOT NULL,
    "inserted_at" timestamp with time zone DEFAULT "timezone"('utc'::"text", "now"()) NOT NULL,
    "slug" "text" NOT NULL,
    "description" "text" NOT NULL,
    "created_by" "uuid" NOT NULL,
    "is_public" boolean DEFAULT false NOT NULL,
    "earthranger_domain" "text",
    "earthranger_token" "text",
    "video_publisher_token" "text",
    "video_subscriber_token" "text",
    "video_server_url" "text"
);


ALTER TABLE "public"."herds" OWNER TO "postgres";


COMMENT ON TABLE "public"."herds" IS 'Topics and groups.';



CREATE OR REPLACE FUNCTION "private"."get_herd_by_event_id"("event_id" bigint) RETURNS "public"."herds"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$declare
  device_id_caller bigint;
  herd_id_caller bigint;
begin
  select device_id into device_id_caller from public.events where id = event_id;
  select herd_id into herd_id_caller from public.devices where id = device_id_caller;
  return (
    select * from public.herds where id = herd_id_caller
  );
end;$$;


ALTER FUNCTION "private"."get_herd_by_event_id"("event_id" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."get_herd_by_zone_id"("zone_id" bigint) RETURNS bigint
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
declare
  herd_id_caller bigint;
begin
  select herd_id into herd_id_caller from zones where id = zone_id;
  return herd_id_caller;
end;
$$;


ALTER FUNCTION "private"."get_herd_by_zone_id"("zone_id" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."get_herd_id_by_device_id"("device_id" bigint) RETURNS bigint
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$declare
  herd_id_caller bigint;
begin
  select herd_id into herd_id_caller from public.devices where id = device_id;
  return herd_id_caller;
end;$$;


ALTER FUNCTION "private"."get_herd_id_by_device_id"("device_id" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."get_herd_id_by_event_id"("event_id" bigint) RETURNS bigint
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
declare
  device_id_caller bigint;
  herd_id_caller bigint;
begin
  select device_id into device_id_caller from events where id = event_id;
  select herd_id into herd_id_caller from devices where id = device_id_caller;
  return herd_id_caller;
end;
$$;


ALTER FUNCTION "private"."get_herd_id_by_event_id"("event_id" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."get_herd_id_by_session_id"("session_id_param" bigint) RETURNS bigint
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
  herd_id_result bigint;
BEGIN
  SELECT d.herd_id INTO herd_id_result
  FROM public.sessions s
  JOIN public.devices d ON s.device_id = d.id
  WHERE s.id = session_id_param;
  
  RETURN herd_id_result;
END;
$$;


ALTER FUNCTION "private"."get_herd_id_by_session_id"("session_id_param" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "private"."get_herd_id_by_session_id"("session_id_param" bigint) IS 'Gets herd_id from session_id by joining sessions -> devices';



CREATE OR REPLACE FUNCTION "private"."get_herd_id_from_path"("object_name" "text") RETURNS bigint
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
BEGIN
  -- Extract herd_id from path: {herd_id}/{device_id}/filename
  -- Split by '/' and get the first element (index 1)
  RETURN (string_to_array(object_name, '/'))[1]::bigint;
EXCEPTION
  WHEN OTHERS THEN
    RETURN NULL;
END;
$$;


ALTER FUNCTION "private"."get_herd_id_from_path"("object_name" "text") OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."has_admin_role_any_herd"("user_id_caller" "uuid") RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$
begin
  -- Check if the user has admin role in any herd
  return exists (
    select 1
    from public.users_roles_per_herd
    where user_id = user_id_caller
    and role = 'admin'
  );
end;
$$;


ALTER FUNCTION "private"."has_admin_role_any_herd"("user_id_caller" "uuid") OWNER TO "postgres";


COMMENT ON FUNCTION "private"."has_admin_role_any_herd"("user_id_caller" "uuid") IS 'Returns true if the user has admin role in any herd, used for global admin permissions like software version management';



CREATE OR REPLACE FUNCTION "private"."has_good_admin_role"("user_id_caller" "uuid", "herd_id_caller" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$
begin
  return exists (
    select 1 from public.users_roles_per_herd where user_id = (user_id_caller) and herd_id = herd_id_caller and role in ('admin')
  );
end;
$$;


ALTER FUNCTION "private"."has_good_admin_role"("user_id_caller" "uuid", "herd_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."has_good_edit_role"("user_id_caller" "uuid", "herd_id_caller" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$
begin
  return exists (
    select 1 from public.users_roles_per_herd where user_id = (user_id_caller) and herd_id = herd_id_caller and role in ('admin', 'editor')
  );
end;
$$;


ALTER FUNCTION "private"."has_good_edit_role"("user_id_caller" "uuid", "herd_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."has_good_view_role"("user_id_caller" "uuid", "herd_id_caller" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$
begin
  return exists (
    select 1 from public.users_roles_per_herd where user_id = (user_id_caller) and herd_id = herd_id_caller and role in ('admin', 'editor', 'viewer')
  );
end;
$$;


ALTER FUNCTION "private"."has_good_view_role"("user_id_caller" "uuid", "herd_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."herd_has_device"("herd_id_caller" bigint, "device_id_caller" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$begin
  return exists (
    select 1 from public.devices where id = device_id_caller and herd_id = herd_id_caller
  );
end;$$;


ALTER FUNCTION "private"."herd_has_device"("herd_id_caller" bigint, "device_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."is_approved_gateway_device_type"("device_id_caller" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
BEGIN
  RETURN EXISTS (
    SELECT 1
    FROM public.devices
    WHERE id = device_id_caller
    AND device_type IN ('radio_mesh_base_station_gateway', 'radio_mesh_base_station', 'radio_mesh_repeater')
  );
END;
$$;


ALTER FUNCTION "private"."is_approved_gateway_device_type"("device_id_caller" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "private"."is_approved_gateway_device_type"("device_id_caller" bigint) IS 'Returns true if the specified device is of an approved gateway device type (radio_mesh_base_station_gateway, radio_mesh_base_station, or radio_mesh_repeater)';



CREATE OR REPLACE FUNCTION "private"."is_device_authorized_for_event"("p_event_id" bigint, "p_device_id" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO ''
    AS $$
DECLARE
  v_is_authorized boolean;
BEGIN
  -- Check if the device owns the event
  SELECT EXISTS (
    SELECT 1
    FROM public.events
    WHERE events.id = p_event_id
    AND events.device_id = p_device_id
  ) INTO v_is_authorized;
  
  RETURN v_is_authorized;
END;
$$;


ALTER FUNCTION "private"."is_device_authorized_for_event"("p_event_id" bigint, "p_device_id" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."is_herd_creator"("user_id_caller" "uuid", "herd_id_caller" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$
begin
  return exists (
    select 1 from public.herds where id = herd_id_caller and created_by = user_id_caller
  );
end;
$$;


ALTER FUNCTION "private"."is_herd_creator"("user_id_caller" "uuid", "herd_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."is_radio_mesh_gateway"("device_id_caller" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
BEGIN
  RETURN EXISTS (
    SELECT 1
    FROM public.devices
    WHERE id = device_id_caller
    AND device_type = 'radio_mesh_base_station_gateway'
  );
END;
$$;


ALTER FUNCTION "private"."is_radio_mesh_gateway"("device_id_caller" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "private"."is_radio_mesh_gateway"("device_id_caller" bigint) IS 'Returns true if the specified device is of type radio_mesh_base_station_gateway';



CREATE OR REPLACE FUNCTION "private"."key_uid"() RETURNS bigint
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$declare
  project_hash text;
  project_api_key_secret text;
  secret_uuid uuid;
  device_api_key text;
begin
  select current_setting('request.headers', true)::json->>'api_key' into device_api_key;
  select decrypted_secret into project_api_key_secret from vault.decrypted_secrets where name='project_api_key_secret';
  project_hash := encode(extensions.hmac(device_api_key, project_api_key_secret, 'sha512'), 'hex');
  select id into secret_uuid from vault.secrets where name=project_hash;
  if secret_uuid is not null then
      return (select private.jwts.device_id from private.jwts where secret_id=secret_uuid);
  end if;
  return 0;
end;$$;


ALTER FUNCTION "private"."key_uid"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."remove_device_api_secret_and_keys"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public'
    AS $$declare
  jwt_record record;
begin
  delete from vault.secrets where name=old.id::text;
  for jwt_record IN 
    select secret_id
    from private.jwts 
    where device_id=old.id
  loop
    delete from vault.secrets where id=jwt_record.secret_id;
  end loop;
  -- also delete jwts
  delete from private.jwts where device_id=old.id;
  return old;
end;$$;


ALTER FUNCTION "private"."remove_device_api_secret_and_keys"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "private"."user_can_access_herd_topic"("user_id_param" "uuid", "topic_name" "text") RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
  topic_herd_id BIGINT;
BEGIN
  -- Extract herd_id from topic name
  topic_herd_id := private.extract_herd_id_from_topic(topic_name);

  -- If we can't extract a valid herd_id, deny access
  IF topic_herd_id IS NULL THEN
    RETURN FALSE;
  END IF;

  -- Check if user has any role in this herd (viewer, editor, operator, admin)
  RETURN EXISTS (
    SELECT 1
    FROM public.users_roles_per_herd
    WHERE user_id = user_id_param
      AND herd_id = topic_herd_id
      AND role IN ('admin', 'editor', 'operator', 'viewer')
  );
END;
$$;


ALTER FUNCTION "private"."user_can_access_herd_topic"("user_id_param" "uuid", "topic_name" "text") OWNER TO "postgres";


COMMENT ON FUNCTION "private"."user_can_access_herd_topic"("user_id_param" "uuid", "topic_name" "text") IS 'Checks if a user has access to a herd based on topic name format "herd_id-topic". All roles (admin, editor, operator, viewer) have access.';



CREATE OR REPLACE FUNCTION "private"."users_share_herd"("user_id_caller" "uuid", "user_id_target" "uuid") RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$
begin
  return exists (
    select 1 from public.users_roles_per_herd where user_id = (user_id_caller) and herd_id in (select herd_id from public.users_roles_per_herd where user_id = user_id_target)
  );
end;
$$;


ALTER FUNCTION "private"."users_share_herd"("user_id_caller" "uuid", "user_id_target" "uuid") OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."add_user_as_herd_admin"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO ''
    AS $$begin
  insert into public.users_roles_per_herd (user_id, herd_id, role) values (new.created_by, new.id, 'admin');
  return new;
end;$$;


ALTER FUNCTION "public"."add_user_as_herd_admin"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."analyze_device_heartbeats"("p_device_id" bigint, "p_lookback_minutes" integer DEFAULT 60, "p_window_minutes" integer DEFAULT 2) RETURNS "public"."device_heartbeat_analysis"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO ''
    AS $$
DECLARE
    v_result public.device_heartbeat_analysis;
    v_window_count integer;
    v_heartbeat_history boolean[] := '{}';
    v_heartbeat_intervals numeric[] := '{}';
    v_online_windows integer := 0;
    v_now timestamp with time zone := now();
    v_analysis_start timestamp with time zone;
    v_device_herd_id bigint;
    v_last_heartbeat record;
    v_window_data record;
    v_interval_data record;
BEGIN
    -- SECURITY CHECK: Get device's herd_id and verify it exists
    SELECT d.herd_id INTO v_device_herd_id
    FROM public.devices d
    WHERE d.id = p_device_id;

    IF v_device_herd_id IS NULL THEN
        RAISE EXCEPTION 'Device % not found', p_device_id;
    END IF;

    -- SECURITY CHECK: Verify user has view access to this herd OR is the device itself OR is postgres user
    IF NOT (
        private.has_good_view_role((SELECT auth.uid()), v_device_herd_id)
        OR
        p_device_id = private.key_uid()
        OR
        current_user = 'postgres'
    ) THEN
        RAISE EXCEPTION 'Access denied to device %', p_device_id;
    END IF;

    -- Input validation
    IF p_lookback_minutes <= 0 OR p_window_minutes <= 0 THEN
        RAISE EXCEPTION 'Lookback and window minutes must be positive';
    END IF;

    -- Initialize result
    v_result.device_id := p_device_id;
    v_result.analysis_window_end := v_now;
    v_analysis_start := v_now - (p_lookback_minutes || ' minutes')::interval;
    v_result.analysis_window_start := v_analysis_start;
    v_window_count := p_lookback_minutes / p_window_minutes;

    -- Get last heartbeat info efficiently
    SELECT
        h.timestamp,
        EXTRACT(EPOCH FROM (v_now - h.timestamp)) / 60 as minutes_since
    INTO v_last_heartbeat
    FROM public.heartbeats h
    WHERE h.device_id = p_device_id
    ORDER BY h.timestamp DESC
    LIMIT 1;

    -- Set online status and last heartbeat info
    IF v_last_heartbeat.timestamp IS NOT NULL THEN
        v_result.last_heartbeat_time := v_last_heartbeat.timestamp;
        v_result.minutes_since_last_heartbeat := v_last_heartbeat.minutes_since;
        v_result.is_online := v_last_heartbeat.minutes_since <= 5;
    ELSE
        v_result.last_heartbeat_time := NULL;
        v_result.minutes_since_last_heartbeat := NULL;
        v_result.is_online := false;
    END IF;

    -- Generate heartbeat history using efficient set-based approach
    WITH time_windows AS (
        SELECT
            generate_series(0, v_window_count - 1) as window_idx,
            v_now - ((generate_series(0, v_window_count - 1) + 1) * p_window_minutes || ' minutes')::interval as window_start,
            v_now - (generate_series(0, v_window_count - 1) * p_window_minutes || ' minutes')::interval as window_end
    ),
    window_heartbeats AS (
        SELECT
            tw.window_idx,
            EXISTS(
                SELECT 1
                FROM public.heartbeats h
                WHERE h.device_id = p_device_id
                AND h.timestamp >= tw.window_start
                AND h.timestamp < tw.window_end
            ) as has_heartbeat
        FROM time_windows tw
        ORDER BY tw.window_idx
    )
    SELECT
        array_agg(wh.has_heartbeat ORDER BY wh.window_idx) as history,
        sum(CASE WHEN wh.has_heartbeat THEN 1 ELSE 0 END) as online_count
    INTO v_heartbeat_history, v_online_windows
    FROM window_heartbeats wh;

    v_result.heartbeat_history := COALESCE(v_heartbeat_history, '{}');

    -- Calculate uptime percentage
    IF array_length(v_heartbeat_history, 1) > 0 THEN
        v_result.uptime_percentage := round((v_online_windows::numeric / array_length(v_heartbeat_history, 1)::numeric) * 100);
    ELSE
        v_result.uptime_percentage := 0;
    END IF;

    -- Calculate heartbeat intervals using window functions
    WITH ordered_heartbeats AS (
        SELECT
            h.timestamp,
            lag(h.timestamp) OVER (ORDER BY h.timestamp DESC) as prev_timestamp
        FROM public.heartbeats h
        WHERE h.device_id = p_device_id
        AND h.timestamp >= v_analysis_start
        ORDER BY h.timestamp DESC
    ),
    intervals AS (
        SELECT EXTRACT(EPOCH FROM (prev_timestamp - timestamp)) / 60 as interval_minutes
        FROM ordered_heartbeats
        WHERE prev_timestamp IS NOT NULL
    )
    SELECT
        array_agg(interval_minutes) as intervals,
        avg(interval_minutes) as avg_interval
    INTO v_interval_data
    FROM intervals;

    v_result.heartbeat_intervals := COALESCE(v_interval_data.intervals, '{}');
    v_result.average_heartbeat_interval := v_interval_data.avg_interval;

    -- Count total heartbeats efficiently
    SELECT count(*) INTO v_result.total_heartbeats
    FROM public.heartbeats h
    WHERE h.device_id = p_device_id
    AND h.timestamp >= v_analysis_start;

    RETURN v_result;
END;
$$;


ALTER FUNCTION "public"."analyze_device_heartbeats"("p_device_id" bigint, "p_lookback_minutes" integer, "p_window_minutes" integer) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."analyze_device_heartbeats"("p_device_id" bigint, "p_lookback_minutes" integer, "p_window_minutes" integer) IS 'SECURE: Analyzes device heartbeat patterns with proper access control.
- Verifies user has view role for device herd OR is the device itself OR is postgres user
- Uses secure search_path and input validation
- Prevents unauthorized access to heartbeat data';



CREATE OR REPLACE FUNCTION "public"."analyze_herd_device_heartbeats"("p_herd_id" bigint, "p_device_types" "public"."device_type"[] DEFAULT ARRAY['radio_mesh_base_station'::"public"."device_type", 'radio_mesh_base_station_gateway'::"public"."device_type"], "p_lookback_minutes" integer DEFAULT 60, "p_window_minutes" integer DEFAULT 2) RETURNS SETOF "public"."device_heartbeat_analysis"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO ''
    AS $$
DECLARE
    v_device_id bigint;
    v_herd_exists boolean;
BEGIN
    -- SECURITY CHECK: Verify herd exists
    SELECT EXISTS(SELECT 1 FROM public.herds WHERE id = p_herd_id) INTO v_herd_exists;

    IF NOT v_herd_exists THEN
        RAISE EXCEPTION 'Herd % not found', p_herd_id;
    END IF;

    -- SECURITY CHECK: Verify user has view access to this herd OR is postgres user
    IF NOT (
        private.has_good_view_role((SELECT auth.uid()), p_herd_id)
        OR
        current_user = 'postgres'
    ) THEN
        RAISE EXCEPTION 'Access denied to herd %', p_herd_id;
    END IF;

    -- Input validation
    IF p_lookback_minutes <= 0 OR p_window_minutes <= 0 THEN
        RAISE EXCEPTION 'Lookback and window minutes must be positive';
    END IF;

    -- Process each device in the herd (security is checked in analyze_device_heartbeats)
    FOR v_device_id IN (
        SELECT d.id
        FROM public.devices d
        WHERE d.herd_id = p_herd_id
        AND (p_device_types IS NULL OR d.device_type = ANY(p_device_types))
        ORDER BY d.id
    ) LOOP
        RETURN NEXT public.analyze_device_heartbeats(
            v_device_id,
            p_lookback_minutes,
            p_window_minutes
        );
    END LOOP;
END;
$$;


ALTER FUNCTION "public"."analyze_herd_device_heartbeats"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."analyze_herd_device_heartbeats"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) IS 'SECURE: Analyzes multiple devices in a herd with access control.
- Verifies user has view role for the herd OR is postgres user
- Validates herd existence before processing
- Input validation prevents abuse
- Leverages individual device security checks';



CREATE OR REPLACE FUNCTION "public"."broadcast_connectivity_changes"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public, extensions, private'
    AS $$
DECLARE
  herd_id_val BIGINT;
  connectivity_record JSONB;
BEGIN
  -- Only handle INSERT operations
  IF TG_OP != 'INSERT' THEN
    RETURN NEW;
  END IF;

  BEGIN
    -- Get herd_id via device_id
    SELECT d.herd_id INTO herd_id_val
    FROM public.devices d
    WHERE d.id = NEW.device_id;

    -- Fallback: try session-based herd lookup if device_id didn't work
    IF herd_id_val IS NULL AND NEW.session_id IS NOT NULL THEN
      SELECT d.herd_id INTO herd_id_val
      FROM public.devices d
      JOIN public.sessions s ON s.device_id = d.id
      WHERE s.id = NEW.session_id;
    END IF;

    -- Build the complete connectivity record with coordinates
    IF herd_id_val IS NOT NULL THEN
      connectivity_record := jsonb_build_object(
        'id', NEW.id,
        'session_id', NEW.session_id,
        'device_id', NEW.device_id,
        'inserted_at', NEW.inserted_at,
        'timestamp_start', NEW.timestamp_start,
        'signal', NEW.signal,
        'noise', NEW.noise,
        'altitude', NEW.altitude,
        'heading', NEW.heading,
        'latitude', CASE
          WHEN NEW.location IS NOT NULL
          THEN extensions.ST_Y(NEW.location::extensions.geometry)
          ELSE NULL
        END,
        'longitude', CASE
          WHEN NEW.location IS NOT NULL
          THEN extensions.ST_X(NEW.location::extensions.geometry)
          ELSE NULL
        END,
        'h14_index', NEW.h14_index,
        'h13_index', NEW.h13_index,
        'h12_index', NEW.h12_index,
        'h11_index', NEW.h11_index,
        'battery_percentage', NEW.battery_percentage,
        'frequency_hz', NEW.frequency_hz,
        'bandwidth_hz', NEW.bandwidth_hz,
        'associated_station', NEW.associated_station,
        'mode', NEW.mode
      );

      -- Send the realtime broadcast with private: TRUE
      PERFORM realtime.send(
        jsonb_build_object(
          'table', 'connectivity',
          'schema', 'public',
          'operation', 'INSERT',
          'record', connectivity_record,
          'old_record', NULL
        ),
        'INSERT',
        herd_id_val::text || '-connectivity',
        TRUE  -- <-- Changed from FALSE to TRUE
      );
    END IF;

  EXCEPTION WHEN OTHERS THEN
    -- Silent error handling - don't break the INSERT
    NULL;
  END;

  RETURN NEW;
END;
$$;


ALTER FUNCTION "public"."broadcast_connectivity_changes"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."broadcast_device_changes"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public, extensions, private'
    AS $$
DECLARE
  device_record JSONB;
  old_device_record JSONB;
BEGIN
  -- Handle INSERT operations
  IF TG_OP = 'INSERT' THEN
    BEGIN
      -- Build the complete device record with coordinates
      device_record := jsonb_build_object(
        'id', NEW.id,
        'inserted_at', NEW.inserted_at,
        'created_by', NEW.created_by,
        'herd_id', NEW.herd_id,
        'device_type', NEW.device_type,
        'domain_name', NEW.domain_name,
        'location', NEW.location::text,
        'altitude', NEW.altitude,
        'heading', NEW.heading,
        'name', NEW.name,
        'description', NEW.description,
        'latitude', CASE
          WHEN NEW.location IS NOT NULL 
          THEN extensions.ST_Y(NEW.location::extensions.geometry)
          ELSE NULL
        END,
        'longitude', CASE
          WHEN NEW.location IS NOT NULL 
          THEN extensions.ST_X(NEW.location::extensions.geometry)
          ELSE NULL
        END
      );

      -- Send the realtime broadcast with private: TRUE
      PERFORM realtime.send(
        jsonb_build_object(
          'table', 'devices',
          'schema', 'public',
          'operation', 'INSERT',
          'record', device_record,
          'old_record', NULL
        ),
        'INSERT',
        NEW.herd_id::text || '-devices',
        TRUE  -- <-- Changed from FALSE to TRUE
      );

    EXCEPTION WHEN OTHERS THEN
      -- Silent error handling - don't break the INSERT
      NULL;
    END;

  -- Handle UPDATE operations
  ELSIF TG_OP = 'UPDATE' THEN
    BEGIN
      -- Build NEW device record
      device_record := jsonb_build_object(
        'id', NEW.id,
        'inserted_at', NEW.inserted_at,
        'created_by', NEW.created_by,
        'herd_id', NEW.herd_id,
        'device_type', NEW.device_type,
        'domain_name', NEW.domain_name,
        'location', NEW.location::text,
        'altitude', NEW.altitude,
        'heading', NEW.heading,
        'name', NEW.name,
        'description', NEW.description,
        'latitude', CASE
          WHEN NEW.location IS NOT NULL 
          THEN extensions.ST_Y(NEW.location::extensions.geometry)
          ELSE NULL
        END,
        'longitude', CASE
          WHEN NEW.location IS NOT NULL 
          THEN extensions.ST_X(NEW.location::extensions.geometry)
          ELSE NULL
        END
      );

      -- Build OLD device record (minimal for comparison)
      old_device_record := jsonb_build_object(
        'id', OLD.id,
        'name', OLD.name,
        'herd_id', OLD.herd_id,
        'latitude', CASE
          WHEN OLD.location IS NOT NULL 
          THEN extensions.ST_Y(OLD.location::extensions.geometry)
          ELSE NULL
        END,
        'longitude', CASE
          WHEN OLD.location IS NOT NULL 
          THEN extensions.ST_X(OLD.location::extensions.geometry)
          ELSE NULL
        END
      );

      -- Send UPDATE broadcast with private: TRUE
      PERFORM realtime.send(
        jsonb_build_object(
          'table', 'devices',
          'schema', 'public',
          'operation', 'UPDATE',
          'record', device_record,
          'old_record', old_device_record
        ),
        'UPDATE',
        COALESCE(NEW.herd_id, OLD.herd_id)::text || '-devices',
        TRUE  -- <-- Changed from FALSE to TRUE
      );

    EXCEPTION WHEN OTHERS THEN
      -- Silent error handling
      NULL;
    END;

  -- Handle DELETE operations
  ELSIF TG_OP = 'DELETE' THEN
    BEGIN
      old_device_record := jsonb_build_object(
        'id', OLD.id,
        'name', OLD.name,
        'herd_id', OLD.herd_id
      );

      PERFORM realtime.send(
        jsonb_build_object(
          'table', 'devices',
          'schema', 'public',
          'operation', 'DELETE',
          'record', NULL,
          'old_record', old_device_record
        ),
        'DELETE',
        OLD.herd_id::text || '-devices',
        TRUE  -- <-- Changed from FALSE to TRUE
      );

    EXCEPTION WHEN OTHERS THEN
      -- Silent error handling
      NULL;
    END;
  END IF;

  RETURN CASE TG_OP
    WHEN 'DELETE' THEN OLD
    ELSE NEW
  END;
END;
$$;


ALTER FUNCTION "public"."broadcast_device_changes"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."broadcast_events_changes"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
    herd_id_val BIGINT;
    event_record JSONB;
BEGIN
    -- Get herd_id via device_id for all operations
    IF TG_OP = 'DELETE' THEN
        SELECT d.herd_id INTO herd_id_val
        FROM public.devices d
        WHERE d.id = OLD.device_id;
    ELSE
        SELECT d.herd_id INTO herd_id_val
        FROM public.devices d
        WHERE d.id = NEW.device_id;
    END IF;

    -- Only proceed if we found a valid herd_id
    IF herd_id_val IS NULL THEN
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        ELSE
            RETURN NEW;
        END IF;
    END IF;

    -- Build the payload based on operation type
    IF TG_OP = 'INSERT' THEN
        -- Extract location data for INSERT
        BEGIN
            -- Build event record with formatted location data
            event_record := jsonb_build_object(
                'id', NEW.id,
                'inserted_at', NEW.inserted_at,
                'message', NEW.message,
                'media_url', NEW.media_url,
                'file_path', NEW.file_path,
                'latitude', CASE
                    WHEN NEW.location IS NOT NULL
                    THEN extensions.ST_Y(NEW.location::extensions.geometry)
                    ELSE NULL
                END,
                'longitude', CASE
                    WHEN NEW.location IS NOT NULL
                    THEN extensions.ST_X(NEW.location::extensions.geometry)
                    ELSE NULL
                END,
                'earthranger_url', NEW.earthranger_url,
                'altitude', NEW.altitude,
                'heading', NEW.heading,
                'media_type', NEW.media_type,
                'device_id', NEW.device_id,
                'timestamp_observation', NEW.timestamp_observation,
                'is_public', NEW.is_public,
                'session_id', NEW.session_id,
                'herd_id', herd_id_val,
                'tags', '[]'::jsonb  -- Tags will be populated separately by tag broadcasts
            );
        EXCEPTION WHEN OTHERS THEN
            -- Fallback to basic record if coordinate extraction fails
            event_record := row_to_json(NEW)::jsonb || jsonb_build_object('herd_id', herd_id_val);
        END;

        -- Send INSERT broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'events',
                'schema', 'public',
                'operation', 'INSERT',
                'record', event_record,
                'old_record', NULL
            ),
            'INSERT',
            herd_id_val::text || '-events',
            TRUE
        );

    ELSIF TG_OP = 'UPDATE' THEN
        -- Extract location data for UPDATE (NEW record)
        BEGIN
            event_record := jsonb_build_object(
                'id', NEW.id,
                'inserted_at', NEW.inserted_at,
                'message', NEW.message,
                'media_url', NEW.media_url,
                'file_path', NEW.file_path,
                'latitude', CASE
                    WHEN NEW.location IS NOT NULL
                    THEN extensions.ST_Y(NEW.location::extensions.geometry)
                    ELSE NULL
                END,
                'longitude', CASE
                    WHEN NEW.location IS NOT NULL
                    THEN extensions.ST_X(NEW.location::extensions.geometry)
                    ELSE NULL
                END,
                'earthranger_url', NEW.earthranger_url,
                'altitude', NEW.altitude,
                'heading', NEW.heading,
                'media_type', NEW.media_type,
                'device_id', NEW.device_id,
                'timestamp_observation', NEW.timestamp_observation,
                'is_public', NEW.is_public,
                'session_id', NEW.session_id,
                'herd_id', herd_id_val,
                'tags', '[]'::jsonb  -- Tags will be populated separately by tag broadcasts
            );
        EXCEPTION WHEN OTHERS THEN
            event_record := row_to_json(NEW)::jsonb || jsonb_build_object('herd_id', herd_id_val);
        END;

        -- Send UPDATE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'events',
                'schema', 'public',
                'operation', 'UPDATE',
                'record', event_record,
                'old_record', row_to_json(OLD)::jsonb
            ),
            'UPDATE',
            herd_id_val::text || '-events',
            TRUE
        );

    ELSIF TG_OP = 'DELETE' THEN
        -- For DELETE, use OLD record data
        BEGIN
            event_record := jsonb_build_object(
                'id', OLD.id,
                'inserted_at', OLD.inserted_at,
                'message', OLD.message,
                'media_url', OLD.media_url,
                'file_path', OLD.file_path,
                'latitude', CASE
                    WHEN OLD.location IS NOT NULL
                    THEN extensions.ST_Y(OLD.location::extensions.geometry)
                    ELSE NULL
                END,
                'longitude', CASE
                    WHEN OLD.location IS NOT NULL
                    THEN extensions.ST_X(OLD.location::extensions.geometry)
                    ELSE NULL
                END,
                'earthranger_url', OLD.earthranger_url,
                'altitude', OLD.altitude,
                'heading', OLD.heading,
                'media_type', OLD.media_type,
                'device_id', OLD.device_id,
                'timestamp_observation', OLD.timestamp_observation,
                'is_public', OLD.is_public,
                'session_id', OLD.session_id,
                'herd_id', herd_id_val,
                'tags', '[]'::jsonb
            );
        EXCEPTION WHEN OTHERS THEN
            event_record := row_to_json(OLD)::jsonb || jsonb_build_object('herd_id', herd_id_val);
        END;

        -- Send DELETE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'events',
                'schema', 'public',
                'operation', 'DELETE',
                'record', NULL,
                'old_record', event_record
            ),
            'DELETE',
            herd_id_val::text || '-events',
            TRUE
        );
    END IF;

    -- Return the appropriate record
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$;


ALTER FUNCTION "public"."broadcast_events_changes"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."broadcast_events_changes"() IS 'Broadcasts changes to the events table via Supabase realtime. Triggers on INSERT, UPDATE, and DELETE operations. Formats location data as latitude/longitude coordinates. Tags are managed separately by tag broadcasts.';



CREATE OR REPLACE FUNCTION "public"."broadcast_pins_changes"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
    herd_id_val BIGINT;
    pin_record JSONB;
BEGIN
    -- Get herd_id for all operations
    IF TG_OP = 'DELETE' THEN
        herd_id_val := OLD.herd_id;
    ELSE
        herd_id_val := NEW.herd_id;
    END IF;

    -- Only proceed if we found a valid herd_id
    IF herd_id_val IS NULL THEN
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        ELSE
            RETURN NEW;
        END IF;
    END IF;

    -- Build the payload based on operation type
    IF TG_OP = 'INSERT' THEN
        -- Extract location data for INSERT
        BEGIN
            -- Get the formatted pin data with coordinates
            SELECT to_json(pin_with_coords) INTO pin_record
            FROM (
                SELECT
                    NEW.id,
                    NEW.created_at,
                    NEW.location,
                    NEW.altitude_relative_to_ground,
                    NEW.color,
                    NEW.name,
                    NEW.description,
                    NEW.herd_id,
                    NEW.created_by,
                    -- Format location data as coordinate values if location exists
                    CASE
                        WHEN NEW.location IS NOT NULL THEN
                            extensions.ST_Y(NEW.location::extensions.geometry)
                        ELSE NULL
                    END as latitude,
                    CASE
                        WHEN NEW.location IS NOT NULL THEN
                            extensions.ST_X(NEW.location::extensions.geometry)
                        ELSE NULL
                    END as longitude
            ) pin_with_coords;
        EXCEPTION WHEN OTHERS THEN
            -- Fallback to basic record if coordinate extraction fails
            pin_record := row_to_json(NEW);
        END;

        -- Send INSERT broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'pins',
                'schema', 'public',
                'operation', 'INSERT',
                'record', pin_record,
                'old_record', NULL
            ),
            'INSERT',
            herd_id_val::text || '-pins',
            TRUE
        );

    ELSIF TG_OP = 'UPDATE' THEN
        -- Extract location data for UPDATE (NEW record)
        BEGIN
            SELECT to_json(pin_with_coords) INTO pin_record
            FROM (
                SELECT
                    NEW.id,
                    NEW.created_at,
                    NEW.location,
                    NEW.altitude_relative_to_ground,
                    NEW.color,
                    NEW.name,
                    NEW.description,
                    NEW.herd_id,
                    NEW.created_by,
                    -- Format location data as coordinate values if location exists
                    CASE
                        WHEN NEW.location IS NOT NULL THEN
                            extensions.ST_Y(NEW.location::extensions.geometry)
                        ELSE NULL
                    END as latitude,
                    CASE
                        WHEN NEW.location IS NOT NULL THEN
                            extensions.ST_X(NEW.location::extensions.geometry)
                        ELSE NULL
                    END as longitude
            ) pin_with_coords;
        EXCEPTION WHEN OTHERS THEN
            pin_record := row_to_json(NEW);
        END;

        -- Send UPDATE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'pins',
                'schema', 'public',
                'operation', 'UPDATE',
                'record', pin_record,
                'old_record', row_to_json(OLD)
            ),
            'UPDATE',
            herd_id_val::text || '-pins',
            TRUE
        );

    ELSIF TG_OP = 'DELETE' THEN
        -- For DELETE, use OLD record data
        BEGIN
            SELECT to_json(pin_with_coords) INTO pin_record
            FROM (
                SELECT
                    OLD.id,
                    OLD.created_at,
                    OLD.location,
                    OLD.altitude_relative_to_ground,
                    OLD.color,
                    OLD.name,
                    OLD.description,
                    OLD.herd_id,
                    OLD.created_by,
                    -- Format location data as coordinate values if location exists
                    CASE
                        WHEN OLD.location IS NOT NULL THEN
                            extensions.ST_Y(OLD.location::extensions.geometry)
                        ELSE NULL
                    END as latitude,
                    CASE
                        WHEN OLD.location IS NOT NULL THEN
                            extensions.ST_X(OLD.location::extensions.geometry)
                        ELSE NULL
                    END as longitude
            ) pin_with_coords;
        EXCEPTION WHEN OTHERS THEN
            pin_record := row_to_json(OLD);
        END;

        -- Send DELETE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'pins',
                'schema', 'public',
                'operation', 'DELETE',
                'record', NULL,
                'old_record', pin_record
            ),
            'DELETE',
            herd_id_val::text || '-pins',
            TRUE
        );
    END IF;

    -- Return the appropriate record
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$;


ALTER FUNCTION "public"."broadcast_pins_changes"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."broadcast_pins_changes"() IS 'Broadcasts changes to the pins table via Supabase realtime. Triggers on INSERT, UPDATE, and DELETE operations. Formats location data as coordinate values (latitude, longitude) extracted from PostGIS geography field.';



CREATE OR REPLACE FUNCTION "public"."broadcast_plans_changes"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
    payload JSONB;
BEGIN
    -- Build the payload based on operation type
    IF TG_OP = 'INSERT' THEN
        payload := jsonb_build_object(
            'operation', 'INSERT',
            'table', 'plans',
            'schema', 'public',
            'record', row_to_json(NEW)
        );
    ELSIF TG_OP = 'UPDATE' THEN
        payload := jsonb_build_object(
            'operation', 'UPDATE',
            'table', 'plans',
            'schema', 'public',
            'record', row_to_json(NEW),
            'old_record', row_to_json(OLD)
        );
    ELSIF TG_OP = 'DELETE' THEN
        payload := jsonb_build_object(
            'operation', 'DELETE',
            'table', 'plans',
            'schema', 'public',
            'old_record', row_to_json(OLD)
        );
    END IF;

    -- Broadcast using pg_notify for Supabase realtime
    PERFORM pg_notify('plans_changes', payload::text);

    -- Return the appropriate record
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$;


ALTER FUNCTION "public"."broadcast_plans_changes"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."broadcast_plans_changes"() IS 'Broadcasts changes to the plans table via Supabase realtime. Triggers on INSERT, UPDATE, and DELETE operations.';



CREATE OR REPLACE FUNCTION "public"."broadcast_sessions_changes"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
    herd_id_val BIGINT;
    session_record JSONB;
    locations_geojson_data JSON;
BEGIN
    -- Get herd_id via device_id for all operations
    IF TG_OP = 'DELETE' THEN
        SELECT d.herd_id INTO herd_id_val
        FROM public.devices d
        WHERE d.id = OLD.device_id;
    ELSE
        SELECT d.herd_id INTO herd_id_val
        FROM public.devices d
        WHERE d.id = NEW.device_id;
    END IF;

    -- Only proceed if we found a valid herd_id
    IF herd_id_val IS NULL THEN
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        ELSE
            RETURN NEW;
        END IF;
    END IF;

    -- Build the payload based on operation type
    IF TG_OP = 'INSERT' THEN
        -- Extract location data for INSERT
        BEGIN
            -- Get the formatted locations_geojson from the session_with_coordinates function
            SELECT to_json(swc) INTO session_record
            FROM (
                SELECT
                    NEW.id,
                    NEW.device_id,
                    NEW.timestamp_start,
                    NEW.timestamp_end,
                    NEW.inserted_at,
                    NEW.software_version,
                    -- Format location data as GeoJSON if location exists
                    CASE
                        WHEN NEW.location IS NOT NULL THEN
                            json_build_object(
                                'type', 'Point',
                                'coordinates', json_build_array(
                                    extensions.ST_X(NEW.location::extensions.geometry),
                                    extensions.ST_Y(NEW.location::extensions.geometry)
                                )
                            )
                        ELSE NULL
                    END as locations_geojson,
                    NEW.altitude_max,
                    NEW.altitude_min,
                    NEW.altitude_average,
                    NEW.velocity_max,
                    NEW.velocity_min,
                    NEW.velocity_average,
                    NEW.distance_total,
                    NEW.distance_max_from_start
            ) swc;
        EXCEPTION WHEN OTHERS THEN
            -- Fallback to basic record if coordinate extraction fails
            session_record := row_to_json(NEW);
        END;

        -- Send INSERT broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'sessions',
                'schema', 'public',
                'operation', 'INSERT',
                'record', session_record,
                'old_record', NULL
            ),
            'INSERT',
            herd_id_val::text || '-sessions',
            TRUE
        );

    ELSIF TG_OP = 'UPDATE' THEN
        -- Extract location data for UPDATE (NEW record)
        BEGIN
            SELECT to_json(swc) INTO session_record
            FROM (
                SELECT
                    NEW.id,
                    NEW.device_id,
                    NEW.timestamp_start,
                    NEW.timestamp_end,
                    NEW.inserted_at,
                    NEW.software_version,
                    -- Format location data as GeoJSON if location exists
                    CASE
                        WHEN NEW.location IS NOT NULL THEN
                            json_build_object(
                                'type', 'Point',
                                'coordinates', json_build_array(
                                    extensions.ST_X(NEW.location::extensions.geometry),
                                    extensions.ST_Y(NEW.location::extensions.geometry)
                                )
                            )
                        ELSE NULL
                    END as locations_geojson,
                    NEW.altitude_max,
                    NEW.altitude_min,
                    NEW.altitude_average,
                    NEW.velocity_max,
                    NEW.velocity_min,
                    NEW.velocity_average,
                    NEW.distance_total,
                    NEW.distance_max_from_start
            ) swc;
        EXCEPTION WHEN OTHERS THEN
            session_record := row_to_json(NEW);
        END;

        -- Send UPDATE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'sessions',
                'schema', 'public',
                'operation', 'UPDATE',
                'record', session_record,
                'old_record', row_to_json(OLD)
            ),
            'UPDATE',
            herd_id_val::text || '-sessions',
            TRUE
        );

    ELSIF TG_OP = 'DELETE' THEN
        -- For DELETE, use OLD record data
        BEGIN
            SELECT to_json(swc) INTO session_record
            FROM (
                SELECT
                    OLD.id,
                    OLD.device_id,
                    OLD.timestamp_start,
                    OLD.timestamp_end,
                    OLD.inserted_at,
                    OLD.software_version,
                    -- Format location data as GeoJSON if location exists
                    CASE
                        WHEN OLD.location IS NOT NULL THEN
                            json_build_object(
                                'type', 'Point',
                                'coordinates', json_build_array(
                                    extensions.ST_X(OLD.location::extensions.geometry),
                                    extensions.ST_Y(OLD.location::extensions.geometry)
                                )
                            )
                        ELSE NULL
                    END as locations_geojson,
                    OLD.altitude_max,
                    OLD.altitude_min,
                    OLD.altitude_average,
                    OLD.velocity_max,
                    OLD.velocity_min,
                    OLD.velocity_average,
                    OLD.distance_total,
                    OLD.distance_max_from_start
            ) swc;
        EXCEPTION WHEN OTHERS THEN
            session_record := row_to_json(OLD);
        END;

        -- Send DELETE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'sessions',
                'schema', 'public',
                'operation', 'DELETE',
                'record', NULL,
                'old_record', session_record
            ),
            'DELETE',
            herd_id_val::text || '-sessions',
            TRUE
        );
    END IF;

    -- Return the appropriate record
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$;


ALTER FUNCTION "public"."broadcast_sessions_changes"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."broadcast_sessions_changes"() IS 'Broadcasts changes to the sessions table via Supabase realtime. Triggers on INSERT, UPDATE, and DELETE operations. Formats location data as GeoJSON coordinates.';



CREATE OR REPLACE FUNCTION "public"."broadcast_tags_changes"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
    herd_id_val BIGINT;
    tag_record JSONB;
BEGIN
    -- Get herd_id via event_id -> device_id for all operations
    IF TG_OP = 'DELETE' THEN
        SELECT d.herd_id INTO herd_id_val
        FROM public.devices d
        JOIN public.events e ON e.device_id = d.id
        WHERE e.id = OLD.event_id;
    ELSE
        SELECT d.herd_id INTO herd_id_val
        FROM public.devices d
        JOIN public.events e ON e.device_id = d.id
        WHERE e.id = NEW.event_id;
    END IF;

    -- Only proceed if we found a valid herd_id
    IF herd_id_val IS NULL THEN
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        ELSE
            RETURN NEW;
        END IF;
    END IF;

    -- Build the payload based on operation type
    IF TG_OP = 'INSERT' THEN
        -- Extract location data for INSERT
        BEGIN
            -- Build tag record with formatted location data (tags_pretty_location format)
            tag_record := jsonb_build_object(
                'id', NEW.id,
                'inserted_at', NEW.inserted_at,
                'x', NEW.x,
                'y', NEW.y,
                'width', NEW.width,
                'height', NEW.height,
                'conf', NEW.conf,
                'observation_type', NEW.observation_type,
                'event_id', NEW.event_id,
                'class_name', NEW.class_name,
                'latitude', CASE
                    WHEN NEW.location IS NOT NULL
                    THEN extensions.ST_Y(NEW.location::extensions.geometry)
                    ELSE NULL
                END,
                'longitude', CASE
                    WHEN NEW.location IS NOT NULL
                    THEN extensions.ST_X(NEW.location::extensions.geometry)
                    ELSE NULL
                END,
                'herd_id', herd_id_val
            );
        EXCEPTION WHEN OTHERS THEN
            -- Fallback to basic record if coordinate extraction fails
            tag_record := row_to_json(NEW)::jsonb || jsonb_build_object('herd_id', herd_id_val);
        END;

        -- Send INSERT broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'tags',
                'schema', 'public',
                'operation', 'INSERT',
                'record', tag_record,
                'old_record', NULL
            ),
            'INSERT',
            herd_id_val::text || '-tags',
            TRUE
        );

    ELSIF TG_OP = 'UPDATE' THEN
        -- Extract location data for UPDATE (NEW record)
        BEGIN
            tag_record := jsonb_build_object(
                'id', NEW.id,
                'inserted_at', NEW.inserted_at,
                'x', NEW.x,
                'y', NEW.y,
                'width', NEW.width,
                'height', NEW.height,
                'conf', NEW.conf,
                'observation_type', NEW.observation_type,
                'event_id', NEW.event_id,
                'class_name', NEW.class_name,
                'latitude', CASE
                    WHEN NEW.location IS NOT NULL
                    THEN extensions.ST_Y(NEW.location::extensions.geometry)
                    ELSE NULL
                END,
                'longitude', CASE
                    WHEN NEW.location IS NOT NULL
                    THEN extensions.ST_X(NEW.location::extensions.geometry)
                    ELSE NULL
                END,
                'herd_id', herd_id_val
            );
        EXCEPTION WHEN OTHERS THEN
            tag_record := row_to_json(NEW)::jsonb || jsonb_build_object('herd_id', herd_id_val);
        END;

        -- Send UPDATE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'tags',
                'schema', 'public',
                'operation', 'UPDATE',
                'record', tag_record,
                'old_record', row_to_json(OLD)::jsonb
            ),
            'UPDATE',
            herd_id_val::text || '-tags',
            TRUE
        );

    ELSIF TG_OP = 'DELETE' THEN
        -- For DELETE, use OLD record data
        BEGIN
            tag_record := jsonb_build_object(
                'id', OLD.id,
                'inserted_at', OLD.inserted_at,
                'x', OLD.x,
                'y', OLD.y,
                'width', OLD.width,
                'height', OLD.height,
                'conf', OLD.conf,
                'observation_type', OLD.observation_type,
                'event_id', OLD.event_id,
                'class_name', OLD.class_name,
                'latitude', CASE
                    WHEN OLD.location IS NOT NULL
                    THEN extensions.ST_Y(OLD.location::extensions.geometry)
                    ELSE NULL
                END,
                'longitude', CASE
                    WHEN OLD.location IS NOT NULL
                    THEN extensions.ST_X(OLD.location::extensions.geometry)
                    ELSE NULL
                END,
                'herd_id', herd_id_val
            );
        EXCEPTION WHEN OTHERS THEN
            tag_record := row_to_json(OLD)::jsonb || jsonb_build_object('herd_id', herd_id_val);
        END;

        -- Send DELETE broadcast
        PERFORM realtime.send(
            jsonb_build_object(
                'table', 'tags',
                'schema', 'public',
                'operation', 'DELETE',
                'record', NULL,
                'old_record', tag_record
            ),
            'DELETE',
            herd_id_val::text || '-tags',
            TRUE
        );
    END IF;

    -- Return the appropriate record
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$;


ALTER FUNCTION "public"."broadcast_tags_changes"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."broadcast_tags_changes"() IS 'Broadcasts changes to the tags table via Supabase realtime. Triggers on INSERT, UPDATE, and DELETE operations. Formats location data as latitude/longitude coordinates and includes bounding box data (x, y, width, height).';



CREATE OR REPLACE FUNCTION "public"."broadcast_versions_software_changes"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
    payload JSONB;
BEGIN
    -- Build the payload based on operation type
    IF TG_OP = 'INSERT' THEN
        payload := jsonb_build_object(
            'operation', 'INSERT',
            'table', 'versions_software',
            'schema', 'public',
            'record', row_to_json(NEW)
        );
    ELSIF TG_OP = 'UPDATE' THEN
        payload := jsonb_build_object(
            'operation', 'UPDATE',
            'table', 'versions_software',
            'schema', 'public',
            'record', row_to_json(NEW),
            'old_record', row_to_json(OLD)
        );
    ELSIF TG_OP = 'DELETE' THEN
        payload := jsonb_build_object(
            'operation', 'DELETE',
            'table', 'versions_software',
            'schema', 'public',
            'old_record', row_to_json(OLD)
        );
    END IF;

    -- Broadcast using pg_notify for Supabase realtime
    PERFORM pg_notify('versions_software_changes', payload::text);

    -- Return the appropriate record
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$;


ALTER FUNCTION "public"."broadcast_versions_software_changes"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."broadcast_versions_software_changes"() IS 'Broadcasts changes to the versions_software table via Supabase realtime. Triggers on INSERT, UPDATE, and DELETE operations.';



CREATE OR REPLACE FUNCTION "public"."check_realtime_schema_status"() RETURNS TABLE("check_type" "text", "schema_name" "text", "table_name" "text", "status" "text", "details" "text")
    LANGUAGE "plpgsql"
    AS $$
BEGIN
  -- Check if realtime schema exists
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'realtime') THEN
    RETURN QUERY SELECT 'schema'::text, 'realtime'::text, ''::text, 'exists'::text, 'Realtime schema found'::text;
    
    -- Check if messages table exists
    IF EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'realtime' AND c.relname = 'messages') THEN
      RETURN QUERY SELECT 'table'::text, 'realtime'::text, 'messages'::text, 'exists'::text, 'Messages table found'::text;
      
      -- Check RLS status
      RETURN QUERY 
      SELECT 
        'rls'::text,
        'realtime'::text, 
        'messages'::text,
        CASE WHEN c.relrowsecurity THEN 'enabled' ELSE 'disabled' END::text,
        CASE 
          WHEN c.relrowsecurity AND c.relforcerowsecurity THEN 'RLS enabled and forced'
          WHEN c.relrowsecurity THEN 'RLS enabled'
          ELSE 'RLS disabled'
        END::text
      FROM pg_class c
      JOIN pg_namespace n ON n.oid = c.relnamespace
      WHERE n.nspname = 'realtime' AND c.relname = 'messages';
      
      -- Check policies
      RETURN QUERY
      SELECT 
        'policy'::text,
        'realtime'::text,
        'messages'::text,
        COALESCE(pol.policyname, 'none')::text,
        COALESCE('Command: ' || pol.cmd || ', Roles: ' || array_to_string(pol.roles, ','), 'No policies found')::text
      FROM pg_policies pol
      WHERE pol.schemaname = 'realtime' AND pol.tablename = 'messages'
      UNION ALL
      SELECT 'policy'::text, 'realtime'::text, 'messages'::text, 'none'::text, 'No policies found'::text
      WHERE NOT EXISTS (SELECT 1 FROM pg_policies WHERE schemaname = 'realtime' AND tablename = 'messages');
      
    ELSE
      RETURN QUERY SELECT 'table'::text, 'realtime'::text, 'messages'::text, 'missing'::text, 'Messages table not found'::text;
    END IF;
  ELSE
    RETURN QUERY SELECT 'schema'::text, 'realtime'::text, ''::text, 'missing'::text, 'Realtime schema not found'::text;
  END IF;
END;
$$;


ALTER FUNCTION "public"."check_realtime_schema_status"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."check_stable_pre_exclusivity"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
BEGIN
    -- Check that both stable and pre are not true simultaneously
    IF NEW.stable = true AND NEW.pre = true THEN
        RAISE EXCEPTION 'A software version cannot be both stable and pre-release. Only one can be true.';
    END IF;

    RETURN NEW;
END;
$$;


ALTER FUNCTION "public"."check_stable_pre_exclusivity"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."check_stable_pre_exclusivity"() IS 'Ensures that a software version cannot be both stable and pre-release simultaneously';



CREATE OR REPLACE FUNCTION "public"."create_user_api_key_secret"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$
declare
  rand_bytes bytea := gen_random_bytes(32);
  user_api_key_secret text := encode(digest(rand_bytes, 'sha512'), 'hex');
begin
  insert into vault.secrets (secret, name) values (user_api_key_secret, new.id);
  return new;
end;
$$;


ALTER FUNCTION "public"."create_user_api_key_secret"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."delete_all_orphaned_sessions"("min_age_seconds" integer DEFAULT 120) RETURNS TABLE("session_id" bigint, "device_id" bigint, "timestamp_start" timestamp with time zone, "age_seconds" integer, "status" "text")
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public'
    AS $$
DECLARE
    session_record RECORD;
    session_age_seconds integer;
    total_deleted integer := 0;
    total_candidates integer := 0;
BEGIN
    -- Loop through all sessions that meet deletion criteria
    FOR session_record IN 
        SELECT s.id, s.device_id, s.timestamp_start
        FROM public.sessions s
        WHERE s.timestamp_end IS NULL
        AND NOT EXISTS (
            SELECT 1 FROM public.connectivity c 
            WHERE c.session_id = s.id
        )
        AND EXTRACT(EPOCH FROM (NOW() - s.timestamp_start)) >= min_age_seconds
        ORDER BY s.id
    LOOP
        total_candidates := total_candidates + 1;
        
        -- Calculate session age for reporting
        SELECT EXTRACT(EPOCH FROM (NOW() - session_record.timestamp_start))::integer
        INTO session_age_seconds;
        
        -- Delete the orphaned session
        DELETE FROM public.sessions
        WHERE id = session_record.id;
        
        total_deleted := total_deleted + 1;
        
        RETURN QUERY SELECT 
            session_record.id,
            session_record.device_id,
            session_record.timestamp_start,
            session_age_seconds,
            'Successfully deleted orphaned session'::text;
    END LOOP;
    
    -- Return summary if no sessions were found
    IF total_candidates = 0 THEN
        RETURN QUERY SELECT 
            NULL::bigint,
            NULL::bigint,
            NULL::timestamp with time zone,
            0::integer,
            FORMAT('No orphaned sessions found (older than %s seconds)', min_age_seconds)::text;
    END IF;
END;
$$;


ALTER FUNCTION "public"."delete_all_orphaned_sessions"("min_age_seconds" integer) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."delete_orphaned_session"("session_id_param" bigint, "min_age_seconds" integer DEFAULT 120) RETURNS TABLE("session_id" bigint, "device_id" bigint, "timestamp_start" timestamp with time zone, "age_seconds" integer, "connectivity_count" bigint, "status" "text")
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public'
    AS $$
DECLARE
    session_record RECORD;
    connectivity_count bigint;
    session_age_seconds integer;
    deleted_count integer;
BEGIN
    -- Get session details
    SELECT s.id, s.device_id, s.timestamp_start, s.timestamp_end
    INTO session_record
    FROM public.sessions s
    WHERE s.id = session_id_param;
    
    -- Check if session exists
    IF NOT FOUND THEN
        RETURN QUERY SELECT 
            session_id_param,
            NULL::bigint,
            NULL::timestamp with time zone,
            0::integer,
            0::bigint,
            'Session not found'::text;
        RETURN;
    END IF;
    
    -- Calculate session age in seconds
    SELECT EXTRACT(EPOCH FROM (NOW() - session_record.timestamp_start))::integer
    INTO session_age_seconds;
    
    -- Count connectivity records for this session
    SELECT COUNT(*)
    INTO connectivity_count
    FROM public.connectivity c
    WHERE c.session_id = session_id_param;
    
    -- Check if session is old enough
    IF session_age_seconds < min_age_seconds THEN
        RETURN QUERY SELECT 
            session_id_param,
            session_record.device_id,
            session_record.timestamp_start,
            session_age_seconds,
            connectivity_count,
            FORMAT('Session too recent (%s seconds old, minimum %s seconds required)', session_age_seconds, min_age_seconds)::text;
        RETURN;
    END IF;
    
    -- Check if session has an end timestamp
    IF session_record.timestamp_end IS NOT NULL THEN
        RETURN QUERY SELECT 
            session_id_param,
            session_record.device_id,
            session_record.timestamp_start,
            session_age_seconds,
            connectivity_count,
            'Session has end timestamp - not deleting'::text;
        RETURN;
    END IF;
    
    -- Check if session has connectivity records
    IF connectivity_count > 0 THEN
        RETURN QUERY SELECT 
            session_id_param,
            session_record.device_id,
            session_record.timestamp_start,
            session_age_seconds,
            connectivity_count,
            'Session has connectivity records - not deleting'::text;
        RETURN;
    END IF;
    
    -- Session meets all criteria for deletion (no connectivity + no end timestamp + old enough)
    DELETE FROM public.sessions
    WHERE id = session_id_param;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    IF deleted_count > 0 THEN
        RETURN QUERY SELECT 
            session_id_param,
            session_record.device_id,
            session_record.timestamp_start,
            session_age_seconds,
            connectivity_count,
            'Successfully deleted orphaned session'::text;
    ELSE
        RETURN QUERY SELECT 
            session_id_param,
            session_record.device_id,
            session_record.timestamp_start,
            session_age_seconds,
            connectivity_count,
            'Failed to delete session'::text;
    END IF;
END;
$$;


ALTER FUNCTION "public"."delete_orphaned_session"("session_id_param" bigint, "min_age_seconds" integer) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."enforce_single_min_per_system"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
    existing_min_record RECORD;
BEGIN
    -- If setting min to true, auto-downgrade any existing min version for the same system
    IF NEW.min = true THEN
        -- Find and downgrade existing min version for this system (excluding the current row if updating)
        -- Use SECURITY DEFINER context to bypass RLS for this administrative operation
        FOR existing_min_record IN
            SELECT id, version, title
            FROM versions_software
            WHERE system = NEW.system
            AND min = true
            AND id != COALESCE(NEW.id, -1)
        LOOP
            -- Auto-downgrade the existing min version
            UPDATE versions_software
            SET min = false,
                updated_at = now()
            WHERE id = existing_min_record.id;

            -- Log the auto-downgrade action
            RAISE NOTICE 'Auto-downgraded min version: ID=%, Version=%, Title=% for system=% to make room for new min version',
                existing_min_record.id,
                existing_min_record.version,
                COALESCE(existing_min_record.title, 'N/A'),
                NEW.system;
        END LOOP;
    END IF;

    RETURN NEW;
END;
$$;


ALTER FUNCTION "public"."enforce_single_min_per_system"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."enforce_single_min_per_system"() IS 'Ensures that only one software version per system can be marked as min by auto-downgrading existing min versions';



CREATE OR REPLACE FUNCTION "public"."enforce_single_stable_per_system"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
DECLARE
    existing_stable_record RECORD;
BEGIN
    -- If setting stable to true, auto-downgrade any existing stable version for the same system
    IF NEW.stable = true THEN
        -- Find and downgrade existing stable version for this system (excluding the current row if updating)
        -- Use SECURITY DEFINER context to bypass RLS for this administrative operation
        FOR existing_stable_record IN
            SELECT id, version, title
            FROM versions_software
            WHERE system = NEW.system
            AND stable = true
            AND id != COALESCE(NEW.id, -1)
        LOOP
            -- Auto-downgrade the existing stable version
            UPDATE versions_software
            SET stable = false,
                updated_at = now()
            WHERE id = existing_stable_record.id;

            -- Log the auto-downgrade action
            RAISE NOTICE 'Auto-downgraded stable version: ID=%, Version=%, Title=% for system=% to make room for new stable version',
                existing_stable_record.id,
                existing_stable_record.version,
                COALESCE(existing_stable_record.title, 'N/A'),
                NEW.system;
        END LOOP;
    END IF;

    RETURN NEW;
END;
$$;


ALTER FUNCTION "public"."enforce_single_stable_per_system"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."enforce_single_stable_per_system"() IS 'Ensures that only one software version per system can be marked as stable by auto-downgrading existing stable versions';



CREATE OR REPLACE FUNCTION "public"."fix_all_sessions_missing_end_timestamps"() RETURNS TABLE("session_id" bigint, "device_id" bigint, "old_timestamp_end" timestamp with time zone, "new_timestamp_end" timestamp with time zone, "status" "text")
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public'
    AS $$
DECLARE
    session_record RECORD;
    last_connectivity_timestamp timestamp with time zone;
    total_fixed integer := 0;
BEGIN
    -- Loop through all sessions with missing end timestamps
    FOR session_record IN 
        SELECT s.id, s.device_id, s.timestamp_start
        FROM public.sessions s
        WHERE s.timestamp_end IS NULL
        ORDER BY s.id
    LOOP
        -- Find the last connectivity timestamp for this session
        SELECT MAX(c.timestamp_start)
        INTO last_connectivity_timestamp
        FROM public.connectivity c
        WHERE c.session_id = session_record.id;
        
        IF last_connectivity_timestamp IS NOT NULL THEN
            -- Update the session with the last connectivity timestamp
            UPDATE public.sessions
            SET timestamp_end = last_connectivity_timestamp
            WHERE id = session_record.id;
            
            total_fixed := total_fixed + 1;
            
            RETURN QUERY SELECT 
                session_record.id,
                session_record.device_id,
                NULL::timestamp with time zone,
                last_connectivity_timestamp,
                'Successfully updated'::text;
        ELSE
            RETURN QUERY SELECT 
                session_record.id,
                session_record.device_id,
                NULL::timestamp with time zone,
                NULL::timestamp with time zone,
                'No connectivity records found'::text;
        END IF;
    END LOOP;
    
    -- Return summary if no individual records were processed
    IF total_fixed = 0 AND NOT FOUND THEN
        RETURN QUERY SELECT 
            NULL::bigint,
            NULL::bigint,
            NULL::timestamp with time zone,
            NULL::timestamp with time zone,
            'No sessions with missing end timestamps found'::text;
    END IF;
END;
$$;


ALTER FUNCTION "public"."fix_all_sessions_missing_end_timestamps"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."fix_session_end_timestamp"("session_id_param" bigint) RETURNS TABLE("session_id" bigint, "old_timestamp_end" timestamp with time zone, "new_timestamp_end" timestamp with time zone, "status" "text")
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public'
    AS $$
DECLARE
    session_record RECORD;
    last_connectivity_timestamp timestamp with time zone;
    updated_count integer;
BEGIN
    -- Get session details
    SELECT s.id, s.timestamp_end, s.timestamp_start, s.device_id
    INTO session_record
    FROM public.sessions s
    WHERE s.id = session_id_param;
    
    -- Check if session exists
    IF NOT FOUND THEN
        RETURN QUERY SELECT 
            session_id_param,
            NULL::timestamp with time zone,
            NULL::timestamp with time zone,
            'Session not found'::text;
        RETURN;
    END IF;
    
    -- Check if session already has an end timestamp
    IF session_record.timestamp_end IS NOT NULL THEN
        RETURN QUERY SELECT 
            session_id_param,
            session_record.timestamp_end,
            session_record.timestamp_end,
            'Session already has end timestamp'::text;
        RETURN;
    END IF;
    
    -- Find the last connectivity timestamp for this session
    SELECT MAX(c.timestamp_start)
    INTO last_connectivity_timestamp
    FROM public.connectivity c
    WHERE c.session_id = session_id_param;
    
    -- If no connectivity records found, return error
    IF last_connectivity_timestamp IS NULL THEN
        RETURN QUERY SELECT 
            session_id_param,
            NULL::timestamp with time zone,
            NULL::timestamp with time zone,
            'No connectivity records found for session'::text;
        RETURN;
    END IF;
    
    -- Update the session with the last connectivity timestamp
    UPDATE public.sessions
    SET timestamp_end = last_connectivity_timestamp
    WHERE id = session_id_param;
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    
    IF updated_count > 0 THEN
        RETURN QUERY SELECT 
            session_id_param,
            NULL::timestamp with time zone,
            last_connectivity_timestamp,
            'Successfully updated end timestamp'::text;
    ELSE
        RETURN QUERY SELECT 
            session_id_param,
            NULL::timestamp with time zone,
            NULL::timestamp with time zone,
            'Failed to update session'::text;
    END IF;
END;
$$;


ALTER FUNCTION "public"."fix_session_end_timestamp"("session_id_param" bigint) OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."artifacts" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "file_path" "text" NOT NULL,
    "session_id" bigint,
    "timestamp_observation" timestamp with time zone,
    "modality" "text",
    "device_id" bigint NOT NULL,
    "updated_at" timestamp with time zone DEFAULT "now"(),
    "timestamp_observation_end" timestamp with time zone DEFAULT "now"() NOT NULL
);


ALTER TABLE "public"."artifacts" OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_artifacts_for_device"("device_id_caller" bigint, "limit_caller" bigint DEFAULT 50, "offset_caller" bigint DEFAULT 0) RETURNS SETOF "public"."artifacts"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM public.artifacts
    WHERE device_id = device_id_caller
    ORDER BY created_at DESC
    LIMIT limit_caller
    OFFSET offset_caller;
END;
$$;


ALTER FUNCTION "public"."get_artifacts_for_device"("device_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_artifacts_for_devices_batch"("device_ids" bigint[], "limit_per_device" bigint DEFAULT 10) RETURNS SETOF "public"."artifacts"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    WITH ranked_artifacts AS (
        SELECT *,
               ROW_NUMBER() OVER (PARTITION BY device_id ORDER BY created_at DESC) as rn
        FROM public.artifacts
        WHERE device_id = ANY(device_ids)
    )
    SELECT id, created_at, file_path, session_id, timestamp_observation, modality, device_id, updated_at
    FROM ranked_artifacts
    WHERE rn <= limit_per_device
    ORDER BY device_id, created_at DESC;
END;
$$;


ALTER FUNCTION "public"."get_artifacts_for_devices_batch"("device_ids" bigint[], "limit_per_device" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_artifacts_for_herd"("herd_id_caller" bigint, "limit_caller" bigint DEFAULT 50, "offset_caller" bigint DEFAULT 0) RETURNS SETOF "public"."artifacts"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    SELECT a.*
    FROM public.artifacts a
    INNER JOIN public.devices d ON a.device_id = d.id
    WHERE d.herd_id = herd_id_caller
    ORDER BY a.created_at DESC
    LIMIT limit_caller
    OFFSET offset_caller;
END;
$$;


ALTER FUNCTION "public"."get_artifacts_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_artifacts_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer DEFAULT 20, "cursor_timestamp" timestamp with time zone DEFAULT NULL::timestamp with time zone, "cursor_id" bigint DEFAULT NULL::bigint) RETURNS SETOF "public"."artifacts"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    SELECT a.*
    FROM artifacts a
    WHERE a.device_id = device_id_caller
      AND (
          cursor_timestamp IS NULL
          OR a.created_at < cursor_timestamp
          OR (a.created_at = cursor_timestamp AND a.id < cursor_id)
      )
    ORDER BY a.created_at DESC, a.id DESC
    LIMIT limit_caller;
END;
$$;


ALTER FUNCTION "public"."get_artifacts_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_artifacts_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) IS 'Cursor-based pagination for artifacts by device with timestamp and id ordering. Uses SECURITY INVOKER to respect RLS policies.';



CREATE OR REPLACE FUNCTION "public"."get_artifacts_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer DEFAULT 20, "cursor_timestamp" timestamp with time zone DEFAULT NULL::timestamp with time zone, "cursor_id" bigint DEFAULT NULL::bigint) RETURNS SETOF "public"."artifacts"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    SELECT a.*
    FROM artifacts a
    INNER JOIN devices d ON a.device_id = d.id
    WHERE d.herd_id = herd_id_caller
      AND (
          cursor_timestamp IS NULL
          OR a.created_at < cursor_timestamp
          OR (a.created_at = cursor_timestamp AND a.id < cursor_id)
      )
    ORDER BY a.created_at DESC, a.id DESC
    LIMIT limit_caller;
END;
$$;


ALTER FUNCTION "public"."get_artifacts_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_artifacts_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) IS 'Cursor-based pagination for artifacts by herd with timestamp and id ordering. Uses SECURITY INVOKER to respect RLS policies.';



CREATE OR REPLACE FUNCTION "public"."get_connectivity_with_coordinates"("session_id_caller" bigint) RETURNS SETOF "public"."connectivity_with_coordinates"
    LANGUAGE "plpgsql"
    AS $$
begin
  return query
  select
    c.id,
    c.session_id,
    c.device_id,
    c.inserted_at,
    c.timestamp_start,
    c.signal,
    c.noise,
    c.altitude,
    c.heading,
    extensions.ST_Y(c.location::extensions.geometry) as latitude,
    extensions.ST_X(c.location::extensions.geometry) as longitude,
    c.h14_index,
    c.h13_index,
    c.h12_index,
    c.h11_index,
    c.battery_percentage,
    c.frequency_hz,
    c.bandwidth_hz,
    c.associated_station,
    c.mode
  from public.connectivity c
  where c.session_id = session_id_caller
  order by c.timestamp_start asc;
end;
$$;


ALTER FUNCTION "public"."get_connectivity_with_coordinates"("session_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_connectivity_with_coordinates_by_device_and_timestamp"("device_id_caller" bigint, "timestamp_filter" timestamp with time zone) RETURNS SETOF "public"."connectivity_with_coordinates"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public'
    AS $$
BEGIN
  RETURN QUERY
  SELECT
    c.id,
    c.session_id,
    c.device_id,
    c.inserted_at,
    c.timestamp_start,
    c.signal,
    c.noise,
    c.altitude,
    c.heading,
    -- Use proper extensions schema prefix for PostGIS functions
    CASE
      WHEN c.location IS NOT NULL THEN extensions.ST_Y(c.location::extensions.geometry)
      ELSE NULL
    END as latitude,
    CASE
      WHEN c.location IS NOT NULL THEN extensions.ST_X(c.location::extensions.geometry)
      ELSE NULL
    END as longitude,
    c.h14_index,
    c.h13_index,
    c.h12_index,
    c.h11_index,
    c.battery_percentage,
    c.frequency_hz,
    c.bandwidth_hz,
    c.associated_station,
    c.mode
  FROM public.connectivity c
  WHERE
    -- Only return data newer than the timestamp filter
    c.timestamp_start > timestamp_filter
    AND (
      -- Case 1: Direct device-based connectivity
      (c.device_id = device_id_caller)
      OR
      -- Case 2: Session-based connectivity where session belongs to the device
      (c.device_id IS NULL AND c.session_id IS NOT NULL AND EXISTS (
        SELECT 1 FROM public.sessions s
        WHERE s.id = c.session_id AND s.device_id = device_id_caller
      ))
      OR
      -- Case 3: Hybrid connectivity where both device_id and session_id are set
      (c.device_id = device_id_caller AND c.session_id IS NOT NULL AND EXISTS (
        SELECT 1 FROM public.sessions s
        WHERE s.id = c.session_id AND s.device_id = device_id_caller
      ))
    )
  ORDER BY c.timestamp_start ASC;
END;
$$;


ALTER FUNCTION "public"."get_connectivity_with_coordinates_by_device_and_timestamp"("device_id_caller" bigint, "timestamp_filter" timestamp with time zone) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_connectivity_with_coordinates_by_device_and_timestamp"("device_id_caller" bigint, "timestamp_filter" timestamp with time zone) IS 'Returns connectivity data with coordinates for a specific device filtered by timestamp. Updated to include mode field.';



CREATE OR REPLACE FUNCTION "public"."get_device_by_api_key"("device_api_key" "text") RETURNS "public"."device_pretty_location"
    LANGUAGE "plpgsql"
    AS $$declare
  device_id_requester bigint;
begin
  SELECT public.get_device_id_from_key(device_api_key) INTO device_id_requester;
  return public.get_device_by_id(device_id_requester);
end;$$;


ALTER FUNCTION "public"."get_device_by_api_key"("device_api_key" "text") OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_device_by_id"("device_id_caller" bigint) RETURNS "public"."device_pretty_location"
    LANGUAGE "plpgsql"
    AS $$
DECLARE
  device_record "public"."device_pretty_location";
BEGIN
  SELECT
    d.id,
    d.inserted_at,
    d.created_by,
    d.herd_id,
    d.device_type,
    d.domain_name,
    d.location::text,
    d.altitude,
    d.heading,
    d.name,
    d.description,
    extensions.ST_Y(d.location::extensions.geometry) AS latitude,
    extensions.ST_X(d.location::extensions.geometry) AS longitude
  INTO device_record
  FROM public.devices d
  WHERE d.id = device_id_caller
  LIMIT 1;

  RETURN device_record;
END;
$$;


ALTER FUNCTION "public"."get_device_by_id"("device_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_device_id_from_key"("device_api_key" "text") RETURNS bigint
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$-- now select decrypted secret from vault.secrets where name='project_api_key_secret';
declare
  project_hash text;
  project_api_key_secret text;
  secret_uuid uuid;
begin
  select decrypted_secret into project_api_key_secret from vault.decrypted_secrets where name='project_api_key_secret';
  project_hash := encode(hmac(device_api_key, project_api_key_secret, 'sha512'), 'hex');
  select id into secret_uuid from vault.secrets where name=project_hash;
  if secret_uuid is not null then 
      return (select private.jwts.device_id from private.jwts where secret_id=secret_uuid);
  end if;
end;$$;


ALTER FUNCTION "public"."get_device_id_from_key"("device_api_key" "text") OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_devices_for_herd"("herd_id_caller" bigint) RETURNS SETOF "public"."device_pretty_location"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
  RETURN QUERY
  SELECT
    d.id,
    d.inserted_at,
    d.created_by,
    d.herd_id,
    d.device_type,
    d.domain_name,
    d.location::text,
    d.altitude,
    d.heading,
    d.name,
    d.description,
    extensions.ST_Y(d.location::extensions.geometry) AS latitude,
    extensions.ST_X(d.location::extensions.geometry) AS longitude
  FROM public.devices d
  WHERE d.herd_id = herd_id_caller
  ORDER BY d.inserted_at DESC;
END;
$$;


ALTER FUNCTION "public"."get_devices_for_herd"("herd_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_events_and_tags_for_device"("device_id_caller" bigint, "limit_caller" bigint) RETURNS SETOF "public"."event_and_tags_pretty_location"
    LANGUAGE "plpgsql"
    AS $$BEGIN
  RETURN QUERY
    WITH device_events AS (
      -- Use device_id index first
      SELECT 
        e.id,
        e.inserted_at,
        e.message,
        e.media_url,
        e.file_path,
        e.location,
        e.earthranger_url,
        e.altitude,
        e.heading,
        e.media_type,
        e.device_id,
        e.timestamp_observation,
        e.is_public
      FROM public.events e
      WHERE e.device_id = device_id_caller
      ORDER BY e.timestamp_observation DESC
      LIMIT limit_caller
    ),
    events_with_devices AS (
      -- Join with devices to get herd_id
      SELECT 
        de.*,
        d.herd_id
      FROM device_events de
      JOIN public.devices d ON de.device_id = d.id
    )
    SELECT
      ewd.id,
      ewd.inserted_at,
      ewd.message,
      ewd.media_url,
      ewd.file_path,
      extensions.ST_Y(ewd.location::extensions.geometry) AS latitude,
      extensions.ST_X(ewd.location::extensions.geometry) AS longitude,
      ewd.earthranger_url,
      ewd.altitude,
      ewd.heading,
      ewd.media_type,
      ewd.device_id,
      ewd.timestamp_observation,
      ewd.is_public,
      COALESCE(
        array_agg(
          ROW(
            t.id,
            t.inserted_at,
            t.x,
            t.y,
            t.width,
            t.conf,
            t.observation_type,
            t.event_id,
            t.class_name,
            t.height,
            t.location,
            extensions.ST_Y(t.location::extensions.geometry),
            extensions.ST_X(t.location::extensions.geometry)
          )::public.tags_pretty_location
        ) FILTER (WHERE t.id IS NOT NULL),
        ARRAY[]::public.tags_pretty_location[]
      ) AS tags,
      ewd.herd_id
    FROM events_with_devices ewd
    LEFT JOIN public.tags t ON ewd.id = t.event_id
    GROUP BY ewd.id, ewd.inserted_at, ewd.message, ewd.media_url, ewd.file_path, ewd.location, ewd.earthranger_url, ewd.altitude, ewd.heading, ewd.media_type, ewd.device_id, ewd.timestamp_observation, ewd.is_public, ewd.herd_id
    ORDER BY ewd.timestamp_observation DESC;
END;$$;


ALTER FUNCTION "public"."get_events_and_tags_for_device"("device_id_caller" bigint, "limit_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_events_and_tags_for_devices_batch"("device_ids" bigint[], "limit_per_device" integer DEFAULT 1) RETURNS SETOF "public"."event_and_tags_pretty_location"
    LANGUAGE "plpgsql"
    AS $$BEGIN
  RETURN QUERY
    WITH device_events AS (
      -- Use device_id index with ANY operator
      SELECT 
        e.id,
        e.inserted_at,
        e.message,
        e.media_url,
        e.file_path,
        e.location,
        e.earthranger_url,
        e.altitude,
        e.heading,
        e.media_type,
        e.device_id,
        e.timestamp_observation,
        e.is_public,
        ROW_NUMBER() OVER (PARTITION BY e.device_id ORDER BY e.timestamp_observation DESC) as rn
      FROM public.events e
      WHERE e.device_id = ANY(device_ids)
    ),
    filtered_events AS (
      -- Apply limit per device
      SELECT * FROM device_events WHERE rn <= limit_per_device
    ),
    events_with_devices AS (
      -- Join with devices to get herd_id
      SELECT 
        fe.*,
        d.herd_id
      FROM filtered_events fe
      JOIN public.devices d ON fe.device_id = d.id
    )
    SELECT
      ewd.id,
      ewd.inserted_at,
      ewd.message,
      ewd.media_url,
      ewd.file_path,
      extensions.ST_Y(ewd.location::extensions.geometry) AS latitude,
      extensions.ST_X(ewd.location::extensions.geometry) AS longitude,
      ewd.earthranger_url,
      ewd.altitude,
      ewd.heading,
      ewd.media_type,
      ewd.device_id,
      ewd.timestamp_observation,
      ewd.is_public,
      COALESCE(
        array_agg(
          ROW(
            t.id,
            t.inserted_at,
            t.x,
            t.y,
            t.width,
            t.conf,
            t.observation_type,
            t.event_id,
            t.class_name,
            t.height,
            t.location,
            extensions.ST_Y(t.location::extensions.geometry),
            extensions.ST_X(t.location::extensions.geometry)
          )::public.tags_pretty_location
        ) FILTER (WHERE t.id IS NOT NULL),
        ARRAY[]::public.tags_pretty_location[]
      ) AS tags,
      ewd.herd_id
    FROM events_with_devices ewd
    LEFT JOIN public.tags t ON ewd.id = t.event_id
    GROUP BY ewd.id, ewd.inserted_at, ewd.message, ewd.media_url, ewd.file_path, ewd.location, ewd.earthranger_url, ewd.altitude, ewd.heading, ewd.media_type, ewd.device_id, ewd.timestamp_observation, ewd.is_public, ewd.herd_id
    ORDER BY ewd.timestamp_observation DESC;
END;$$;


ALTER FUNCTION "public"."get_events_and_tags_for_devices_batch"("device_ids" bigint[], "limit_per_device" integer) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_events_and_tags_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) RETURNS SETOF "public"."event_and_tags_pretty_location"
    LANGUAGE "plpgsql"
    AS $$BEGIN
  RETURN QUERY
    WITH herd_devices AS (
      -- Use the herd_id index on devices first
      SELECT id FROM public.devices WHERE herd_id = herd_id_caller
    ),
    herd_events AS (
      -- Use device_id index and filter for non-session events
      SELECT 
        e.id,
        e.inserted_at,
        e.message,
        e.media_url,
        e.file_path,
        e.location,
        e.earthranger_url,
        e.altitude,
        e.heading,
        e.media_type,
        e.device_id,
        e.timestamp_observation,
        e.is_public
      FROM public.events e
      WHERE e.device_id IN (SELECT id FROM herd_devices)
      AND e.session_id IS NULL
      ORDER BY e.timestamp_observation DESC
      OFFSET offset_caller LIMIT limit_caller
    ),
    events_with_devices AS (
      -- Join back to get herd_id
      SELECT 
        he.*,
        d.herd_id
      FROM herd_events he
      JOIN public.devices d ON he.device_id = d.id
    )
    SELECT
      ewd.id,
      ewd.inserted_at,
      ewd.message,
      ewd.media_url,
      ewd.file_path,
      extensions.ST_Y(ewd.location::extensions.geometry) AS latitude,
      extensions.ST_X(ewd.location::extensions.geometry) AS longitude,
      ewd.earthranger_url,
      ewd.altitude,
      ewd.heading,
      ewd.media_type,
      ewd.device_id,
      ewd.timestamp_observation,
      ewd.is_public,
      COALESCE(
        array_agg(
          ROW(
            t.id,
            t.inserted_at,
            t.x,
            t.y,
            t.width,
            t.conf,
            t.observation_type,
            t.event_id,
            t.class_name,
            t.height,
            t.location,
            extensions.ST_Y(t.location::extensions.geometry),
            extensions.ST_X(t.location::extensions.geometry)
          )::public.tags_pretty_location
        ) FILTER (WHERE t.id IS NOT NULL),
        ARRAY[]::public.tags_pretty_location[]
      ) AS tags,
      ewd.herd_id
    FROM events_with_devices ewd
    LEFT JOIN public.tags t ON ewd.id = t.event_id
    GROUP BY ewd.id, ewd.inserted_at, ewd.message, ewd.media_url, ewd.file_path, ewd.location, ewd.earthranger_url, ewd.altitude, ewd.heading, ewd.media_type, ewd.device_id, ewd.timestamp_observation, ewd.is_public, ewd.herd_id
    ORDER BY ewd.timestamp_observation DESC;
END;$$;


ALTER FUNCTION "public"."get_events_and_tags_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_events_and_tags_for_session"("session_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) RETURNS SETOF "public"."event_and_tags_pretty_location"
    LANGUAGE "plpgsql"
    AS $$BEGIN
  RETURN QUERY
    WITH session_events AS (
      -- Use the session_id index first, then order by timestamp
      SELECT 
        e.id,
        e.inserted_at,
        e.message,
        e.media_url,
        e.file_path,
        e.location,
        e.earthranger_url,
        e.altitude,
        e.heading,
        e.media_type,
        e.device_id,
        e.timestamp_observation,
        e.is_public
      FROM public.events e
      WHERE e.session_id = session_id_caller
      ORDER BY e.timestamp_observation DESC
      OFFSET offset_caller LIMIT limit_caller
    ),
    events_with_devices AS (
      -- Join with devices using the device_id index
      SELECT 
        se.*,
        d.herd_id
      FROM session_events se
      JOIN public.devices d ON se.device_id = d.id
    )
    SELECT 
      ewd.id, 
      ewd.inserted_at, 
      ewd.message, 
      ewd.media_url, 
      ewd.file_path, 
      extensions.ST_Y(ewd.location::extensions.geometry) AS latitude, 
      extensions.ST_X(ewd.location::extensions.geometry) AS longitude, 
      ewd.earthranger_url, 
      ewd.altitude, 
      ewd.heading, 
      ewd.media_type, 
      ewd.device_id, 
      ewd.timestamp_observation, 
      ewd.is_public, 
      COALESCE(
        array_agg(
          ROW(
            t.id,
            t.inserted_at,
            t.x,
            t.y,
            t.width,
            t.conf,
            t.observation_type,
            t.event_id,
            t.class_name,
            t.height,
            t.location,
            extensions.ST_Y(t.location::extensions.geometry),
            extensions.ST_X(t.location::extensions.geometry)
          )::public.tags_pretty_location
        ) FILTER (WHERE t.id IS NOT NULL),
        ARRAY[]::public.tags_pretty_location[]
      ) AS tags, 
      ewd.herd_id
    FROM events_with_devices ewd
    LEFT JOIN public.tags t ON ewd.id = t.event_id
    GROUP BY ewd.id, ewd.inserted_at, ewd.message, ewd.media_url, ewd.file_path, ewd.location, ewd.earthranger_url, ewd.altitude, ewd.heading, ewd.media_type, ewd.device_id, ewd.timestamp_observation, ewd.is_public, ewd.herd_id
    ORDER BY ewd.timestamp_observation DESC;
END;$$;


ALTER FUNCTION "public"."get_events_and_tags_for_session"("session_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_events_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer DEFAULT 20, "cursor_timestamp" timestamp with time zone DEFAULT NULL::timestamp with time zone, "cursor_id" bigint DEFAULT NULL::bigint) RETURNS SETOF "public"."event_and_tags_pretty_location"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    WITH paginated_events AS (
        SELECT
            e.id,
            e.inserted_at,
            e.message,
            e.media_url,
            e.file_path,
            e.location,
            e.earthranger_url,
            e.altitude,
            e.heading,
            e.media_type,
            e.device_id,
            e.timestamp_observation,
            e.is_public
        FROM events e
        WHERE e.device_id = device_id_caller
          AND (
              cursor_timestamp IS NULL
              OR e.timestamp_observation < cursor_timestamp
              OR (e.timestamp_observation = cursor_timestamp AND e.id < cursor_id)
          )
        ORDER BY e.timestamp_observation DESC, e.id DESC
        LIMIT limit_caller
    ),
    events_with_herd AS (
        SELECT
            pe.*,
            d.herd_id
        FROM paginated_events pe
        JOIN devices d ON pe.device_id = d.id
    )
    SELECT
        ewh.id,
        ewh.inserted_at,
        ewh.message,
        ewh.media_url,
        ewh.file_path,
        extensions.ST_Y(ewh.location::extensions.geometry) AS latitude,
        extensions.ST_X(ewh.location::extensions.geometry) AS longitude,
        ewh.earthranger_url,
        ewh.altitude,
        ewh.heading,
        ewh.media_type,
        ewh.device_id,
        ewh.timestamp_observation,
        ewh.is_public,
        COALESCE(
            array_agg(
                ROW(
                    t.id,
                    t.inserted_at,
                    t.x,
                    t.y,
                    t.width,
                    t.conf,
                    t.observation_type,
                    t.event_id,
                    t.class_name,
                    t.height,
                    t.location,
                    extensions.ST_Y(t.location::extensions.geometry),
                    extensions.ST_X(t.location::extensions.geometry)
                )::tags_pretty_location
            ) FILTER (WHERE t.id IS NOT NULL),
            ARRAY[]::tags_pretty_location[]
        ) AS tags,
        ewh.herd_id
    FROM events_with_herd ewh
    LEFT JOIN tags t ON ewh.id = t.event_id
    GROUP BY
        ewh.id, ewh.inserted_at, ewh.message, ewh.media_url, ewh.file_path,
        ewh.location, ewh.earthranger_url, ewh.altitude, ewh.heading,
        ewh.media_type, ewh.device_id, ewh.timestamp_observation,
        ewh.is_public, ewh.herd_id
    ORDER BY ewh.timestamp_observation DESC, ewh.id DESC;
END;
$$;


ALTER FUNCTION "public"."get_events_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_events_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) IS 'Cursor-based pagination for events by device with tags and pretty location formatting. Uses SECURITY INVOKER to respect RLS policies.';



CREATE OR REPLACE FUNCTION "public"."get_events_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer DEFAULT 20, "cursor_timestamp" timestamp with time zone DEFAULT NULL::timestamp with time zone, "cursor_id" bigint DEFAULT NULL::bigint) RETURNS SETOF "public"."event_and_tags_pretty_location"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    WITH paginated_events AS (
        SELECT
            e.id,
            e.inserted_at,
            e.message,
            e.media_url,
            e.file_path,
            e.location,
            e.earthranger_url,
            e.altitude,
            e.heading,
            e.media_type,
            e.device_id,
            e.timestamp_observation,
            e.is_public
        FROM events e
        INNER JOIN devices d ON e.device_id = d.id
        WHERE d.herd_id = herd_id_caller
          AND (
              cursor_timestamp IS NULL
              OR e.timestamp_observation < cursor_timestamp
              OR (e.timestamp_observation = cursor_timestamp AND e.id < cursor_id)
          )
        ORDER BY e.timestamp_observation DESC, e.id DESC
        LIMIT limit_caller
    ),
    events_with_herd AS (
        SELECT
            pe.*,
            d.herd_id
        FROM paginated_events pe
        JOIN devices d ON pe.device_id = d.id
    )
    SELECT
        ewh.id,
        ewh.inserted_at,
        ewh.message,
        ewh.media_url,
        ewh.file_path,
        extensions.ST_Y(ewh.location::extensions.geometry) AS latitude,
        extensions.ST_X(ewh.location::extensions.geometry) AS longitude,
        ewh.earthranger_url,
        ewh.altitude,
        ewh.heading,
        ewh.media_type,
        ewh.device_id,
        ewh.timestamp_observation,
        ewh.is_public,
        COALESCE(
            array_agg(
                ROW(
                    t.id,
                    t.inserted_at,
                    t.x,
                    t.y,
                    t.width,
                    t.conf,
                    t.observation_type,
                    t.event_id,
                    t.class_name,
                    t.height,
                    t.location,
                    extensions.ST_Y(t.location::extensions.geometry),
                    extensions.ST_X(t.location::extensions.geometry)
                )::tags_pretty_location
            ) FILTER (WHERE t.id IS NOT NULL),
            ARRAY[]::tags_pretty_location[]
        ) AS tags,
        ewh.herd_id
    FROM events_with_herd ewh
    LEFT JOIN tags t ON ewh.id = t.event_id
    GROUP BY
        ewh.id, ewh.inserted_at, ewh.message, ewh.media_url, ewh.file_path,
        ewh.location, ewh.earthranger_url, ewh.altitude, ewh.heading,
        ewh.media_type, ewh.device_id, ewh.timestamp_observation,
        ewh.is_public, ewh.herd_id
    ORDER BY ewh.timestamp_observation DESC, ewh.id DESC;
END;
$$;


ALTER FUNCTION "public"."get_events_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_events_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) IS 'Cursor-based pagination for events by herd with tags and pretty location formatting. Uses SECURITY INVOKER to respect RLS policies.';



CREATE OR REPLACE FUNCTION "public"."get_events_with_tags_for_herd"("herd_id_caller" bigint, "offset_caller" bigint, "limit_caller" bigint) RETURNS SETOF "public"."event_with_tags"
    LANGUAGE "plpgsql"
    SET "search_path" TO 'public,extensions'
    AS $$begin
  return query
    select e.id, e.inserted_at, e.message, e.media_url, extensions.st_y(e.location::extensions.geometry) as latitude, extensions.st_x(e.location::extensions.geometry) as longitude, e.altitude, e.heading, e.media_type, e.device_id, e.timestamp_observation, e.is_public, array_agg(t) as tags
    where herd_id = herd_id_caller
    offset offset_caller limit limit_caller;
end;$$;


ALTER FUNCTION "public"."get_events_with_tags_for_herd"("herd_id_caller" bigint, "offset_caller" bigint, "limit_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_herd_uptime_summary"("p_herd_id" bigint, "p_device_types" "public"."device_type"[] DEFAULT ARRAY['radio_mesh_base_station'::"public"."device_type", 'radio_mesh_base_station_gateway'::"public"."device_type"], "p_lookback_minutes" integer DEFAULT 60, "p_window_minutes" integer DEFAULT 2) RETURNS TABLE("total_devices" integer, "online_devices" integer, "offline_devices" integer, "overall_uptime_percentage" integer, "average_heartbeat_interval" numeric, "total_heartbeats" bigint)
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO ''
    AS $$
DECLARE
    v_now timestamp with time zone := now();
    v_analysis_start timestamp with time zone;
    v_summary record;
    v_herd_exists boolean;
BEGIN
    -- SECURITY CHECK: Verify herd exists
    SELECT EXISTS(SELECT 1 FROM public.herds WHERE id = p_herd_id) INTO v_herd_exists;

    IF NOT v_herd_exists THEN
        RAISE EXCEPTION 'Herd % not found', p_herd_id;
    END IF;

    -- SECURITY CHECK: Verify user has view access to this herd OR is postgres user
    IF NOT (
        private.has_good_view_role((SELECT auth.uid()), p_herd_id)
        OR
        current_user = 'postgres'
    ) THEN
        RAISE EXCEPTION 'Access denied to herd %', p_herd_id;
    END IF;

    -- Input validation
    IF p_lookback_minutes <= 0 OR p_window_minutes <= 0 THEN
        RAISE EXCEPTION 'Lookback and window minutes must be positive';
    END IF;

    v_analysis_start := v_now - (p_lookback_minutes || ' minutes')::interval;

    -- Direct calculation for better performance
    WITH herd_devices AS (
        SELECT d.id as device_id
        FROM public.devices d
        WHERE d.herd_id = p_herd_id
        AND (p_device_types IS NULL OR d.device_type = ANY(p_device_types))
    ),
    device_stats AS (
        SELECT
            hd.device_id,
            -- Check if device is online (has heartbeat within 5 minutes)
            EXISTS(
                SELECT 1 FROM public.heartbeats h
                WHERE h.device_id = hd.device_id
                AND h.timestamp >= v_now - '5 minutes'::interval
            ) as is_online,
            -- Count heartbeats in analysis window
            (SELECT count(*) FROM public.heartbeats h
             WHERE h.device_id = hd.device_id
             AND h.timestamp >= v_analysis_start) as heartbeat_count,
            -- Calculate average interval for this device
            (SELECT avg(EXTRACT(EPOCH FROM (lag_ts - timestamp)) / 60)
             FROM (
                SELECT
                    timestamp,
                    lag(timestamp) OVER (ORDER BY timestamp DESC) as lag_ts
                FROM public.heartbeats h2
                WHERE h2.device_id = hd.device_id
                AND h2.timestamp >= v_analysis_start
             ) t WHERE lag_ts IS NOT NULL) as avg_interval
        FROM herd_devices hd
    )
    SELECT
        count(*)::integer as total,
        sum(CASE WHEN is_online THEN 1 ELSE 0 END)::integer as online,
        sum(heartbeat_count) as total_beats,
        avg(avg_interval) as overall_avg_interval
    INTO v_summary
    FROM device_stats;

    total_devices := COALESCE(v_summary.total, 0);
    online_devices := COALESCE(v_summary.online, 0);
    offline_devices := total_devices - online_devices;
    total_heartbeats := COALESCE(v_summary.total_beats, 0);
    average_heartbeat_interval := v_summary.overall_avg_interval;

    -- Calculate overall uptime percentage based on online/offline ratio
    IF total_devices > 0 THEN
        overall_uptime_percentage := round((online_devices::numeric / total_devices::numeric) * 100);
    ELSE
        overall_uptime_percentage := 0;
    END IF;

    RETURN NEXT;
END;
$$;


ALTER FUNCTION "public"."get_herd_uptime_summary"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_herd_uptime_summary"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) IS 'SECURE: Provides herd-level uptime summary with access control.
- Direct calculation for optimal performance
- Full security validation before processing (postgres user bypass)
- Prevents unauthorized herd data access';



CREATE OR REPLACE FUNCTION "public"."get_pins_for_herd"("herd_id_caller" bigint) RETURNS SETOF "public"."pins_pretty_location"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
  RETURN QUERY
  SELECT
    p.id,
    p.created_at,
    p.location,
    p.altitude_relative_to_ground,
    p.color,
    p.name,
    p.description,
    p.herd_id,
    p.created_by,
    extensions.ST_Y(p.location::extensions.geometry) AS latitude,
    extensions.ST_X(p.location::extensions.geometry) AS longitude
  FROM public.pins p
  WHERE p.herd_id = herd_id_caller
  ORDER BY p.created_at DESC;
END;
$$;


ALTER FUNCTION "public"."get_pins_for_herd"("herd_id_caller" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_pins_for_herd"("herd_id_caller" bigint) IS 'Returns all pins for a specific herd with pretty-formatted location coordinates (latitude, longitude) extracted from PostGIS geography field';



CREATE OR REPLACE FUNCTION "public"."get_session_by_id"("session_id_caller" bigint) RETURNS SETOF "public"."session_with_coordinates"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        s.id,
        s.device_id,
        s.timestamp_start,
        s.timestamp_end,
        s.inserted_at,
        s.software_version,
        -- Use ST_AsGeoJSON to handle LINESTRING locations properly
        extensions.ST_AsGeoJSON(s.locations::extensions.geometry)::jsonb as locations_geojson,
        s.altitude_max,
        s.altitude_min,
        s.altitude_average,
        s.velocity_max,
        s.velocity_min,
        s.velocity_average,
        s.distance_total,
        s.distance_max_from_start
    FROM sessions s
    WHERE s.id = session_id_caller;
END;
$$;


ALTER FUNCTION "public"."get_session_by_id"("session_id_caller" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_session_by_id"("session_id_caller" bigint) IS 'Gets a single session by ID with coordinates and GeoJSON formatting. Uses SECURITY INVOKER to respect RLS policies.';



CREATE OR REPLACE FUNCTION "public"."get_session_summaries"("start_date_caller" "date" DEFAULT NULL::"date", "end_date_caller" "date" DEFAULT NULL::"date", "device_id_caller" bigint DEFAULT NULL::bigint, "herd_id_caller" bigint DEFAULT NULL::bigint) RETURNS json
    LANGUAGE "plpgsql"
    AS $$
DECLARE
    summary_result JSON;
BEGIN
    WITH filtered_sessions AS (
        SELECT
            s.*,
            d.herd_id,
            -- Calculate session duration in minutes
            CASE
                WHEN s.timestamp_end IS NOT NULL THEN
                    EXTRACT(EPOCH FROM (s.timestamp_end - s.timestamp_start)) / 60.0
                ELSE 0
            END as duration_minutes,
            -- Determine if session is during day or night (6 AM - 6 PM is day)
            CASE
                WHEN EXTRACT(HOUR FROM s.timestamp_start AT TIME ZONE 'UTC') BETWEEN 6 AND 17 THEN 'day'
                ELSE 'night'
            END as time_period
        FROM sessions s
        INNER JOIN devices d ON s.device_id = d.id
        WHERE
            -- Apply date filters if provided
            (start_date_caller IS NULL OR DATE(s.timestamp_start) >= start_date_caller)
            AND (end_date_caller IS NULL OR DATE(s.timestamp_start) <= end_date_caller)
            -- Apply device filter if provided
            AND (device_id_caller IS NULL OR s.device_id = device_id_caller)
            -- Apply herd filter if provided
            AND (herd_id_caller IS NULL OR d.herd_id = herd_id_caller)
    ),
    session_stats AS (
        SELECT
            -- Total session time in minutes
            COALESCE(SUM(duration_minutes), 0) as total_session_time_minutes,

            -- Day/night session times
            COALESCE(SUM(CASE WHEN time_period = 'day' THEN duration_minutes ELSE 0 END), 0) as total_day_minutes,
            COALESCE(SUM(CASE WHEN time_period = 'night' THEN duration_minutes ELSE 0 END), 0) as total_night_minutes,

            -- Session counts
            COUNT(*) as total_sessions,

            -- First and last session timestamps
            MIN(timestamp_start) as first_session_timestamp,
            MAX(timestamp_start) as last_session_timestamp,

            -- Distance statistics (in meters)
            COALESCE(SUM(distance_total), 0) as total_distance,
            COALESCE(AVG(distance_total), 0) as average_distance
        FROM filtered_sessions
        WHERE duration_minutes > 0 -- Only include sessions with valid duration
    ),
    version_stats AS (
        SELECT
            json_object_agg(
                COALESCE(software_version, 'unknown'),
                ROUND(version_time::numeric, 2)
            ) as session_time_by_version
        FROM (
            SELECT
                software_version,
                SUM(duration_minutes) as version_time
            FROM filtered_sessions
            WHERE duration_minutes > 0
            GROUP BY software_version
        ) version_summary
    )
    SELECT
        json_build_object(
            'total_session_time_minutes', ROUND(ss.total_session_time_minutes::numeric, 2),
            'total_session_time_night_minutes', ROUND(ss.total_night_minutes::numeric, 2),
            'total_session_time_day_minutes', ROUND(ss.total_day_minutes::numeric, 2),
            'total_sessions', ss.total_sessions,
            'first_session_timestamp', ss.first_session_timestamp,
            'last_session_timestamp', ss.last_session_timestamp,
            'total_distance_meters', ROUND(ss.total_distance::numeric, 2),
            'average_distance_meters', ROUND(ss.average_distance::numeric, 2),
            'session_time_by_version', COALESCE(vs.session_time_by_version, '{}'),
            'summary_generated_at', NOW(),
            'filters_applied', json_build_object(
                'start_date', start_date_caller,
                'end_date', end_date_caller,
                'device_id', device_id_caller,
                'herd_id', herd_id_caller
            )
        ) INTO summary_result
    FROM session_stats ss
    CROSS JOIN version_stats vs;

    -- Return the summary, or empty summary if no sessions found
    RETURN COALESCE(
        summary_result,
        json_build_object(
            'total_session_time_minutes', 0,
            'total_session_time_night_minutes', 0,
            'total_session_time_day_minutes', 0,
            'total_sessions', 0,
            'first_session_timestamp', NULL,
            'last_session_timestamp', NULL,
            'total_distance_meters', 0,
            'average_distance_meters', 0,
            'session_time_by_version', '{}',
            'summary_generated_at', NOW(),
            'filters_applied', json_build_object(
                'start_date', start_date_caller,
                'end_date', end_date_caller,
                'device_id', device_id_caller,
                'herd_id', herd_id_caller
            )
        )
    );
END;
$$;


ALTER FUNCTION "public"."get_session_summaries"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_session_summaries"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) IS 'Returns comprehensive session summaries as JSON with optional filtering by date range, device, or herd. Uses SECURITY INVOKER to respect RLS policies. Includes total time, day/night breakdown, distance statistics, and software version analysis.';



CREATE OR REPLACE FUNCTION "public"."get_session_usage_over_time"("start_date_caller" "date" DEFAULT NULL::"date", "end_date_caller" "date" DEFAULT NULL::"date", "device_id_caller" bigint DEFAULT NULL::bigint, "herd_id_caller" bigint DEFAULT NULL::bigint) RETURNS json
    LANGUAGE "plpgsql"
    AS $$
DECLARE
    usage_summary_result JSON;
BEGIN
    WITH filtered_sessions AS (
        SELECT
            s.*,
            d.herd_id,
            -- Calculate flight duration in minutes
            CASE
                WHEN s.timestamp_end IS NOT NULL THEN
                    EXTRACT(EPOCH FROM (s.timestamp_end - s.timestamp_start)) / 60.0
                ELSE 0
            END as flight_duration_minutes,
            -- Extract date for grouping
            DATE(s.timestamp_start) as flight_date,
            -- Extract year-month for monthly aggregation
            DATE_TRUNC('month', s.timestamp_start) as flight_month,
            -- Extract year for yearly aggregation
            DATE_TRUNC('year', s.timestamp_start) as flight_year
        FROM sessions s
        INNER JOIN devices d ON s.device_id = d.id
        WHERE
            -- Apply date filters if provided
            (start_date_caller IS NULL OR DATE(s.timestamp_start) >= start_date_caller)
            AND (end_date_caller IS NULL OR DATE(s.timestamp_start) <= end_date_caller)
            -- Apply device filter if provided
            AND (device_id_caller IS NULL OR s.device_id = device_id_caller)
            -- Apply herd filter if provided
            AND (herd_id_caller IS NULL OR d.herd_id = herd_id_caller)
            -- Only include sessions with valid duration and distance
            AND s.timestamp_end IS NOT NULL
            AND s.distance_total > 0
    ),
    daily_stats AS (
        SELECT
            flight_date,
            COUNT(*) as flights_count,
            ROUND(SUM(flight_duration_minutes)::numeric, 2) as total_flight_time_minutes,
            ROUND(SUM(distance_total)::numeric, 2) as total_distance_meters
        FROM filtered_sessions
        GROUP BY flight_date
        ORDER BY flight_date
    ),
    monthly_stats AS (
        SELECT
            flight_month,
            COUNT(*) as flights_count,
            ROUND(SUM(flight_duration_minutes)::numeric, 2) as total_flight_time_minutes,
            ROUND(SUM(distance_total)::numeric, 2) as total_distance_meters
        FROM filtered_sessions
        GROUP BY flight_month
        ORDER BY flight_month
    ),
    yearly_stats AS (
        SELECT
            flight_year,
            COUNT(*) as flights_count,
            ROUND(SUM(flight_duration_minutes)::numeric, 2) as total_flight_time_minutes,
            ROUND(SUM(distance_total)::numeric, 2) as total_distance_meters
        FROM filtered_sessions
        GROUP BY flight_year
        ORDER BY flight_year
    )
    SELECT
        json_build_object(
            'daily', COALESCE(
                (SELECT json_agg(
                    json_build_object(
                        'date', flight_date,
                        'flights_count', flights_count,
                        'total_flight_time_minutes', total_flight_time_minutes,
                        'total_distance_meters', total_distance_meters
                    )
                ) FROM daily_stats),
                '[]'::json
            ),
            'monthly', COALESCE(
                (SELECT json_agg(
                    json_build_object(
                        'month', flight_month,
                        'flights_count', flights_count,
                        'total_flight_time_minutes', total_flight_time_minutes,
                        'total_distance_meters', total_distance_meters
                    )
                ) FROM monthly_stats),
                '[]'::json
            ),
            'yearly', COALESCE(
                (SELECT json_agg(
                    json_build_object(
                        'year', flight_year,
                        'flights_count', flights_count,
                        'total_flight_time_minutes', total_flight_time_minutes,
                        'total_distance_meters', total_distance_meters
                    )
                ) FROM yearly_stats),
                '[]'::json
            ),
            'summary_generated_at', NOW()::timestamptz
        ) INTO usage_summary_result;

    RETURN usage_summary_result;
END;
$$;


ALTER FUNCTION "public"."get_session_usage_over_time"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_session_usage_over_time"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) IS 'Gets session usage trends including total distance and flight time aggregated by day, month, and year. Focused on time-based trends without overall statistics.';



CREATE OR REPLACE FUNCTION "public"."get_sessions_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer DEFAULT 20, "cursor_timestamp" timestamp with time zone DEFAULT NULL::timestamp with time zone, "cursor_id" bigint DEFAULT NULL::bigint) RETURNS SETOF "public"."session_with_coordinates"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        s.id,
        s.device_id,
        s.timestamp_start,
        s.timestamp_end,
        s.inserted_at,
        s.software_version,
        -- Use ST_AsGeoJSON to handle LINESTRING locations properly
        extensions.ST_AsGeoJSON(s.locations::extensions.geometry)::jsonb as locations_geojson,
        s.altitude_max,
        s.altitude_min,
        s.altitude_average,
        s.velocity_max,
        s.velocity_min,
        s.velocity_average,
        s.distance_total,
        s.distance_max_from_start
    FROM sessions s
    WHERE s.device_id = device_id_caller
      AND (
          cursor_timestamp IS NULL
          OR s.timestamp_start < cursor_timestamp
          OR (s.timestamp_start = cursor_timestamp AND s.id < cursor_id)
      )
    ORDER BY s.timestamp_start DESC, s.id DESC
    LIMIT limit_caller;
END;
$$;


ALTER FUNCTION "public"."get_sessions_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_sessions_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) IS 'Cursor-based pagination for sessions by device with coordinates and GeoJSON formatting. Uses SECURITY INVOKER to respect RLS policies.';



CREATE OR REPLACE FUNCTION "public"."get_sessions_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer DEFAULT 20, "cursor_timestamp" timestamp with time zone DEFAULT NULL::timestamp with time zone, "cursor_id" bigint DEFAULT NULL::bigint) RETURNS SETOF "public"."session_with_coordinates"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    RETURN QUERY
    SELECT
        s.id,
        s.device_id,
        s.timestamp_start,
        s.timestamp_end,
        s.inserted_at,
        s.software_version,
        -- Use ST_AsGeoJSON to handle LINESTRING locations properly
        extensions.ST_AsGeoJSON(s.locations::extensions.geometry)::jsonb as locations_geojson,
        s.altitude_max,
        s.altitude_min,
        s.altitude_average,
        s.velocity_max,
        s.velocity_min,
        s.velocity_average,
        s.distance_total,
        s.distance_max_from_start
    FROM sessions s
    INNER JOIN devices d ON s.device_id = d.id
    WHERE d.herd_id = herd_id_caller
      AND (
          cursor_timestamp IS NULL
          OR s.timestamp_start < cursor_timestamp
          OR (s.timestamp_start = cursor_timestamp AND s.id < cursor_id)
      )
    ORDER BY s.timestamp_start DESC, s.id DESC
    LIMIT limit_caller;
END;
$$;


ALTER FUNCTION "public"."get_sessions_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) OWNER TO "postgres";


COMMENT ON FUNCTION "public"."get_sessions_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) IS 'Cursor-based pagination for sessions by herd with coordinates and GeoJSON formatting. Uses SECURITY INVOKER to respect RLS policies.';



CREATE OR REPLACE FUNCTION "public"."get_sessions_with_coordinates"("herd_id_caller" bigint) RETURNS SETOF "public"."session_with_coordinates"
    LANGUAGE "plpgsql"
    AS $$begin
  return query
  select 
    s.id,
    s.device_id,
    s.timestamp_start,
    s.timestamp_end,
    s.inserted_at,
    s.software_version,
    extensions.ST_AsGeoJSON(s.locations::extensions.geometry)::jsonb as locations_geojson,
    s.altitude_max,
    s.altitude_min,
    s.altitude_average,
    s.velocity_max,
    s.velocity_min,
    s.velocity_average,
    s.distance_total,
    s.distance_max_from_start
  from public.sessions s
  inner join public.devices d on s.device_id = d.id
  where d.herd_id = herd_id_caller
  order by s.timestamp_start desc;
end;$$;


ALTER FUNCTION "public"."get_sessions_with_coordinates"("herd_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_sessions_with_coordinates_by_device"("device_id_caller" bigint) RETURNS SETOF "public"."session_with_coordinates"
    LANGUAGE "plpgsql"
    AS $$begin
  return query
  select 
    s.id,
    s.device_id,
    s.timestamp_start,
    s.timestamp_end,
    s.inserted_at,
    s.software_version,
    extensions.ST_AsGeoJSON(s.locations::extensions.geometry)::jsonb as locations_geojson,
    s.altitude_max,
    s.altitude_min,
    s.altitude_average,
    s.velocity_max,
    s.velocity_min,
    s.velocity_average,
    s.distance_total,
    s.distance_max_from_start
  from public.sessions s
  where s.device_id = device_id_caller
  order by s.timestamp_start desc;
end;$$;


ALTER FUNCTION "public"."get_sessions_with_coordinates_by_device"("device_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_total_artifacts_for_herd"("herd_id_caller" bigint) RETURNS bigint
    LANGUAGE "plpgsql"
    AS $$
DECLARE
    artifact_count bigint;
BEGIN
    SELECT COUNT(*)
    INTO artifact_count
    FROM public.artifacts a
    INNER JOIN public.devices d ON a.device_id = d.id
    WHERE d.herd_id = herd_id_caller;

    RETURN artifact_count;
END;
$$;


ALTER FUNCTION "public"."get_total_artifacts_for_herd"("herd_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_total_events_for_herd_with_session_filter"("herd_id_caller" bigint, "exclude_session_events" boolean) RETURNS bigint
    LANGUAGE "plpgsql"
    AS $$
declare
    total_events bigint;  -- Declare a variable to hold the count
begin
    SELECT COUNT(*)
    INTO total_events  -- Store the result of the SELECT into the variable
    FROM public.events e
    JOIN public.devices d ON e.device_id = d.id
    WHERE d.herd_id = herd_id_caller
      AND (NOT exclude_session_events OR e.session_id IS NULL);

    RETURN total_events;  -- Return the count
END;
$$;


ALTER FUNCTION "public"."get_total_events_for_herd_with_session_filter"("herd_id_caller" bigint, "exclude_session_events" boolean) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_total_events_for_session"("session_id_caller" bigint) RETURNS bigint
    LANGUAGE "plpgsql"
    AS $$
DECLARE
    total_events bigint;
BEGIN
    SELECT COUNT(*)
    INTO total_events
    FROM public.events e
    WHERE e.session_id = session_id_caller;

    RETURN total_events;
END;
$$;


ALTER FUNCTION "public"."get_total_events_for_session"("session_id_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."get_zones_and_actions_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) RETURNS SETOF "public"."zones_and_actions_pretty_location"
    LANGUAGE "plpgsql"
    AS $$begin
  return query
    select z.id, z.inserted_at, extensions.st_astext(z.region) as region, z.herd_id, z.actions
    from public.zones_and_actions z
    join public.actions a on z.id = a.zone_id
    where z.herd_id = herd_id_caller
    order by z.inserted_at desc
    offset offset_caller limit limit_caller;
end;$$;


ALTER FUNCTION "public"."get_zones_and_actions_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."handle_new_user"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO ''
    AS $$
begin
  insert into public.users (id, username) values (new.id, new.email);
  return new;
end;
$$;


ALTER FUNCTION "public"."handle_new_user"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."load_api_keys"("id_of_device" bigint) RETURNS "text"[]
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$declare
  device_api_key_secret text;
  project_api_key_secret text;
  vault_record record;
  keys text[] := '{}';
  current_set jsonb;
  jwt_record record;
begin
    if not exists (select 1 from public.devices where id = id_of_device) then
      raise exception 'Device % does not exist', id_of_device;
    end if;
    select decrypted_secret into device_api_key_secret from vault.decrypted_secrets where name=id_of_device::text;

    for jwt_record IN
      select secret_id
      from private.jwts
      where device_id=id_of_device
    loop
      select decrypted_secret into vault_record from vault.decrypted_secrets where id=jwt_record.secret_id;
      current_set := jsonb_build_object(
        'id', to_jsonb(jwt_record.secret_id),
        'key', to_jsonb(encode(extensions.hmac(vault_record.decrypted_secret, device_api_key_secret, 'sha512'), 'hex'))
      );
      select into keys array_append(keys, current_set::text);
    end loop;
    return keys;
end;$$;


ALTER FUNCTION "public"."load_api_keys"("id_of_device" bigint) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."load_api_keys"("id_of_device" "text") RETURNS "jsonb"[]
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$declare
  device_api_key_secret text;
  project_api_key_secret text;
  vault_record record;
  keys jsonb[] := '{}';
  current_set jsonb;
  jwt_record record;
begin
    if not exists (select 1 from public.devices where id = id_of_device::bigint) then
      raise exception 'Device % does not exist', id_of_device;
    end if;
    select decrypted_secret into device_api_key_secret from vault.decrypted_secrets where name=id_of_device;

    for jwt_record IN
      select secret_id
      from private.jwts
      where device_id=id_of_device::bigint
    loop
      select decrypted_secret into vault_record from vault.decrypted_secrets where id=jwt_record.secret_id;
      current_set := jsonb_build_object(
        'id', to_jsonb(jwt_record.secret_id),
        'key', to_jsonb(encode(extensions.hmac(vault_record.decrypted_secret, device_api_key_secret, 'sha512'), 'hex'))
      );
      select into keys array_append(keys, current_set);
    end loop;
    return keys;
end;$$;


ALTER FUNCTION "public"."load_api_keys"("id_of_device" "text") OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."load_api_keys_batch"("device_ids" bigint[]) RETURNS TABLE("device_id" bigint, "api_key_id" "text", "api_key_key" "text")
    LANGUAGE "plpgsql"
    AS $$
DECLARE
  current_device_id bigint;
  keys text[];
  current_key text;
BEGIN
  -- Loop through each device ID and call the existing load_api_keys function
  FOREACH current_device_id IN ARRAY device_ids
  LOOP
    -- Call the new load_api_keys function for this device (takes bigint parameter)
    keys := public.load_api_keys(current_device_id);
    
    -- Convert the returned keys array to individual rows
    FOREACH current_key IN ARRAY keys
    LOOP
      -- Parse the JSON key and return as a row
      RETURN QUERY
      SELECT 
        current_device_id as device_id,
        current_key::jsonb->>'id' as api_key_id,
        current_key::jsonb->>'key' as api_key_key;
    END LOOP;
  END LOOP;
END;
$$;


ALTER FUNCTION "public"."load_api_keys_batch"("device_ids" bigint[]) OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."load_api_keys_old"("id_of_device" "text") RETURNS "text"[]
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'extensions'
    AS $$declare
  current_set jsonb;
  jwt_record record;
  keys jsonb[];
  device_api_key_secret text;
  vault_record record;
  herd_id_of_device bigint;
begin
    -- find herd id from device id
    select into herd_id_of_device herd_id from public.devices where id=id_of_device::bigint;
    -- now check if user has appropriate role to view keys
    if not private.has_good_edit_role(auth.uid(), herd_id_of_device) then
      return keys;
    end if;
    select decrypted_secret into device_api_key_secret from vault.decrypted_secrets where name=id_of_device;

    for jwt_record IN 
      select secret_id
      from private.jwts 
      where device_id=id_of_device::bigint
    loop
      select decrypted_secret into vault_record from vault.decrypted_secrets where id=jwt_record.secret_id;
      current_set := jsonb_build_object(
        'id', to_jsonb(jwt_record.secret_id),
        'key', to_jsonb(encode(hmac(vault_record.decrypted_secret, device_api_key_secret, 'sha512'), 'hex'))
      );
      select into keys array_append(keys, current_set);
    end loop;

  return keys;
end;$$;


ALTER FUNCTION "public"."load_api_keys_old"("id_of_device" "text") OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."remove_rls_broadcast_triggers"() RETURNS "void"
    LANGUAGE "plpgsql"
    AS $$
begin
  -- Remove triggers
  drop trigger if exists handle_plans_changes on public.plans;
  drop trigger if exists handle_devices_changes on public.devices;
  drop trigger if exists handle_tags_changes on public.tags;
  drop trigger if exists handle_sessions_changes on public.sessions;
  drop trigger if exists handle_connectivity_changes on public.connectivity;
  drop trigger if exists handle_events_changes on public.events;
  drop trigger if exists handle_herds_changes on public.herds;
  drop trigger if exists handle_zones_changes on public.zones;
  drop trigger if exists handle_actions_changes on public.actions;
  
  -- Remove functions
  drop function if exists public.plans_broadcast_changes();
  drop function if exists public.devices_broadcast_changes();
  drop function if exists public.tags_broadcast_changes();
  drop function if exists public.sessions_broadcast_changes();
  drop function if exists public.connectivity_broadcast_changes();
  drop function if exists public.events_broadcast_changes();
  drop function if exists public.herds_broadcast_changes();
  drop function if exists public.zones_broadcast_changes();
  drop function if exists public.actions_broadcast_changes();
  
  -- Remove helper functions
  drop function if exists public.can_view_herd(bigint);
  drop function if exists public.can_view_device(bigint);
  drop function if exists public.can_view_event(bigint);
  drop function if exists public.can_view_session(bigint);
end;
$$;


ALTER FUNCTION "public"."remove_rls_broadcast_triggers"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."remove_user_vault_secrets"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public'
    AS $$declare
  jwt_record record;
begin
  delete from vault.secrets where name=old.id::text;
  for jwt_record IN 
    select secret_id
    from private.jwts 
    where user_id=old.id
  loop
    delete from vault.secrets where id=jwt_record.secret_id;
  end loop;
  return old;
end;$$;


ALTER FUNCTION "public"."remove_user_vault_secrets"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."test_connectivity_before_trigger"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public, extensions'
    AS $$
BEGIN
  PERFORM realtime.send(
    jsonb_build_object(
      'test', 'before_trigger',
      'tg_op', TG_OP,
      'tg_when', 'BEFORE',
      'new_exists', CASE WHEN NEW IS NOT NULL THEN true ELSE false END,
      'new_id', CASE WHEN NEW IS NOT NULL THEN NEW.id ELSE NULL END
    ),
    'TEST_BEFORE',
    '10-connectivity',
    FALSE
  );

  RETURN CASE TG_OP
    WHEN 'DELETE' THEN OLD
    ELSE NEW
  END;
END;
$$;


ALTER FUNCTION "public"."test_connectivity_before_trigger"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."test_connectivity_trigger"() RETURNS "trigger"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
  -- Test what we actually receive
  PERFORM realtime.send(
    jsonb_build_object(
      'test', 'trigger_called',
      'tg_op', TG_OP,
      'tg_when', TG_WHEN,
      'tg_level', TG_LEVEL,
      'new_exists', CASE WHEN NEW IS NOT NULL THEN true ELSE false END,
      'old_exists', CASE WHEN OLD IS NOT NULL THEN true ELSE false END,
      'new_id', CASE WHEN NEW IS NOT NULL THEN NEW.id ELSE NULL END
    ),
    'TEST',
    '10-connectivity',
    FALSE
  );

  RETURN CASE TG_OP
    WHEN 'DELETE' THEN OLD
    ELSE NEW
  END;
END;
$$;


ALTER FUNCTION "public"."test_connectivity_trigger"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."test_connectivity_trigger_bypass_rls"() RETURNS "trigger"
    LANGUAGE "plpgsql" SECURITY DEFINER
    SET "search_path" TO 'public, extensions'
    AS $$
BEGIN
  -- Test if running as SECURITY DEFINER helps
  PERFORM realtime.send(
    jsonb_build_object(
      'test', 'trigger_with_security_definer',
      'tg_op', TG_OP,
      'current_user', current_user,
      'session_user', session_user,
      'new_exists', CASE WHEN NEW IS NOT NULL THEN true ELSE false END,
      'old_exists', CASE WHEN OLD IS NOT NULL THEN true ELSE false END,
      'new_id_attempt', CASE 
        WHEN NEW IS NOT NULL THEN 
          CASE 
            WHEN TG_OP = 'DELETE' THEN 'not_applicable'
            ELSE NEW.id::text 
          END
        ELSE 'new_is_null' 
      END
    ),
    'TEST_BYPASS',
    '10-connectivity',
    FALSE
  );

  -- Try to return properly
  RETURN CASE TG_OP
    WHEN 'DELETE' THEN OLD
    WHEN 'INSERT' THEN NEW  
    WHEN 'UPDATE' THEN NEW
    ELSE NULL
  END;
END;
$$;


ALTER FUNCTION "public"."test_connectivity_trigger_bypass_rls"() OWNER TO "postgres";


CREATE OR REPLACE FUNCTION "public"."update_updated_at_column"() RETURNS "trigger"
    LANGUAGE "plpgsql"
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


ALTER FUNCTION "public"."update_updated_at_column"() OWNER TO "postgres";


COMMENT ON FUNCTION "public"."update_updated_at_column"() IS 'Automatically updates updated_at column on row updates. Uses SECURITY DEFINER to work with RLS.';



CREATE TABLE IF NOT EXISTS "private"."jwts" (
    "secret_id" "uuid" NOT NULL,
    "device_id" bigint NOT NULL
);


ALTER TABLE "private"."jwts" OWNER TO "postgres";


ALTER TABLE "public"."actions" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."actions_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



ALTER TABLE "public"."artifacts" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."artifacts_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."certificates" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "issuer" "text" NOT NULL,
    "expiration" timestamp with time zone,
    "type" "text" NOT NULL,
    "tracking_number" "text",
    "updated_at" timestamp with time zone DEFAULT "now"()
);


ALTER TABLE "public"."certificates" OWNER TO "postgres";


ALTER TABLE "public"."certificates" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."certificates_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."chat" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "message" "text" NOT NULL,
    "sender" "uuid",
    "herd_id" bigint NOT NULL
);


ALTER TABLE "public"."chat" OWNER TO "postgres";


ALTER TABLE "public"."chat" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."chat_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."connectivity" (
    "id" bigint NOT NULL,
    "session_id" bigint,
    "inserted_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "timestamp_start" timestamp with time zone NOT NULL,
    "signal" double precision NOT NULL,
    "noise" double precision NOT NULL,
    "altitude" double precision NOT NULL,
    "heading" double precision NOT NULL,
    "location" "extensions"."geography" NOT NULL,
    "h14_index" "text" NOT NULL,
    "h13_index" "text" NOT NULL,
    "h12_index" "text" NOT NULL,
    "h11_index" "text" NOT NULL,
    "battery_percentage" real,
    "device_id" bigint,
    "frequency_hz" real,
    "bandwidth_hz" real,
    "associated_station" "text",
    "mode" "text"
);


ALTER TABLE "public"."connectivity" OWNER TO "postgres";


ALTER TABLE "public"."connectivity" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."connectivity_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."devices" (
    "id" bigint NOT NULL,
    "inserted_at" timestamp with time zone DEFAULT "timezone"('utc'::"text", "now"()) NOT NULL,
    "created_by" "uuid" NOT NULL,
    "herd_id" bigint NOT NULL,
    "name" "text" NOT NULL,
    "description" "text" NOT NULL,
    "domain_name" "text",
    "altitude" double precision,
    "heading" double precision,
    "location" "extensions"."geography"(Point,4326),
    "video_publisher_token" "text",
    "video_subscriber_token" "text",
    "device_type" "public"."device_type" DEFAULT 'unknown'::"public"."device_type" NOT NULL
);

ALTER TABLE ONLY "public"."devices" REPLICA IDENTITY FULL;


ALTER TABLE "public"."devices" OWNER TO "postgres";


ALTER TABLE "public"."devices" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."devices_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."events" (
    "id" bigint NOT NULL,
    "inserted_at" timestamp with time zone DEFAULT "timezone"('utc'::"text", "now"()) NOT NULL,
    "message" "text",
    "media_url" "text",
    "altitude" double precision DEFAULT '0'::double precision NOT NULL,
    "heading" double precision DEFAULT '0'::double precision NOT NULL,
    "media_type" "public"."media_type" DEFAULT 'image'::"public"."media_type" NOT NULL,
    "device_id" bigint NOT NULL,
    "timestamp_observation" timestamp with time zone DEFAULT "timezone"('utc'::"text", "now"()) NOT NULL,
    "is_public" boolean DEFAULT false NOT NULL,
    "location" "extensions"."geography"(Point,4326) DEFAULT '0101000020E610000000000000000000000000000000000000'::"extensions"."geography",
    "earthranger_url" "text",
    "file_path" "text",
    "session_id" bigint
);


ALTER TABLE "public"."events" OWNER TO "postgres";


COMMENT ON TABLE "public"."events" IS 'Individual events sent by each user.';



COMMENT ON COLUMN "public"."events"."file_path" IS 'File path extracted from existing public URLs during migration to signed URLs';



ALTER TABLE "public"."events" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."events_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE OR REPLACE VIEW "public"."events_with_tags" WITH ("security_invoker"='on') AS
 SELECT "e"."id",
    "e"."inserted_at",
    "e"."message",
    "e"."media_url",
    "e"."file_path",
    "e"."location",
    "e"."altitude",
    "e"."heading",
    "e"."media_type",
    "e"."device_id",
    "e"."timestamp_observation",
    "e"."is_public",
    "array_agg"("t".*) AS "tags",
    "d"."herd_id",
    "e"."earthranger_url",
    "e"."session_id"
   FROM (("public"."events" "e"
     JOIN "public"."devices" "d" ON (("e"."device_id" = "d"."id")))
     LEFT JOIN "public"."tags" "t" ON (("e"."id" = "t"."event_id")))
  WHERE ("e"."session_id" IS NULL)
  GROUP BY "e"."id", "e"."inserted_at", "e"."message", "e"."media_url", "e"."file_path", "e"."location", "e"."altitude", "e"."heading", "e"."media_type", "e"."device_id", "e"."timestamp_observation", "e"."is_public", "d"."herd_id", "e"."earthranger_url", "e"."session_id"
  ORDER BY "e"."inserted_at" DESC;


ALTER VIEW "public"."events_with_tags" OWNER TO "postgres";


CREATE OR REPLACE VIEW "public"."events_with_tags_by_session" AS
SELECT
    NULL::bigint AS "id",
    NULL::timestamp with time zone AS "inserted_at",
    NULL::"text" AS "message",
    NULL::"text" AS "media_url",
    NULL::"text" AS "file_path",
    NULL::"extensions"."geography"(Point,4326) AS "location",
    NULL::double precision AS "altitude",
    NULL::double precision AS "heading",
    NULL::"public"."media_type" AS "media_type",
    NULL::bigint AS "device_id",
    NULL::timestamp with time zone AS "timestamp_observation",
    NULL::boolean AS "is_public",
    NULL::"public"."tags"[] AS "tags",
    NULL::bigint AS "herd_id",
    NULL::"text" AS "earthranger_url",
    NULL::bigint AS "session_id";


ALTER VIEW "public"."events_with_tags_by_session" OWNER TO "postgres";


CREATE TABLE IF NOT EXISTS "public"."heartbeats" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "timestamp" timestamp with time zone NOT NULL,
    "device_id" bigint NOT NULL
);


ALTER TABLE "public"."heartbeats" OWNER TO "postgres";


ALTER TABLE "public"."heartbeats" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."heartbeats_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



ALTER TABLE "public"."herds" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."herds_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."layers" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "features" json NOT NULL,
    "herd_id" bigint NOT NULL
);


ALTER TABLE "public"."layers" OWNER TO "postgres";


ALTER TABLE "public"."layers" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."layers_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."operators" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "timestamp" timestamp with time zone,
    "session_id" bigint,
    "user_id" "uuid" NOT NULL,
    "action" "text"
);


ALTER TABLE "public"."operators" OWNER TO "postgres";


ALTER TABLE "public"."operators" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."operators_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."parts" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "device_id" bigint NOT NULL,
    "serial_number" "text" NOT NULL,
    "product_number" "text" NOT NULL,
    "certificate_id" bigint,
    "status" "public"."component_status" DEFAULT 'active'::"public"."component_status" NOT NULL,
    "updated_at" timestamp with time zone DEFAULT "now"(),
    "deleted_at" timestamp with time zone
);


ALTER TABLE "public"."parts" OWNER TO "postgres";


COMMENT ON COLUMN "public"."parts"."deleted_at" IS 'Timestamp when part was soft deleted. NULL means the part is active.';



ALTER TABLE "public"."parts" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."parts_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."pins" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "altitude_relative_to_ground" bigint NOT NULL,
    "color" "text" NOT NULL,
    "name" "text" NOT NULL,
    "description" "text",
    "herd_id" bigint NOT NULL,
    "created_by" "uuid",
    "location" "extensions"."geography"(Point,4326) DEFAULT '0101000020E610000000000000000000000000000000000000'::"extensions"."geography"
);


ALTER TABLE "public"."pins" OWNER TO "postgres";


COMMENT ON COLUMN "public"."pins"."location" IS 'PostGIS geography field storing pin location as Point(longitude, latitude) in WGS84 (SRID 4326), following same pattern as events table';



ALTER TABLE "public"."pins" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."pins_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."plans" (
    "id" bigint NOT NULL,
    "inserted_at" timestamp with time zone DEFAULT "timezone"('utc'::"text", "now"()),
    "name" "text" NOT NULL,
    "instructions" "text" NOT NULL,
    "herd_id" bigint NOT NULL,
    "plan_type" "public"."plan_type" DEFAULT 'mission'::"public"."plan_type" NOT NULL
);


ALTER TABLE "public"."plans" OWNER TO "postgres";


ALTER TABLE "public"."plans" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."plans_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."providers" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "type" "text" NOT NULL,
    "key" "text",
    "source" "text" NOT NULL,
    "herd_id" bigint NOT NULL
);


ALTER TABLE "public"."providers" OWNER TO "postgres";


COMMENT ON TABLE "public"."providers" IS 'Providers table with RLS enabled. Access controlled by herd membership and user roles.';



ALTER TABLE "public"."providers" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."providers_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."sessions" (
    "id" bigint NOT NULL,
    "device_id" bigint NOT NULL,
    "timestamp_start" timestamp with time zone NOT NULL,
    "timestamp_end" timestamp with time zone,
    "inserted_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "software_version" "text" NOT NULL,
    "locations" "extensions"."geography",
    "altitude_max" double precision NOT NULL,
    "altitude_min" double precision NOT NULL,
    "altitude_average" double precision NOT NULL,
    "velocity_max" double precision NOT NULL,
    "velocity_min" double precision NOT NULL,
    "velocity_average" double precision NOT NULL,
    "distance_total" double precision NOT NULL,
    "distance_max_from_start" double precision NOT NULL,
    "earthranger_url" "text"
);


ALTER TABLE "public"."sessions" OWNER TO "postgres";


ALTER TABLE "public"."sessions" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."sessions_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



ALTER TABLE "public"."tags" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."tags_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."users" (
    "id" "uuid" NOT NULL,
    "username" "text"
);


ALTER TABLE "public"."users" OWNER TO "postgres";


COMMENT ON TABLE "public"."users" IS 'Profile data for each user.';



COMMENT ON COLUMN "public"."users"."id" IS 'References the internal Supabase Auth user.';



CREATE TABLE IF NOT EXISTS "public"."users_roles_per_herd" (
    "id" bigint NOT NULL,
    "inserted_at" timestamp with time zone DEFAULT "timezone"('utc'::"text", "now"()) NOT NULL,
    "user_id" "uuid" NOT NULL,
    "herd_id" bigint NOT NULL,
    "role" "public"."role" NOT NULL
);


ALTER TABLE "public"."users_roles_per_herd" OWNER TO "postgres";


ALTER TABLE "public"."users_roles_per_herd" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."users_roles_per_herd_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."versions_software" (
    "id" bigint NOT NULL,
    "created_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "version" "text" NOT NULL,
    "hyperlink" "text",
    "description" "text" NOT NULL,
    "title" "text",
    "system" "text" NOT NULL,
    "updated_at" timestamp with time zone DEFAULT "now"(),
    "commit_hash" "text",
    "stable" boolean DEFAULT false NOT NULL,
    "pre" boolean DEFAULT true NOT NULL,
    "min" boolean DEFAULT false NOT NULL
);


ALTER TABLE "public"."versions_software" OWNER TO "postgres";


ALTER TABLE "public"."versions_software" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."versions_software_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



CREATE TABLE IF NOT EXISTS "public"."zones" (
    "id" bigint NOT NULL,
    "inserted_at" timestamp with time zone DEFAULT "now"() NOT NULL,
    "region" "extensions"."geometry" NOT NULL,
    "herd_id" bigint NOT NULL
);


ALTER TABLE "public"."zones" OWNER TO "postgres";


CREATE OR REPLACE VIEW "public"."zones_and_actions" WITH ("security_invoker"='on') AS
 SELECT "z"."id",
    "z"."inserted_at",
    "z"."region",
    "z"."herd_id",
    "array_agg"("a".*) AS "actions"
   FROM ("public"."zones" "z"
     JOIN "public"."actions" "a" ON (("z"."id" = "a"."zone_id")))
  GROUP BY "z"."id", "z"."inserted_at", "z"."region", "z"."herd_id"
  ORDER BY "z"."inserted_at" DESC;


ALTER VIEW "public"."zones_and_actions" OWNER TO "postgres";


ALTER TABLE "public"."zones" ALTER COLUMN "id" ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME "public"."zones_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);



ALTER TABLE ONLY "private"."jwts"
    ADD CONSTRAINT "jwts_pkey" PRIMARY KEY ("secret_id");



ALTER TABLE ONLY "public"."actions"
    ADD CONSTRAINT "actions_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."artifacts"
    ADD CONSTRAINT "artifacts_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."certificates"
    ADD CONSTRAINT "certificates_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."chat"
    ADD CONSTRAINT "chat_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."connectivity"
    ADD CONSTRAINT "connectivity_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."devices"
    ADD CONSTRAINT "devices_name_key" UNIQUE ("name");



ALTER TABLE ONLY "public"."devices"
    ADD CONSTRAINT "devices_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."events"
    ADD CONSTRAINT "events_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."heartbeats"
    ADD CONSTRAINT "heartbeats_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."herds"
    ADD CONSTRAINT "herds_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."herds"
    ADD CONSTRAINT "herds_slug_key" UNIQUE ("slug");



ALTER TABLE ONLY "public"."layers"
    ADD CONSTRAINT "layers_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."operators"
    ADD CONSTRAINT "operators_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."parts"
    ADD CONSTRAINT "parts_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."parts"
    ADD CONSTRAINT "parts_product_serial_unique" UNIQUE ("product_number", "serial_number");



COMMENT ON CONSTRAINT "parts_product_serial_unique" ON "public"."parts" IS 'Ensures unique combination of product_number and serial_number. Allows same serial numbers across different product types.';



ALTER TABLE ONLY "public"."pins"
    ADD CONSTRAINT "pins_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."plans"
    ADD CONSTRAINT "plans_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."providers"
    ADD CONSTRAINT "providers_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."sessions"
    ADD CONSTRAINT "sessions_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."tags"
    ADD CONSTRAINT "tags_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."users"
    ADD CONSTRAINT "users_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."users_roles_per_herd"
    ADD CONSTRAINT "users_roles_per_herd_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."versions_software"
    ADD CONSTRAINT "versions_software_pkey" PRIMARY KEY ("id");



ALTER TABLE ONLY "public"."zones"
    ADD CONSTRAINT "zones_pkey" PRIMARY KEY ("id");



CREATE INDEX "idx_actions_zone_id" ON "public"."actions" USING "btree" ("zone_id");



CREATE INDEX "idx_artifacts_session_id_fkey" ON "public"."artifacts" USING "btree" ("session_id");



CREATE INDEX "idx_chat_herd_id_fkey" ON "public"."chat" USING "btree" ("herd_id");



CREATE INDEX "idx_chat_sender" ON "public"."chat" USING "btree" ("sender");



CREATE INDEX "idx_connectivity_device_timestamp_filter" ON "public"."connectivity" USING "btree" ("device_id", "timestamp_start" DESC) WHERE ("device_id" IS NOT NULL);



CREATE INDEX "idx_connectivity_session_id_fkey" ON "public"."connectivity" USING "btree" ("session_id");



CREATE INDEX "idx_connectivity_session_timestamp_filter" ON "public"."connectivity" USING "btree" ("session_id", "timestamp_start" DESC) WHERE ("session_id" IS NOT NULL);



CREATE INDEX "idx_devices_created_by" ON "public"."devices" USING "btree" ("created_by");



CREATE INDEX "idx_devices_herd_id" ON "public"."devices" USING "btree" ("herd_id");



CREATE INDEX "idx_events_device_id" ON "public"."events" USING "btree" ("device_id");



CREATE INDEX "idx_events_device_timestamp" ON "public"."events" USING "btree" ("device_id", "timestamp_observation" DESC);



CREATE INDEX "idx_events_herd_session" ON "public"."events" USING "btree" ("device_id") WHERE ("session_id" IS NULL);



CREATE INDEX "idx_events_inserted_at" ON "public"."events" USING "btree" ("inserted_at" DESC);



CREATE INDEX "idx_events_is_public" ON "public"."events" USING "btree" ("is_public");



CREATE INDEX "idx_events_media_type" ON "public"."events" USING "btree" ("media_type");



CREATE INDEX "idx_events_session_id" ON "public"."events" USING "btree" ("session_id");



CREATE INDEX "idx_events_session_timestamp" ON "public"."events" USING "btree" ("session_id", "timestamp_observation" DESC);



CREATE INDEX "idx_events_timestamp_observation" ON "public"."events" USING "btree" ("timestamp_observation" DESC);



CREATE INDEX "idx_herds_created_by" ON "public"."herds" USING "btree" ("created_by");



CREATE INDEX "idx_layers_herd_id" ON "public"."layers" USING "btree" ("herd_id");



CREATE INDEX "idx_parts_active" ON "public"."parts" USING "btree" ("device_id", "created_at") WHERE ("deleted_at" IS NULL);



CREATE INDEX "idx_parts_deleted_at" ON "public"."parts" USING "btree" ("deleted_at");



CREATE INDEX "idx_parts_product_serial" ON "public"."parts" USING "btree" ("product_number", "serial_number") WHERE ("deleted_at" IS NULL);



CREATE INDEX "idx_pins_created_at" ON "public"."pins" USING "btree" ("created_at" DESC);



CREATE INDEX "idx_pins_created_by" ON "public"."pins" USING "btree" ("created_by");



CREATE INDEX "idx_pins_herd_id" ON "public"."pins" USING "btree" ("herd_id");



CREATE INDEX "idx_pins_location_gist" ON "public"."pins" USING "gist" ("location");



CREATE INDEX "idx_plans_herd_id" ON "public"."plans" USING "btree" ("herd_id");



CREATE INDEX "idx_plans_herd_id_fkey" ON "public"."plans" USING "btree" ("herd_id");



CREATE INDEX "idx_plans_inserted_at" ON "public"."plans" USING "btree" ("inserted_at" DESC);



CREATE INDEX "idx_plans_plan_type" ON "public"."plans" USING "btree" ("plan_type");



CREATE INDEX "idx_sessions_device_id" ON "public"."sessions" USING "btree" ("device_id");



CREATE INDEX "idx_sessions_device_id_fkey" ON "public"."sessions" USING "btree" ("device_id");



CREATE INDEX "idx_sessions_device_timestamp" ON "public"."sessions" USING "btree" ("device_id", "timestamp_start" DESC);



CREATE INDEX "idx_sessions_inserted_at" ON "public"."sessions" USING "btree" ("inserted_at" DESC);



CREATE INDEX "idx_sessions_software_version" ON "public"."sessions" USING "btree" ("software_version") WHERE ("software_version" IS NOT NULL);



CREATE INDEX "idx_sessions_summary_filters" ON "public"."sessions" USING "btree" ("device_id", "timestamp_start" DESC, "timestamp_end", "distance_total", "software_version");



CREATE INDEX "idx_sessions_timestamp_end" ON "public"."sessions" USING "btree" ("timestamp_end" DESC);



CREATE INDEX "idx_sessions_timestamp_start" ON "public"."sessions" USING "btree" ("timestamp_start" DESC);



CREATE INDEX "idx_tags_class_name" ON "public"."tags" USING "btree" ("class_name");



CREATE INDEX "idx_tags_conf" ON "public"."tags" USING "btree" ("conf" DESC);



CREATE INDEX "idx_tags_event_id" ON "public"."tags" USING "btree" ("event_id");



CREATE INDEX "idx_tags_inserted_at" ON "public"."tags" USING "btree" ("inserted_at" DESC);



CREATE INDEX "idx_tags_observation_type" ON "public"."tags" USING "btree" ("observation_type");



CREATE INDEX "idx_users_roles_per_herd_herd_id_fkey" ON "public"."users_roles_per_herd" USING "btree" ("herd_id");



CREATE INDEX "idx_users_roles_per_herd_user_id" ON "public"."users_roles_per_herd" USING "btree" ("user_id");



CREATE INDEX "idx_versions_software_created_at" ON "public"."versions_software" USING "btree" ("created_at" DESC);



CREATE INDEX "idx_versions_software_system_version" ON "public"."versions_software" USING "btree" ("system", "version");



CREATE INDEX "idx_zones_herd_id" ON "public"."zones" USING "btree" ("herd_id");



CREATE OR REPLACE VIEW "public"."events_with_tags_by_session" WITH ("security_invoker"='on') AS
 SELECT "e"."id",
    "e"."inserted_at",
    "e"."message",
    "e"."media_url",
    "e"."file_path",
    "e"."location",
    "e"."altitude",
    "e"."heading",
    "e"."media_type",
    "e"."device_id",
    "e"."timestamp_observation",
    "e"."is_public",
    "array_agg"("t".*) AS "tags",
    "d"."herd_id",
    "e"."earthranger_url",
    "e"."session_id"
   FROM (("public"."events" "e"
     JOIN "public"."devices" "d" ON (("e"."device_id" = "d"."id")))
     LEFT JOIN "public"."tags" "t" ON (("e"."id" = "t"."event_id")))
  WHERE ("e"."session_id" IS NOT NULL)
  GROUP BY "e"."id", "e"."inserted_at", "e"."message", "e"."media_url", "e"."file_path", "e"."location", "d"."herd_id", "e"."session_id"
  ORDER BY "e"."timestamp_observation" DESC;



CREATE OR REPLACE TRIGGER "events_delete_broadcast_trigger" AFTER DELETE ON "public"."events" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_events_changes"();



CREATE OR REPLACE TRIGGER "events_insert_broadcast_trigger" AFTER INSERT ON "public"."events" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_events_changes"();



CREATE OR REPLACE TRIGGER "events_update_broadcast_trigger" AFTER UPDATE ON "public"."events" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_events_changes"();



CREATE OR REPLACE TRIGGER "handle_connectivity_changes" AFTER INSERT ON "public"."connectivity" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_connectivity_changes"();



CREATE OR REPLACE TRIGGER "handle_device_changes" AFTER INSERT OR DELETE OR UPDATE ON "public"."devices" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_device_changes"();



CREATE OR REPLACE TRIGGER "handle_device_realtime_changes" AFTER INSERT OR DELETE OR UPDATE ON "public"."devices" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_device_changes"();



CREATE OR REPLACE TRIGGER "on_device_created__create_device_api_key_secret" AFTER INSERT ON "public"."devices" FOR EACH ROW EXECUTE FUNCTION "private"."add_device_api_secret_and_keys"();



CREATE OR REPLACE TRIGGER "on_device_deleted__remove_device_api_key_secret" AFTER DELETE ON "public"."devices" FOR EACH ROW EXECUTE FUNCTION "private"."remove_device_api_secret_and_keys"();



CREATE OR REPLACE TRIGGER "on_event_created__fill_event_location" BEFORE INSERT ON "public"."events" FOR EACH ROW EXECUTE FUNCTION "private"."fill_event_location"();



CREATE OR REPLACE TRIGGER "on_herd_created__add_user_as_herd_admin" AFTER INSERT ON "public"."herds" FOR EACH ROW EXECUTE FUNCTION "public"."add_user_as_herd_admin"();



CREATE OR REPLACE TRIGGER "parts_delete_broadcast_trigger" AFTER DELETE ON "public"."parts" FOR EACH ROW EXECUTE FUNCTION "private"."broadcast_parts_changes"();



CREATE OR REPLACE TRIGGER "parts_insert_broadcast_trigger" AFTER INSERT ON "public"."parts" FOR EACH ROW EXECUTE FUNCTION "private"."broadcast_parts_changes"();



CREATE OR REPLACE TRIGGER "parts_update_broadcast_trigger" AFTER UPDATE ON "public"."parts" FOR EACH ROW EXECUTE FUNCTION "private"."broadcast_parts_changes"();



CREATE OR REPLACE TRIGGER "pins_delete_broadcast_trigger" AFTER DELETE ON "public"."pins" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_pins_changes"();



CREATE OR REPLACE TRIGGER "pins_insert_broadcast_trigger" AFTER INSERT ON "public"."pins" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_pins_changes"();



CREATE OR REPLACE TRIGGER "pins_update_broadcast_trigger" AFTER UPDATE ON "public"."pins" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_pins_changes"();



CREATE OR REPLACE TRIGGER "plans_delete_broadcast_trigger" AFTER DELETE ON "public"."plans" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_plans_changes"();



CREATE OR REPLACE TRIGGER "plans_insert_broadcast_trigger" AFTER INSERT ON "public"."plans" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_plans_changes"();



CREATE OR REPLACE TRIGGER "plans_update_broadcast_trigger" AFTER UPDATE ON "public"."plans" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_plans_changes"();



CREATE OR REPLACE TRIGGER "sessions_delete_broadcast_trigger" AFTER DELETE ON "public"."sessions" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_sessions_changes"();



CREATE OR REPLACE TRIGGER "sessions_insert_broadcast_trigger" AFTER INSERT ON "public"."sessions" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_sessions_changes"();



CREATE OR REPLACE TRIGGER "sessions_update_broadcast_trigger" AFTER UPDATE ON "public"."sessions" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_sessions_changes"();



CREATE OR REPLACE TRIGGER "tags_delete_broadcast_trigger" AFTER DELETE ON "public"."tags" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_tags_changes"();



CREATE OR REPLACE TRIGGER "tags_insert_broadcast_trigger" AFTER INSERT ON "public"."tags" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_tags_changes"();



CREATE OR REPLACE TRIGGER "tags_update_broadcast_trigger" AFTER UPDATE ON "public"."tags" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_tags_changes"();



CREATE OR REPLACE TRIGGER "trigger_single_min_per_system_insert" BEFORE INSERT ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."enforce_single_min_per_system"();



CREATE OR REPLACE TRIGGER "trigger_single_min_per_system_update" BEFORE UPDATE ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."enforce_single_min_per_system"();



CREATE OR REPLACE TRIGGER "trigger_single_stable_per_system_insert" BEFORE INSERT ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."enforce_single_stable_per_system"();



CREATE OR REPLACE TRIGGER "trigger_single_stable_per_system_update" BEFORE UPDATE ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."enforce_single_stable_per_system"();



CREATE OR REPLACE TRIGGER "trigger_stable_pre_exclusivity_insert" BEFORE INSERT ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."check_stable_pre_exclusivity"();



CREATE OR REPLACE TRIGGER "trigger_stable_pre_exclusivity_update" BEFORE UPDATE ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."check_stable_pre_exclusivity"();



CREATE OR REPLACE TRIGGER "update_artifacts_updated_at" BEFORE UPDATE ON "public"."artifacts" FOR EACH ROW EXECUTE FUNCTION "public"."update_updated_at_column"();



CREATE OR REPLACE TRIGGER "update_certificates_updated_at" BEFORE UPDATE ON "public"."certificates" FOR EACH ROW EXECUTE FUNCTION "public"."update_updated_at_column"();



CREATE OR REPLACE TRIGGER "update_parts_updated_at" BEFORE UPDATE ON "public"."parts" FOR EACH ROW EXECUTE FUNCTION "public"."update_updated_at_column"();



COMMENT ON TRIGGER "update_parts_updated_at" ON "public"."parts" IS 'Automatically updates the updated_at column whenever a part record is modified';



CREATE OR REPLACE TRIGGER "update_versions_software_updated_at" BEFORE UPDATE ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."update_updated_at_column"();



CREATE OR REPLACE TRIGGER "versions_software_delete_broadcast_trigger" AFTER DELETE ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_versions_software_changes"();



CREATE OR REPLACE TRIGGER "versions_software_insert_broadcast_trigger" AFTER INSERT ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_versions_software_changes"();



CREATE OR REPLACE TRIGGER "versions_software_update_broadcast_trigger" AFTER UPDATE ON "public"."versions_software" FOR EACH ROW EXECUTE FUNCTION "public"."broadcast_versions_software_changes"();



ALTER TABLE ONLY "public"."actions"
    ADD CONSTRAINT "actions_zone_id_fkey" FOREIGN KEY ("zone_id") REFERENCES "public"."zones"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."artifacts"
    ADD CONSTRAINT "artifacts_device_id_fkey" FOREIGN KEY ("device_id") REFERENCES "public"."devices"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."artifacts"
    ADD CONSTRAINT "artifacts_session_id_fkey" FOREIGN KEY ("session_id") REFERENCES "public"."sessions"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."chat"
    ADD CONSTRAINT "chat_herd_id_fkey" FOREIGN KEY ("herd_id") REFERENCES "public"."herds"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."chat"
    ADD CONSTRAINT "chat_sender_fkey" FOREIGN KEY ("sender") REFERENCES "public"."users"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."connectivity"
    ADD CONSTRAINT "connectivity_device_id_fkey" FOREIGN KEY ("device_id") REFERENCES "public"."devices"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."connectivity"
    ADD CONSTRAINT "connectivity_session_id_fkey" FOREIGN KEY ("session_id") REFERENCES "public"."sessions"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."devices"
    ADD CONSTRAINT "devices_created_by_fkey" FOREIGN KEY ("created_by") REFERENCES "public"."users"("id");



ALTER TABLE ONLY "public"."devices"
    ADD CONSTRAINT "devices_herd_id_fkey" FOREIGN KEY ("herd_id") REFERENCES "public"."herds"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."events"
    ADD CONSTRAINT "events_device_id_fkey" FOREIGN KEY ("device_id") REFERENCES "public"."devices"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."events"
    ADD CONSTRAINT "events_session_id_fkey" FOREIGN KEY ("session_id") REFERENCES "public"."sessions"("id") ON UPDATE CASCADE ON DELETE SET NULL;



ALTER TABLE ONLY "public"."heartbeats"
    ADD CONSTRAINT "heartbeats_device_id_fkey" FOREIGN KEY ("device_id") REFERENCES "public"."devices"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."herds"
    ADD CONSTRAINT "herds_created_by_fkey" FOREIGN KEY ("created_by") REFERENCES "public"."users"("id");



ALTER TABLE ONLY "public"."layers"
    ADD CONSTRAINT "layers_herd_id_fkey" FOREIGN KEY ("herd_id") REFERENCES "public"."herds"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."operators"
    ADD CONSTRAINT "operators_session_id_fkey" FOREIGN KEY ("session_id") REFERENCES "public"."sessions"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."operators"
    ADD CONSTRAINT "operators_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."parts"
    ADD CONSTRAINT "parts_certificate_id_fkey" FOREIGN KEY ("certificate_id") REFERENCES "public"."certificates"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."parts"
    ADD CONSTRAINT "parts_device_id_fkey" FOREIGN KEY ("device_id") REFERENCES "public"."devices"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."pins"
    ADD CONSTRAINT "pins_created_by_fkey" FOREIGN KEY ("created_by") REFERENCES "public"."users"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."pins"
    ADD CONSTRAINT "pins_herd_id_fkey" FOREIGN KEY ("herd_id") REFERENCES "public"."herds"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."plans"
    ADD CONSTRAINT "plans_herd_id_fkey" FOREIGN KEY ("herd_id") REFERENCES "public"."herds"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."providers"
    ADD CONSTRAINT "providers_herd_id_fkey" FOREIGN KEY ("herd_id") REFERENCES "public"."herds"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."sessions"
    ADD CONSTRAINT "sessions_device_id_fkey" FOREIGN KEY ("device_id") REFERENCES "public"."devices"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."tags"
    ADD CONSTRAINT "tags_event_id_fkey" FOREIGN KEY ("event_id") REFERENCES "public"."events"("id") ON DELETE CASCADE;



ALTER TABLE ONLY "public"."users"
    ADD CONSTRAINT "users_id_fkey" FOREIGN KEY ("id") REFERENCES "auth"."users"("id");



ALTER TABLE ONLY "public"."users_roles_per_herd"
    ADD CONSTRAINT "users_roles_per_herd_herd_id_fkey" FOREIGN KEY ("herd_id") REFERENCES "public"."herds"("id") ON UPDATE CASCADE ON DELETE CASCADE;



ALTER TABLE ONLY "public"."users_roles_per_herd"
    ADD CONSTRAINT "users_roles_per_herd_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id");



ALTER TABLE ONLY "public"."zones"
    ADD CONSTRAINT "zones_herd_id_fkey" FOREIGN KEY ("herd_id") REFERENCES "public"."herds"("id") ON UPDATE CASCADE ON DELETE CASCADE;



CREATE POLICY "Deny all direct access to private.jwts" ON "private"."jwts" USING (false);



ALTER TABLE "private"."jwts" ENABLE ROW LEVEL SECURITY;


CREATE POLICY "Action access: Device API keys and users with view role" ON "public"."actions" FOR SELECT USING (("private"."herd_has_device"(( SELECT "private"."get_herd_by_zone_id"("actions"."zone_id") AS "get_herd_by_zone_id"), ( SELECT "private"."key_uid"() AS "key_uid")) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_by_zone_id"("actions"."zone_id") AS "get_herd_by_zone_id"))));



CREATE POLICY "Action creation: Device API keys and users with edit role" ON "public"."actions" FOR INSERT WITH CHECK (("private"."herd_has_device"(( SELECT "private"."get_herd_by_zone_id"("actions"."zone_id") AS "get_herd_by_zone_id"), ( SELECT "private"."key_uid"() AS "key_uid")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_by_zone_id"("actions"."zone_id") AS "get_herd_by_zone_id"))));



CREATE POLICY "Action deletion: Device API keys and users with edit role" ON "public"."actions" FOR DELETE USING (("private"."herd_has_device"(( SELECT "private"."get_herd_by_zone_id"("actions"."zone_id") AS "get_herd_by_zone_id"), ( SELECT "private"."key_uid"() AS "key_uid")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_by_zone_id"("actions"."zone_id") AS "get_herd_by_zone_id"))));



CREATE POLICY "Action modification: Device API keys and users with edit role" ON "public"."actions" FOR UPDATE USING (("private"."herd_has_device"(( SELECT "private"."get_herd_by_zone_id"("actions"."zone_id") AS "get_herd_by_zone_id"), ( SELECT "private"."key_uid"() AS "key_uid")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_by_zone_id"("actions"."zone_id") AS "get_herd_by_zone_id"))));



CREATE POLICY "Anyone can view users" ON "public"."users" FOR SELECT USING (true);



CREATE POLICY "Artifact access: Device API keys and users with view role" ON "public"."artifacts" FOR SELECT USING ((("device_id" = "private"."key_uid"()) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("artifacts"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Artifact creation: Device API keys and users with edit role" ON "public"."artifacts" FOR INSERT WITH CHECK ((("device_id" = "private"."key_uid"()) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("artifacts"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Artifact deletion: Device API keys and users with edit role" ON "public"."artifacts" FOR DELETE USING ((("device_id" = "private"."key_uid"()) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("artifacts"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Artifact modification: Device API keys and users with edit role" ON "public"."artifacts" FOR UPDATE USING ((("device_id" = "private"."key_uid"()) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("artifacts"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Chat access: Device API keys and users with view role" ON "public"."chat" FOR SELECT TO "authenticated" USING (("private"."herd_has_device"("herd_id", "private"."key_uid"()) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Chat creation: Device API keys and users with view role" ON "public"."chat" FOR INSERT TO "authenticated" WITH CHECK (("private"."herd_has_device"("herd_id", "private"."key_uid"()) OR ("private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id") AND (("sender" IS NULL) OR ("sender" = ( SELECT "auth"."uid"() AS "uid"))))));



CREATE POLICY "Chat deletion: Device API keys and message owners" ON "public"."chat" FOR DELETE TO "authenticated" USING (("private"."herd_has_device"("herd_id", "private"."key_uid"()) OR ("sender" = ( SELECT "auth"."uid"() AS "uid"))));



CREATE POLICY "Chat modification: Device API keys and message owners" ON "public"."chat" FOR UPDATE TO "authenticated" USING (("private"."herd_has_device"("herd_id", "private"."key_uid"()) OR ("sender" = ( SELECT "auth"."uid"() AS "uid")))) WITH CHECK (("private"."herd_has_device"("herd_id", "private"."key_uid"()) OR ("sender" = ( SELECT "auth"."uid"() AS "uid"))));



CREATE POLICY "Connectivity access: Device API keys and users with view role" ON "public"."connectivity" FOR SELECT USING (((("device_id" IS NULL) AND ("session_id" IS NOT NULL) AND (("session_id" IN ( SELECT "sessions"."id"
   FROM "public"."sessions"
  WHERE ("sessions"."device_id" = "private"."key_uid"()))) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), ( SELECT "sessions"."device_id"
   FROM "public"."sessions"
  WHERE ("sessions"."id" = "connectivity"."session_id")))))) OR (("session_id" IS NULL) AND ("device_id" IS NOT NULL) AND ("device_id" = "private"."key_uid"())) OR (("session_id" IS NOT NULL) AND ("device_id" IS NOT NULL) AND ("device_id" = "private"."key_uid"()) AND ("session_id" IN ( SELECT "sessions"."id"
   FROM "public"."sessions"
  WHERE ("sessions"."device_id" = "private"."key_uid"())))) OR (("device_id" IS NOT NULL) AND ("device_id" <> "private"."key_uid"()) AND ("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"),
CASE
    WHEN ("device_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"("device_id")
    WHEN ("session_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"(( SELECT "sessions"."device_id"
       FROM "public"."sessions"
      WHERE ("sessions"."id" = "connectivity"."session_id")))
    ELSE NULL::bigint
END)));



COMMENT ON POLICY "Connectivity access: Device API keys and users with view role" ON "public"."connectivity" IS 'Enhanced policy allowing:
1. Session-based connectivity access for device-owned sessions
2. Devices to read connectivity records for themselves
3. Hybrid connectivity access for device-owned sessions
4. Approved gateway device types to read connectivity records for other devices in the same herd (enables upserts)
5. Users with view role to read connectivity records for devices in their herds
OPTIMIZED: auth.uid() wrapped in subqueries for better performance';



CREATE POLICY "Connectivity creation: Device API keys and users with edit role" ON "public"."connectivity" FOR INSERT WITH CHECK (((("device_id" IS NULL) AND ("session_id" IS NOT NULL) AND (("session_id" IN ( SELECT "sessions"."id"
   FROM "public"."sessions"
  WHERE ("sessions"."device_id" = "private"."key_uid"()))) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), ( SELECT "sessions"."device_id"
   FROM "public"."sessions"
  WHERE ("sessions"."id" = "connectivity"."session_id")))))) OR (("session_id" IS NULL) AND ("device_id" IS NOT NULL) AND ("device_id" = "private"."key_uid"())) OR (("session_id" IS NOT NULL) AND ("device_id" IS NOT NULL) AND ("device_id" = "private"."key_uid"()) AND ("session_id" IN ( SELECT "sessions"."id"
   FROM "public"."sessions"
  WHERE ("sessions"."device_id" = "private"."key_uid"())))) OR (("device_id" IS NOT NULL) AND ("device_id" <> "private"."key_uid"()) AND ("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"),
CASE
    WHEN ("device_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"("device_id")
    WHEN ("session_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"(( SELECT "sessions"."device_id"
       FROM "public"."sessions"
      WHERE ("sessions"."id" = "connectivity"."session_id")))
    ELSE NULL::bigint
END)));



COMMENT ON POLICY "Connectivity creation: Device API keys and users with edit role" ON "public"."connectivity" IS 'Enhanced policy allowing:
1. Session-based connectivity for device-owned sessions
2. Devices to create connectivity records for themselves
3. Hybrid connectivity for device-owned sessions
4. Approved gateway device types to create connectivity records for other devices in the same herd
5. Users with edit role to create connectivity records for devices in their herds
OPTIMIZED: auth.uid() wrapped in subqueries for better performance';



CREATE POLICY "Connectivity deletion: Device API keys and users with edit role" ON "public"."connectivity" FOR DELETE USING (((("device_id" IS NULL) AND ("session_id" IS NOT NULL) AND ("session_id" IN ( SELECT "sessions"."id"
   FROM "public"."sessions"
  WHERE ("sessions"."device_id" = "private"."key_uid"())))) OR (("session_id" IS NULL) AND ("device_id" IS NOT NULL) AND ("device_id" = "private"."key_uid"())) OR (("session_id" IS NOT NULL) AND ("device_id" IS NOT NULL) AND ("device_id" = "private"."key_uid"()) AND ("session_id" IN ( SELECT "sessions"."id"
   FROM "public"."sessions"
  WHERE ("sessions"."device_id" = "private"."key_uid"())))) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"),
CASE
    WHEN ("device_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"("device_id")
    WHEN ("session_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"(( SELECT "sessions"."device_id"
       FROM "public"."sessions"
      WHERE ("sessions"."id" = "connectivity"."session_id")))
    ELSE NULL::bigint
END)));



COMMENT ON POLICY "Connectivity deletion: Device API keys and users with edit role" ON "public"."connectivity" IS 'Enhanced policy allowing:
1. Session-based connectivity deletion for device-owned sessions
2. Devices to delete connectivity records for themselves
3. Hybrid connectivity deletion for device-owned sessions
4. Users with edit role to delete connectivity records for devices in their herds
OPTIMIZED: auth.uid() wrapped in subqueries for better performance';



CREATE POLICY "Connectivity modification: Device API keys and users with edit " ON "public"."connectivity" FOR UPDATE USING (((("device_id" IS NULL) AND ("session_id" IS NOT NULL) AND (("session_id" IN ( SELECT "sessions"."id"
   FROM "public"."sessions"
  WHERE ("sessions"."device_id" = "private"."key_uid"()))) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), ( SELECT "sessions"."device_id"
   FROM "public"."sessions"
  WHERE ("sessions"."id" = "connectivity"."session_id")))))) OR (("session_id" IS NULL) AND ("device_id" IS NOT NULL) AND ("device_id" = "private"."key_uid"())) OR (("session_id" IS NOT NULL) AND ("device_id" IS NOT NULL) AND ("device_id" = "private"."key_uid"()) AND ("session_id" IN ( SELECT "sessions"."id"
   FROM "public"."sessions"
  WHERE ("sessions"."device_id" = "private"."key_uid"())))) OR (("device_id" IS NOT NULL) AND ("device_id" <> "private"."key_uid"()) AND ("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"),
CASE
    WHEN ("device_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"("device_id")
    WHEN ("session_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"(( SELECT "sessions"."device_id"
       FROM "public"."sessions"
      WHERE ("sessions"."id" = "connectivity"."session_id")))
    ELSE NULL::bigint
END)));



COMMENT ON POLICY "Connectivity modification: Device API keys and users with edit " ON "public"."connectivity" IS 'Enhanced policy allowing:
1. Session-based connectivity updates for device-owned sessions
2. Devices to update connectivity records for themselves
3. Hybrid connectivity updates for device-owned sessions
4. Approved gateway device types to update connectivity records for other devices in the same herd
5. Users with edit role to update connectivity records for devices in their herds
OPTIMIZED: auth.uid() wrapped in subqueries for better performance';



CREATE POLICY "Device access: API keys, herd devices, and users with view role" ON "public"."devices" FOR SELECT USING ((("id" = "private"."key_uid"()) OR "private"."herd_has_device"("herd_id", "private"."key_uid"()) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



COMMENT ON POLICY "Device access: API keys, herd devices, and users with view role" ON "public"."devices" IS 'RLS policy for device table allowing:
1. Self-access: Device can see its own details
2. Herd-access: Device can see other devices in the same herd
3. User-access: Users with view role can see devices in their herds
OPTIMIZED: auth.uid() wrapped in subqueries for better performance';



CREATE POLICY "Event access: Device API keys and users with view role" ON "public"."events" FOR SELECT USING ((("device_id" = "private"."key_uid"()) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("events"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Event creation: Device API keys and users with edit role" ON "public"."events" FOR INSERT WITH CHECK ((("device_id" = "private"."key_uid"()) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("events"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Event deletion: Device API keys and users with edit role" ON "public"."events" FOR DELETE USING ((("device_id" = "private"."key_uid"()) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("events"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Event modification: Device API keys and users with edit role" ON "public"."events" FOR UPDATE USING ((("device_id" = "private"."key_uid"()) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("events"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Heartbeat access: Device API keys and users with view role" ON "public"."heartbeats" FOR SELECT USING ((("device_id" = "private"."key_uid"()) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("heartbeats"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Heartbeat creation: Device API keys and users with edit role" ON "public"."heartbeats" FOR INSERT WITH CHECK ((("device_id" = "private"."key_uid"()) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("heartbeats"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Heartbeat deletion: Device API keys and users with edit role" ON "public"."heartbeats" FOR DELETE USING ((("device_id" = "private"."key_uid"()) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("heartbeats"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Heartbeat modification: Device API keys and users with edit rol" ON "public"."heartbeats" FOR UPDATE USING ((("device_id" = "private"."key_uid"()) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("heartbeats"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Herd access: API keys and users with view role" ON "public"."herds" FOR SELECT USING ((("id" IN ( SELECT "devices"."herd_id"
   FROM "public"."devices"
  WHERE ("devices"."id" = "private"."key_uid"()))) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "id")));



CREATE POLICY "Individuals can delete their own profile" ON "public"."users" FOR DELETE USING ((( SELECT "auth"."uid"() AS "uid") = "id"));



CREATE POLICY "Individuals can insert herds" ON "public"."herds" FOR INSERT WITH CHECK ((( SELECT "auth"."uid"() AS "uid") = "created_by"));



CREATE POLICY "Individuals can insert their own profile" ON "public"."users" FOR INSERT WITH CHECK ((( SELECT "auth"."uid"() AS "uid") = "id"));



CREATE POLICY "Individuals can update their own profile" ON "public"."users" FOR UPDATE USING ((( SELECT "auth"."uid"() AS "uid") = "id"));



CREATE POLICY "Operator access: Device API keys and users with view role" ON "public"."operators" FOR SELECT USING ((("session_id" IN ( SELECT "s"."id"
   FROM "public"."sessions" "s"
  WHERE ("s"."device_id" = "private"."key_uid"()))) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "private"."get_herd_id_by_session_id"("session_id"))));



CREATE POLICY "Operator creation: Device API keys and users with edit role" ON "public"."operators" FOR INSERT WITH CHECK ((("session_id" IN ( SELECT "s"."id"
   FROM "public"."sessions" "s"
  WHERE ("s"."device_id" = "private"."key_uid"()))) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "private"."get_herd_id_by_session_id"("session_id"))));



CREATE POLICY "Operator deletion: Device API keys and users with edit role" ON "public"."operators" FOR DELETE USING ((("session_id" IN ( SELECT "s"."id"
   FROM "public"."sessions" "s"
  WHERE ("s"."device_id" = "private"."key_uid"()))) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "private"."get_herd_id_by_session_id"("session_id"))));



CREATE POLICY "Operator modification: Device API keys and users with edit role" ON "public"."operators" FOR UPDATE USING ((("session_id" IN ( SELECT "s"."id"
   FROM "public"."sessions" "s"
  WHERE ("s"."device_id" = "private"."key_uid"()))) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "private"."get_herd_id_by_session_id"("session_id"))));



CREATE POLICY "Parts create: Admins or associated device only" ON "public"."parts" FOR INSERT WITH CHECK ((("device_id" = "private"."key_uid"()) OR "private"."has_good_admin_role"("auth"."uid"(), "private"."get_herd_id_by_device_id"("device_id"))));



CREATE POLICY "Parts delete: Admins or associated device only" ON "public"."parts" FOR DELETE USING ((("device_id" = "private"."key_uid"()) OR "private"."has_good_admin_role"("auth"."uid"(), "private"."get_herd_id_by_device_id"("device_id"))));



CREATE POLICY "Parts read: Device API keys in same herd or users with view rol" ON "public"."parts" FOR SELECT USING (((("private"."key_uid"() <> 0) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR ("device_id" = "private"."key_uid"()) OR "private"."has_good_view_role"("auth"."uid"(), "private"."get_herd_id_by_device_id"("device_id"))));



CREATE POLICY "Parts update: Admins or associated device only" ON "public"."parts" FOR UPDATE USING ((("device_id" = "private"."key_uid"()) OR "private"."has_good_admin_role"("auth"."uid"(), "private"."get_herd_id_by_device_id"("device_id"))));



CREATE POLICY "Pin access: Users with view role can see pins in their herds" ON "public"."pins" FOR SELECT USING ("private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



COMMENT ON POLICY "Pin access: Users with view role can see pins in their herds" ON "public"."pins" IS 'Allows users with view role to see all pins in herds they have access to';



CREATE POLICY "Pin creation: Users can create pins for themselves" ON "public"."pins" FOR INSERT WITH CHECK ((("created_by" = ( SELECT "auth"."uid"() AS "uid")) AND "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



COMMENT ON POLICY "Pin creation: Users can create pins for themselves" ON "public"."pins" IS 'Allows users to create pins only for herds they have edit access to, and only with their own user ID as created_by';



CREATE POLICY "Pin deletion: Users with edit role can delete pins" ON "public"."pins" FOR DELETE USING ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



COMMENT ON POLICY "Pin deletion: Users with edit role can delete pins" ON "public"."pins" IS 'Allows users with edit role to delete pins in herds they have access to';



CREATE POLICY "Pin modification: Users with edit role can update pins" ON "public"."pins" FOR UPDATE USING ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")) WITH CHECK ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



COMMENT ON POLICY "Pin modification: Users with edit role can update pins" ON "public"."pins" IS 'Allows users with edit role to modify pins in herds they have access to';



CREATE POLICY "Plan access: Device API keys and users with view role" ON "public"."plans" FOR SELECT USING (("private"."herd_has_device"("herd_id", ( SELECT "private"."key_uid"() AS "key_uid")) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Plan creation: Device API keys and users with edit role" ON "public"."plans" FOR INSERT WITH CHECK (("private"."herd_has_device"("herd_id", ( SELECT "private"."key_uid"() AS "key_uid")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Plan deletion: Device API keys and users with edit role" ON "public"."plans" FOR DELETE USING (("private"."herd_has_device"("herd_id", ( SELECT "private"."key_uid"() AS "key_uid")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Plan modification: Device API keys and users with edit role" ON "public"."plans" FOR UPDATE USING (("private"."herd_has_device"("herd_id", ( SELECT "private"."key_uid"() AS "key_uid")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Provider access: Device API keys and users with view role" ON "public"."providers" FOR SELECT USING (("private"."herd_has_device"("herd_id", ( SELECT "private"."key_uid"() AS "key_uid")) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Provider creation: Users with edit role" ON "public"."providers" FOR INSERT WITH CHECK ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Provider deletion: Users with edit role" ON "public"."providers" FOR DELETE USING ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Provider modification: Users with edit role" ON "public"."providers" FOR UPDATE USING ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Role access: Herd creators, role users, and individuals" ON "public"."users_roles_per_herd" FOR SELECT TO "authenticated" USING (("private"."is_herd_creator"(( SELECT "auth"."uid"() AS "uid"), "herd_id") OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id") OR "private"."has_good_admin_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id") OR (( SELECT "auth"."uid"() AS "uid") = "user_id")));



CREATE POLICY "Role creation: Herd creators, edit role, and admin role users" ON "public"."users_roles_per_herd" FOR INSERT TO "authenticated" WITH CHECK (("private"."is_herd_creator"(( SELECT "auth"."uid"() AS "uid"), "herd_id") OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id") OR "private"."has_good_admin_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Role deletion: Herd creators, edit role, and admin role users" ON "public"."users_roles_per_herd" FOR DELETE TO "authenticated" USING (("private"."is_herd_creator"(( SELECT "auth"."uid"() AS "uid"), "herd_id") OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id") OR "private"."has_good_admin_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Role modification: Herd creators, edit role, and admin role use" ON "public"."users_roles_per_herd" FOR UPDATE TO "authenticated" USING (("private"."is_herd_creator"(( SELECT "auth"."uid"() AS "uid"), "herd_id") OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id") OR "private"."has_good_admin_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Session access: Device API keys and users with view role" ON "public"."sessions" FOR SELECT USING ((("device_id" = "private"."key_uid"()) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("sessions"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Session creation: Device API keys and users with edit role" ON "public"."sessions" FOR INSERT WITH CHECK ((("device_id" = "private"."key_uid"()) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("sessions"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Session deletion: Device API keys and users with edit role" ON "public"."sessions" FOR DELETE USING ((("device_id" = "private"."key_uid"()) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("sessions"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Session modification: Device API keys and users with edit role" ON "public"."sessions" FOR UPDATE USING ((("device_id" = "private"."key_uid"()) OR (("private"."key_uid"() <> 0) AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_device_id"("sessions"."device_id") AS "get_herd_id_by_device_id"))));



CREATE POLICY "Software versions creation: Admin users only" ON "public"."versions_software" FOR INSERT TO "authenticated" WITH CHECK ("private"."has_admin_role_any_herd"(( SELECT "auth"."uid"() AS "uid")));



COMMENT ON POLICY "Software versions creation: Admin users only" ON "public"."versions_software" IS 'Restricts software version creation to users with admin role in any herd';



CREATE POLICY "Software versions deletion: Admin users only" ON "public"."versions_software" FOR DELETE TO "authenticated" USING ("private"."has_admin_role_any_herd"(( SELECT "auth"."uid"() AS "uid")));



COMMENT ON POLICY "Software versions deletion: Admin users only" ON "public"."versions_software" IS 'Restricts software version deletion to users with admin role in any herd';



CREATE POLICY "Software versions modification: Admin users only" ON "public"."versions_software" FOR UPDATE TO "authenticated" USING ("private"."has_admin_role_any_herd"(( SELECT "auth"."uid"() AS "uid"))) WITH CHECK ("private"."has_admin_role_any_herd"(( SELECT "auth"."uid"() AS "uid")));



COMMENT ON POLICY "Software versions modification: Admin users only" ON "public"."versions_software" IS 'Restricts software version updates to users with admin role in any herd';



CREATE POLICY "Software versions view: All authenticated users" ON "public"."versions_software" FOR SELECT TO "authenticated" USING (true);



COMMENT ON POLICY "Software versions view: All authenticated users" ON "public"."versions_software" IS 'Allows all authenticated users to view software version information';



CREATE POLICY "Tag access: Device API keys and users with view role" ON "public"."tags" FOR SELECT USING (("private"."is_device_authorized_for_event"("event_id", "private"."key_uid"()) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_event_id"("tags"."event_id") AS "get_herd_id_by_event_id"))));



CREATE POLICY "Tag creation: Device API keys and users with edit role" ON "public"."tags" FOR INSERT WITH CHECK (("private"."is_device_authorized_for_event"("event_id", "private"."key_uid"()) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_event_id"("tags"."event_id") AS "get_herd_id_by_event_id"))));



CREATE POLICY "Tag deletion: Device API keys and users with edit role" ON "public"."tags" FOR DELETE USING (("private"."is_device_authorized_for_event"("event_id", "private"."key_uid"()) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_event_id"("tags"."event_id") AS "get_herd_id_by_event_id"))));



CREATE POLICY "Tag modification: Device API keys and users with edit role" ON "public"."tags" FOR UPDATE USING (("private"."is_device_authorized_for_event"("event_id", "private"."key_uid"()) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), ( SELECT "private"."get_herd_id_by_event_id"("tags"."event_id") AS "get_herd_id_by_event_id"))));



CREATE POLICY "Those with good admin role can delete herds" ON "public"."herds" FOR DELETE USING ("private"."has_good_admin_role"(( SELECT "auth"."uid"() AS "uid"), "id"));



CREATE POLICY "Those with good admin role can update herds" ON "public"."herds" FOR UPDATE USING ("private"."has_good_admin_role"(( SELECT "auth"."uid"() AS "uid"), "id"));



CREATE POLICY "Those with good edit role can delete devices" ON "public"."devices" FOR DELETE USING ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Those with good edit role can delete layers" ON "public"."layers" FOR DELETE USING ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Those with good edit role can insert devices" ON "public"."devices" FOR INSERT WITH CHECK ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Those with good edit role can insert layers" ON "public"."layers" FOR INSERT WITH CHECK ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Those with good edit role can update devices" ON "public"."devices" FOR UPDATE USING ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Those with good edit role can update layers" ON "public"."layers" FOR UPDATE USING ("private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Those with good view role can view layers" ON "public"."layers" FOR SELECT USING ("private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id"));



CREATE POLICY "Zone access: Device API keys and users with view role" ON "public"."zones" FOR SELECT USING ((("herd_id" IN ( SELECT "devices"."herd_id"
   FROM "public"."devices"
  WHERE ("devices"."id" = "private"."key_uid"()))) OR "private"."has_good_view_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Zone creation: Device API keys and users with edit role" ON "public"."zones" FOR INSERT WITH CHECK ((("herd_id" IN ( SELECT "devices"."herd_id"
   FROM "public"."devices"
  WHERE ("devices"."id" = "private"."key_uid"()))) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Zone deletion: Device API keys and users with edit role" ON "public"."zones" FOR DELETE USING ((("herd_id" IN ( SELECT "devices"."herd_id"
   FROM "public"."devices"
  WHERE ("devices"."id" = "private"."key_uid"()))) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



CREATE POLICY "Zone modification: Device API keys and users with edit role" ON "public"."zones" FOR UPDATE USING ((("herd_id" IN ( SELECT "devices"."herd_id"
   FROM "public"."devices"
  WHERE ("devices"."id" = "private"."key_uid"()))) OR "private"."has_good_edit_role"(( SELECT "auth"."uid"() AS "uid"), "herd_id")));



ALTER TABLE "public"."actions" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."artifacts" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."certificates" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."chat" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."connectivity" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."devices" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."events" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."heartbeats" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."herds" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."layers" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."operators" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."parts" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."pins" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."plans" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."providers" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."sessions" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."tags" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."users" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."users_roles_per_herd" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."versions_software" ENABLE ROW LEVEL SECURITY;


ALTER TABLE "public"."zones" ENABLE ROW LEVEL SECURITY;




ALTER PUBLICATION "supabase_realtime" OWNER TO "postgres";






ALTER PUBLICATION "supabase_realtime" ADD TABLE ONLY "public"."actions";



ALTER PUBLICATION "supabase_realtime" ADD TABLE ONLY "public"."artifacts";



ALTER PUBLICATION "supabase_realtime" ADD TABLE ONLY "public"."chat";



ALTER PUBLICATION "supabase_realtime" ADD TABLE ONLY "public"."devices";



ALTER PUBLICATION "supabase_realtime" ADD TABLE ONLY "public"."plans";



ALTER PUBLICATION "supabase_realtime" ADD TABLE ONLY "public"."users_roles_per_herd";



ALTER PUBLICATION "supabase_realtime" ADD TABLE ONLY "public"."zones";






GRANT USAGE ON SCHEMA "private" TO "authenticated";



GRANT USAGE ON SCHEMA "public" TO "postgres";
GRANT USAGE ON SCHEMA "public" TO "anon";
GRANT USAGE ON SCHEMA "public" TO "authenticated";
GRANT USAGE ON SCHEMA "public" TO "service_role";
GRANT USAGE ON SCHEMA "public" TO "supabase_auth_admin";











































































GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."tags" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."tags" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."tags" TO "service_role";



GRANT ALL ON TYPE "public"."tags_pretty_location" TO "anon";
GRANT ALL ON TYPE "public"."tags_pretty_location" TO "authenticated";
GRANT ALL ON TYPE "public"."tags_pretty_location" TO "service_role";



GRANT ALL ON TYPE "public"."event_and_tags_pretty_location" TO "anon";
GRANT ALL ON TYPE "public"."event_and_tags_pretty_location" TO "authenticated";
GRANT ALL ON TYPE "public"."event_and_tags_pretty_location" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."actions" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."actions" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."actions" TO "service_role";























































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































GRANT ALL ON FUNCTION "private"."get_device_id_from_path"("object_name" "text") TO "authenticated";
GRANT ALL ON FUNCTION "private"."get_device_id_from_path"("object_name" "text") TO "anon";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."herds" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."herds" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."herds" TO "service_role";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."herds" TO "supabase_auth_admin";



GRANT ALL ON FUNCTION "private"."get_herd_id_from_path"("object_name" "text") TO "authenticated";
GRANT ALL ON FUNCTION "private"."get_herd_id_from_path"("object_name" "text") TO "anon";



GRANT ALL ON FUNCTION "public"."add_user_as_herd_admin"() TO "anon";
GRANT ALL ON FUNCTION "public"."add_user_as_herd_admin"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."add_user_as_herd_admin"() TO "service_role";



REVOKE ALL ON FUNCTION "public"."analyze_device_heartbeats"("p_device_id" bigint, "p_lookback_minutes" integer, "p_window_minutes" integer) FROM PUBLIC;
GRANT ALL ON FUNCTION "public"."analyze_device_heartbeats"("p_device_id" bigint, "p_lookback_minutes" integer, "p_window_minutes" integer) TO "anon";
GRANT ALL ON FUNCTION "public"."analyze_device_heartbeats"("p_device_id" bigint, "p_lookback_minutes" integer, "p_window_minutes" integer) TO "authenticated";
GRANT ALL ON FUNCTION "public"."analyze_device_heartbeats"("p_device_id" bigint, "p_lookback_minutes" integer, "p_window_minutes" integer) TO "service_role";



REVOKE ALL ON FUNCTION "public"."analyze_herd_device_heartbeats"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) FROM PUBLIC;
GRANT ALL ON FUNCTION "public"."analyze_herd_device_heartbeats"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) TO "anon";
GRANT ALL ON FUNCTION "public"."analyze_herd_device_heartbeats"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) TO "authenticated";
GRANT ALL ON FUNCTION "public"."analyze_herd_device_heartbeats"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) TO "service_role";



GRANT ALL ON FUNCTION "public"."broadcast_connectivity_changes"() TO "anon";
GRANT ALL ON FUNCTION "public"."broadcast_connectivity_changes"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."broadcast_connectivity_changes"() TO "service_role";



GRANT ALL ON FUNCTION "public"."broadcast_device_changes"() TO "anon";
GRANT ALL ON FUNCTION "public"."broadcast_device_changes"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."broadcast_device_changes"() TO "service_role";



GRANT ALL ON FUNCTION "public"."broadcast_events_changes"() TO "anon";
GRANT ALL ON FUNCTION "public"."broadcast_events_changes"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."broadcast_events_changes"() TO "service_role";



GRANT ALL ON FUNCTION "public"."broadcast_pins_changes"() TO "anon";
GRANT ALL ON FUNCTION "public"."broadcast_pins_changes"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."broadcast_pins_changes"() TO "service_role";



GRANT ALL ON FUNCTION "public"."broadcast_plans_changes"() TO "anon";
GRANT ALL ON FUNCTION "public"."broadcast_plans_changes"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."broadcast_plans_changes"() TO "service_role";



GRANT ALL ON FUNCTION "public"."broadcast_sessions_changes"() TO "anon";
GRANT ALL ON FUNCTION "public"."broadcast_sessions_changes"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."broadcast_sessions_changes"() TO "service_role";



GRANT ALL ON FUNCTION "public"."broadcast_tags_changes"() TO "anon";
GRANT ALL ON FUNCTION "public"."broadcast_tags_changes"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."broadcast_tags_changes"() TO "service_role";



GRANT ALL ON FUNCTION "public"."broadcast_versions_software_changes"() TO "anon";
GRANT ALL ON FUNCTION "public"."broadcast_versions_software_changes"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."broadcast_versions_software_changes"() TO "service_role";



GRANT ALL ON FUNCTION "public"."check_realtime_schema_status"() TO "anon";
GRANT ALL ON FUNCTION "public"."check_realtime_schema_status"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."check_realtime_schema_status"() TO "service_role";



GRANT ALL ON FUNCTION "public"."check_stable_pre_exclusivity"() TO "anon";
GRANT ALL ON FUNCTION "public"."check_stable_pre_exclusivity"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."check_stable_pre_exclusivity"() TO "service_role";



GRANT ALL ON FUNCTION "public"."create_user_api_key_secret"() TO "anon";
GRANT ALL ON FUNCTION "public"."create_user_api_key_secret"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."create_user_api_key_secret"() TO "service_role";



REVOKE ALL ON FUNCTION "public"."delete_all_orphaned_sessions"("min_age_seconds" integer) FROM PUBLIC;
GRANT ALL ON FUNCTION "public"."delete_all_orphaned_sessions"("min_age_seconds" integer) TO "anon";
GRANT ALL ON FUNCTION "public"."delete_all_orphaned_sessions"("min_age_seconds" integer) TO "authenticated";
GRANT ALL ON FUNCTION "public"."delete_all_orphaned_sessions"("min_age_seconds" integer) TO "service_role";



REVOKE ALL ON FUNCTION "public"."delete_orphaned_session"("session_id_param" bigint, "min_age_seconds" integer) FROM PUBLIC;
GRANT ALL ON FUNCTION "public"."delete_orphaned_session"("session_id_param" bigint, "min_age_seconds" integer) TO "anon";
GRANT ALL ON FUNCTION "public"."delete_orphaned_session"("session_id_param" bigint, "min_age_seconds" integer) TO "authenticated";
GRANT ALL ON FUNCTION "public"."delete_orphaned_session"("session_id_param" bigint, "min_age_seconds" integer) TO "service_role";



GRANT ALL ON FUNCTION "public"."enforce_single_min_per_system"() TO "anon";
GRANT ALL ON FUNCTION "public"."enforce_single_min_per_system"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."enforce_single_min_per_system"() TO "service_role";



GRANT ALL ON FUNCTION "public"."enforce_single_stable_per_system"() TO "anon";
GRANT ALL ON FUNCTION "public"."enforce_single_stable_per_system"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."enforce_single_stable_per_system"() TO "service_role";



REVOKE ALL ON FUNCTION "public"."fix_all_sessions_missing_end_timestamps"() FROM PUBLIC;
GRANT ALL ON FUNCTION "public"."fix_all_sessions_missing_end_timestamps"() TO "anon";
GRANT ALL ON FUNCTION "public"."fix_all_sessions_missing_end_timestamps"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."fix_all_sessions_missing_end_timestamps"() TO "service_role";



REVOKE ALL ON FUNCTION "public"."fix_session_end_timestamp"("session_id_param" bigint) FROM PUBLIC;
GRANT ALL ON FUNCTION "public"."fix_session_end_timestamp"("session_id_param" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."fix_session_end_timestamp"("session_id_param" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."fix_session_end_timestamp"("session_id_param" bigint) TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."artifacts" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."artifacts" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."artifacts" TO "service_role";



GRANT ALL ON FUNCTION "public"."get_artifacts_for_device"("device_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_artifacts_for_device"("device_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_artifacts_for_device"("device_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_artifacts_for_devices_batch"("device_ids" bigint[], "limit_per_device" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_artifacts_for_devices_batch"("device_ids" bigint[], "limit_per_device" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_artifacts_for_devices_batch"("device_ids" bigint[], "limit_per_device" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_artifacts_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_artifacts_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_artifacts_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_artifacts_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_artifacts_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_artifacts_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_artifacts_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_artifacts_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_artifacts_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_connectivity_with_coordinates"("session_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_connectivity_with_coordinates"("session_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_connectivity_with_coordinates"("session_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_connectivity_with_coordinates_by_device_and_timestamp"("device_id_caller" bigint, "timestamp_filter" timestamp with time zone) TO "anon";
GRANT ALL ON FUNCTION "public"."get_connectivity_with_coordinates_by_device_and_timestamp"("device_id_caller" bigint, "timestamp_filter" timestamp with time zone) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_connectivity_with_coordinates_by_device_and_timestamp"("device_id_caller" bigint, "timestamp_filter" timestamp with time zone) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_device_by_api_key"("device_api_key" "text") TO "anon";
GRANT ALL ON FUNCTION "public"."get_device_by_api_key"("device_api_key" "text") TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_device_by_api_key"("device_api_key" "text") TO "service_role";



GRANT ALL ON FUNCTION "public"."get_device_by_id"("device_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_device_by_id"("device_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_device_by_id"("device_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_devices_for_herd"("herd_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_devices_for_herd"("herd_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_devices_for_herd"("herd_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_device"("device_id_caller" bigint, "limit_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_device"("device_id_caller" bigint, "limit_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_device"("device_id_caller" bigint, "limit_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_devices_batch"("device_ids" bigint[], "limit_per_device" integer) TO "anon";
GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_devices_batch"("device_ids" bigint[], "limit_per_device" integer) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_devices_batch"("device_ids" bigint[], "limit_per_device" integer) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_session"("session_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_session"("session_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_events_and_tags_for_session"("session_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_events_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_events_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_events_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_events_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_events_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_events_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_events_with_tags_for_herd"("herd_id_caller" bigint, "offset_caller" bigint, "limit_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_events_with_tags_for_herd"("herd_id_caller" bigint, "offset_caller" bigint, "limit_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_events_with_tags_for_herd"("herd_id_caller" bigint, "offset_caller" bigint, "limit_caller" bigint) TO "service_role";



REVOKE ALL ON FUNCTION "public"."get_herd_uptime_summary"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) FROM PUBLIC;
GRANT ALL ON FUNCTION "public"."get_herd_uptime_summary"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) TO "anon";
GRANT ALL ON FUNCTION "public"."get_herd_uptime_summary"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_herd_uptime_summary"("p_herd_id" bigint, "p_device_types" "public"."device_type"[], "p_lookback_minutes" integer, "p_window_minutes" integer) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_pins_for_herd"("herd_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_pins_for_herd"("herd_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_pins_for_herd"("herd_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_session_by_id"("session_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_session_by_id"("session_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_session_by_id"("session_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_session_summaries"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_session_summaries"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_session_summaries"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_session_usage_over_time"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_session_usage_over_time"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_session_usage_over_time"("start_date_caller" "date", "end_date_caller" "date", "device_id_caller" bigint, "herd_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_sessions_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_sessions_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_sessions_infinite_by_device"("device_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_sessions_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_sessions_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_sessions_infinite_by_herd"("herd_id_caller" bigint, "limit_caller" integer, "cursor_timestamp" timestamp with time zone, "cursor_id" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_sessions_with_coordinates"("herd_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_sessions_with_coordinates"("herd_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_sessions_with_coordinates"("herd_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_sessions_with_coordinates_by_device"("device_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_sessions_with_coordinates_by_device"("device_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_sessions_with_coordinates_by_device"("device_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_total_artifacts_for_herd"("herd_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_total_artifacts_for_herd"("herd_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_total_artifacts_for_herd"("herd_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_total_events_for_herd_with_session_filter"("herd_id_caller" bigint, "exclude_session_events" boolean) TO "anon";
GRANT ALL ON FUNCTION "public"."get_total_events_for_herd_with_session_filter"("herd_id_caller" bigint, "exclude_session_events" boolean) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_total_events_for_herd_with_session_filter"("herd_id_caller" bigint, "exclude_session_events" boolean) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_total_events_for_session"("session_id_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_total_events_for_session"("session_id_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_total_events_for_session"("session_id_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."get_zones_and_actions_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."get_zones_and_actions_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."get_zones_and_actions_for_herd"("herd_id_caller" bigint, "limit_caller" bigint, "offset_caller" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."handle_new_user"() TO "anon";
GRANT ALL ON FUNCTION "public"."handle_new_user"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."handle_new_user"() TO "service_role";



GRANT ALL ON FUNCTION "public"."load_api_keys"("id_of_device" bigint) TO "anon";
GRANT ALL ON FUNCTION "public"."load_api_keys"("id_of_device" bigint) TO "authenticated";
GRANT ALL ON FUNCTION "public"."load_api_keys"("id_of_device" bigint) TO "service_role";



GRANT ALL ON FUNCTION "public"."load_api_keys"("id_of_device" "text") TO "anon";
GRANT ALL ON FUNCTION "public"."load_api_keys"("id_of_device" "text") TO "authenticated";
GRANT ALL ON FUNCTION "public"."load_api_keys"("id_of_device" "text") TO "service_role";



GRANT ALL ON FUNCTION "public"."load_api_keys_batch"("device_ids" bigint[]) TO "anon";
GRANT ALL ON FUNCTION "public"."load_api_keys_batch"("device_ids" bigint[]) TO "authenticated";
GRANT ALL ON FUNCTION "public"."load_api_keys_batch"("device_ids" bigint[]) TO "service_role";



GRANT ALL ON FUNCTION "public"."load_api_keys_old"("id_of_device" "text") TO "anon";
GRANT ALL ON FUNCTION "public"."load_api_keys_old"("id_of_device" "text") TO "authenticated";
GRANT ALL ON FUNCTION "public"."load_api_keys_old"("id_of_device" "text") TO "service_role";



GRANT ALL ON FUNCTION "public"."remove_rls_broadcast_triggers"() TO "anon";
GRANT ALL ON FUNCTION "public"."remove_rls_broadcast_triggers"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."remove_rls_broadcast_triggers"() TO "service_role";



GRANT ALL ON FUNCTION "public"."remove_user_vault_secrets"() TO "anon";
GRANT ALL ON FUNCTION "public"."remove_user_vault_secrets"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."remove_user_vault_secrets"() TO "service_role";



GRANT ALL ON FUNCTION "public"."test_connectivity_before_trigger"() TO "anon";
GRANT ALL ON FUNCTION "public"."test_connectivity_before_trigger"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."test_connectivity_before_trigger"() TO "service_role";



GRANT ALL ON FUNCTION "public"."test_connectivity_trigger"() TO "anon";
GRANT ALL ON FUNCTION "public"."test_connectivity_trigger"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."test_connectivity_trigger"() TO "service_role";



GRANT ALL ON FUNCTION "public"."test_connectivity_trigger_bypass_rls"() TO "anon";
GRANT ALL ON FUNCTION "public"."test_connectivity_trigger_bypass_rls"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."test_connectivity_trigger_bypass_rls"() TO "service_role";



GRANT ALL ON FUNCTION "public"."update_updated_at_column"() TO "anon";
GRANT ALL ON FUNCTION "public"."update_updated_at_column"() TO "authenticated";
GRANT ALL ON FUNCTION "public"."update_updated_at_column"() TO "service_role";
































































































GRANT ALL ON SEQUENCE "public"."actions_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."actions_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."actions_id_seq" TO "service_role";



GRANT ALL ON SEQUENCE "public"."artifacts_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."artifacts_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."artifacts_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."certificates" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."certificates" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."certificates" TO "service_role";



GRANT ALL ON SEQUENCE "public"."certificates_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."certificates_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."certificates_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."chat" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."chat" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."chat" TO "service_role";



GRANT ALL ON SEQUENCE "public"."chat_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."chat_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."chat_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."connectivity" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."connectivity" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."connectivity" TO "service_role";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."connectivity" TO "supabase_auth_admin";



GRANT ALL ON SEQUENCE "public"."connectivity_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."connectivity_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."connectivity_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."devices" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."devices" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."devices" TO "service_role";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."devices" TO "supabase_auth_admin";



GRANT ALL ON SEQUENCE "public"."devices_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."devices_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."devices_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events" TO "service_role";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events" TO "supabase_auth_admin";



GRANT ALL ON SEQUENCE "public"."events_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."events_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."events_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events_with_tags" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events_with_tags" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events_with_tags" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events_with_tags_by_session" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events_with_tags_by_session" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."events_with_tags_by_session" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."heartbeats" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."heartbeats" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."heartbeats" TO "service_role";



GRANT ALL ON SEQUENCE "public"."heartbeats_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."heartbeats_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."heartbeats_id_seq" TO "service_role";



GRANT ALL ON SEQUENCE "public"."herds_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."herds_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."herds_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."layers" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."layers" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."layers" TO "service_role";



GRANT ALL ON SEQUENCE "public"."layers_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."layers_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."layers_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."operators" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."operators" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."operators" TO "service_role";



GRANT ALL ON SEQUENCE "public"."operators_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."operators_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."operators_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."parts" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."parts" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."parts" TO "service_role";



GRANT ALL ON SEQUENCE "public"."parts_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."parts_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."parts_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."pins" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."pins" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."pins" TO "service_role";



GRANT ALL ON SEQUENCE "public"."pins_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."pins_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."pins_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."plans" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."plans" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."plans" TO "service_role";



GRANT ALL ON SEQUENCE "public"."plans_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."plans_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."plans_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."providers" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."providers" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."providers" TO "service_role";



GRANT ALL ON SEQUENCE "public"."providers_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."providers_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."providers_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."sessions" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."sessions" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."sessions" TO "service_role";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."sessions" TO "supabase_auth_admin";



GRANT ALL ON SEQUENCE "public"."sessions_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."sessions_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."sessions_id_seq" TO "service_role";



GRANT ALL ON SEQUENCE "public"."tags_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."tags_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."tags_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."users" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."users" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."users" TO "service_role";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."users" TO "supabase_auth_admin";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."users_roles_per_herd" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."users_roles_per_herd" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."users_roles_per_herd" TO "service_role";



GRANT ALL ON SEQUENCE "public"."users_roles_per_herd_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."users_roles_per_herd_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."users_roles_per_herd_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."versions_software" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."versions_software" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."versions_software" TO "service_role";



GRANT ALL ON SEQUENCE "public"."versions_software_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."versions_software_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."versions_software_id_seq" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."zones" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."zones" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."zones" TO "service_role";



GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."zones_and_actions" TO "anon";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."zones_and_actions" TO "authenticated";
GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLE "public"."zones_and_actions" TO "service_role";



GRANT ALL ON SEQUENCE "public"."zones_id_seq" TO "anon";
GRANT ALL ON SEQUENCE "public"."zones_id_seq" TO "authenticated";
GRANT ALL ON SEQUENCE "public"."zones_id_seq" TO "service_role";









ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON SEQUENCES TO "postgres";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON SEQUENCES TO "anon";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON SEQUENCES TO "authenticated";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON SEQUENCES TO "service_role";






ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON FUNCTIONS TO "postgres";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON FUNCTIONS TO "anon";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON FUNCTIONS TO "authenticated";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT ALL ON FUNCTIONS TO "service_role";






ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLES TO "postgres";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLES TO "anon";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLES TO "authenticated";
ALTER DEFAULT PRIVILEGES FOR ROLE "postgres" IN SCHEMA "public" GRANT SELECT,INSERT,REFERENCES,DELETE,TRIGGER,TRUNCATE,UPDATE ON TABLES TO "service_role";































