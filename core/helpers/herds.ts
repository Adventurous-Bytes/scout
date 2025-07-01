"use server";
import { newServerClient } from "../supabase/server";
import { IHerd } from "../types/db";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";
import { HerdModule, IHerdModule } from "../types/herd_module";

export async function get_herds(
  client: SupabaseClient
): Promise<IWebResponseCompatible<IHerd[]>> {
  const { data: herds } = await client.from("herds").select();
  if (!herds) {
    return {
      status: EnumWebResponse.ERROR,
      msg: "No herds found",
      data: null,
    };
  } else {
    let response: IWebResponse<IHerd[]> = IWebResponse.success(herds);
    return response.to_compatible();
  }
}
export async function get_herd_by_slug(
  slug: string
): Promise<IWebResponseCompatible<IHerd>> {
  const supabase = await newServerClient();
  const { data: herds } = await supabase
    .from("herds")
    .select()
    .eq("slug", slug)
    .limit(1);
  if (!herds) {
    return {
      status: EnumWebResponse.ERROR,
      msg: "No herds found",
      data: null,
    };
  }
  return IWebResponse.success(herds[0]);
}

export async function deleteHerd(
  herd_id: number
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();

  const { error } = await supabase.from("herds").delete().match({ id: 20 });
  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  } else {
    return IWebResponse.success(true).to_compatible();
  }
}

export async function createHerd(
  newHerd: any
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();
  const user = await supabase.auth.getUser();
  const userId = user?.data?.user?.id;
  newHerd.created_by = userId;

  // strip id field from herd object
  const { data, error } = await supabase.from("herds").insert([newHerd]);

  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  } else {
    return IWebResponse.success(true).to_compatible();
  }
}

export async function server_load_herd_modules(): Promise<IHerdModule[]> {
  // load herds
  const client_supabase = await newServerClient();
  let new_herds = await get_herds(client_supabase);
  if (new_herds.status != EnumWebResponse.SUCCESS || !new_herds.data) {
    return [];
  }
  let new_herd_modules: HerdModule[] = [];
  const herdModulePromises = new_herds.data.map((herd) =>
    HerdModule.from_herd(herd, client_supabase)
  );
  new_herd_modules = await Promise.all(herdModulePromises);
  // now serialize the herd modules
  let serialized_herd_modules: IHerdModule[] = new_herd_modules.map(
    (herd_module) => herd_module.to_serializable()
  );
  return serialized_herd_modules;
}
