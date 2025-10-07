"use server";

import { newServerClient } from "../supabase/server";
import { IPlan } from "../types/db";
import {
  IWebResponseCompatible,
  EnumWebResponse,
  IWebResponse,
} from "../types/requests";

// function that fetches the plans our db given a herd id
export async function server_get_plans_by_herd(
  herd_id: number,
): Promise<IWebResponseCompatible<IPlan[]>> {
  const supabase = await newServerClient();
  const { data, error } = await supabase
    .from("plans")
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

// function that uploads plan to our db
export async function server_create_plans(
  plans: IPlan[],
): Promise<IWebResponseCompatible<IPlan[]>> {
  // loop through plans and format
  let formatted_plans = plans.map((plan) => {
    let formatted_plan: any = { ...plan };
    delete formatted_plan.id;
    delete formatted_plan.inserted_at;
    return formatted_plan;
  });
  const supabase = await newServerClient();
  // insert data and return the response
  const { data, error } = await supabase
    .from("plans")
    .insert(formatted_plans)
    .select("*");
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

export async function server_delete_plans_by_ids(
  plan_ids: number[],
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();
  const { error } = await supabase.from("plans").delete().in("id", plan_ids);
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
