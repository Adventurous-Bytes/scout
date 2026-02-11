-- Migration: Add has_view_role_any_herd function
-- Returns true if the user has view role (admin, editor, viewer) in any herd.
-- Use for policies that require "authenticated and part of a herd" (e.g. storage bucket SELECT).

CREATE OR REPLACE FUNCTION "private"."has_view_role_any_herd"("user_id_caller" "uuid")
RETURNS boolean
LANGUAGE "plpgsql"
SECURITY DEFINER
SET "search_path" TO 'extensions'
AS $$
begin
  return exists (
    select 1
    from public.users_roles_per_herd
    where user_id = user_id_caller
    and role in ('admin', 'editor', 'viewer')
  );
end;
$$;

ALTER FUNCTION "private"."has_view_role_any_herd"("user_id_caller" "uuid") OWNER TO "postgres";

COMMENT ON FUNCTION "private"."has_view_role_any_herd"("user_id_caller" "uuid") IS 'Returns true if the user has view role (admin, editor, viewer) in any herd, used for policies that require authenticated users who are part of a herd';
