"use server";

import { newServerClient } from "../supabase/server";
import { IUser, IUserAndRole, IUserRolePerHerd, Role } from "../types/db";
import {
  IWebResponseCompatible,
  IWebResponse,
  EnumWebResponse,
} from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";

export async function server_get_user_roles(
  herd_id: number
): Promise<IWebResponseCompatible<IUserRolePerHerd[]>> {
  const supabase = await newServerClient();
  // fetch user role for herd
  const { data, error } = await supabase
    .from("users_roles_per_herd")
    .select("*")
    .eq("herd_id", herd_id);
  if (!data) {
    return {
      status: EnumWebResponse.ERROR,
      msg: `No user role found for herd ${herd_id}`,
      data: null,
    };
  }
  // TODO: DETERMINE WHEN TO PASS ERROR
  let response: IWebResponse<IUserRolePerHerd[]> = IWebResponse.success(data);
  return response.to_compatible();
}

export async function server_get_user(): Promise<
  IWebResponseCompatible<IUser | null>
> {
  const supabase = await newServerClient();

  const { data } = await supabase.auth.getUser();
  const new_user: IUser | null = data.user;
  return IWebResponse.success(new_user).to_compatible();
}

export async function server_get_users_with_herd_access(
  herd_id: number,
  supabaseClient?: SupabaseClient
): Promise<IWebResponseCompatible<IUserAndRole[]>> {
  const supabase = supabaseClient || (await newServerClient());

  const { data, error } = await supabase
    .from("users_roles_per_herd")
    .select(
      `
      role,
      users (
        id,
        username
      )
    `
    )
    .eq("herd_id", herd_id);

  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  } else {
    // Transform the data to match IUserAndRole interface
    const transformedData: IUserAndRole[] = data.map((item) => ({
      user: item.users,
      role: item.role,
    }));

    return IWebResponse.success(transformedData).to_compatible();
  }
}

export async function server_upsert_user_with_role(
  herd_id: number,
  username: string,
  role: Role
): Promise<IWebResponseCompatible<IUserAndRole | null>> {
  const supabase = await newServerClient();
  // first try to get user by username
  const { data: user, error: user_error } = await supabase
    .from("users")
    .select("*")
    .eq("username", username)
    .single();
  if (user_error) {
    return IWebResponse.error<null>(
      "Unable to fetch user with provided username"
    ).to_compatible();
  }
  if (!user) {
    return IWebResponse.error<null>("User not found").to_compatible();
  }
  const newRoleForDb = {
    user_id: user.id,
    herd_id: herd_id,
    role: role,
  };
  // upddate or insert user role
  const { data, error } = await supabase
    .from("users_roles_per_herd")
    .upsert(newRoleForDb);
  if (error) {
    return IWebResponse.error<null>(
      "Unable to update or add user with provided username. Ensure you have sufficient permissions."
    ).to_compatible();
  }
  return IWebResponse.success(data).to_compatible();
}
