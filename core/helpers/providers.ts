"use server";

import { newServerClient } from "../supabase/server";
import { IProvider } from "../types/db";
import {
  IWebResponseCompatible,
  EnumWebResponse,
  IWebResponse,
} from "../types/requests";

// function that fetches the providers from our db given a herd id
export async function server_get_providers_by_herd(
  herd_id: number,
): Promise<IWebResponseCompatible<IProvider[]>> {
  const supabase = await newServerClient();
  const { data, error } = await supabase
    .from("providers")
    .select("*")
    .eq("herd_id", herd_id);
  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  } else {
    return IWebResponse.success(data).to_compatible();
  }
}

// function that creates a new provider in our db
export async function server_create_provider(
  provider: Omit<IProvider, "id" | "created_at">,
): Promise<IWebResponseCompatible<IProvider>> {
  const supabase = await newServerClient();
  const { data, error } = await supabase
    .from("providers")
    .insert(provider)
    .select("*")
    .single();
  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  } else {
    return IWebResponse.success(data).to_compatible();
  }
}

// function that updates a provider in our db
export async function server_update_provider(
  provider_id: number,
  updates: Partial<Omit<IProvider, "id" | "created_at">>,
): Promise<IWebResponseCompatible<IProvider>> {
  const supabase = await newServerClient();
  const { data, error } = await supabase
    .from("providers")
    .update(updates)
    .eq("id", provider_id)
    .select("*")
    .single();
  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  } else {
    return IWebResponse.success(data).to_compatible();
  }
}

// function that deletes providers by ids
export async function server_delete_providers_by_ids(
  provider_ids: number[],
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();
  const { error } = await supabase
    .from("providers")
    .delete()
    .in("id", provider_ids);
  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: false,
    };
  } else {
    return IWebResponse.success(true).to_compatible();
  }
}
