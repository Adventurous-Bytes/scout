"use server";

import { newServerClient } from "../supabase/server";
import { ILayer } from "../types/db";
import {
  IWebResponseCompatible,
  EnumWebResponse,
  IWebResponse,
} from "../types/requests";

// function that fetches the layers from our db given a herd id
export async function server_get_layers_by_herd(
  herd_id: number
): Promise<IWebResponseCompatible<ILayer[]>> {
  const supabase = await newServerClient();
  const { data, error } = await supabase
    .from("layers")
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
