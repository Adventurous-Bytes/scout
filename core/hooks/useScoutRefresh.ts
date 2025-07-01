import { useEffect } from "react";
import { useAppDispatch } from "../store/hooks";
import {
  EnumScoutStateStatus,
  setActiveHerdId,
  setHerdModules,
  setStatus,
  setUser,
} from "../store/scout";
import { server_load_herd_modules } from "../helpers/herds";
import { server_get_user } from "../helpers/users";
import {
  addDevice,
  addPlan,
  addTag,
  deleteDevice,
  deletePlan,
  deleteTag,
  updateDevice,
  updatePlan,
  updateTag,
} from "../store/scout";

export interface UseScoutRefreshOptions {
  autoRefresh?: boolean;
  onRefreshComplete?: () => void;
}

export function useScoutRefresh(options: UseScoutRefreshOptions = {}) {
  const { autoRefresh = true, onRefreshComplete } = options;
  const dispatch = useAppDispatch();

  const handleRefresh = async () => {
    dispatch(setStatus(EnumScoutStateStatus.LOADING));

    try {
      const compatible_new_herd_modules = await server_load_herd_modules();
      const res_new_user = await server_get_user();

      dispatch(setHerdModules(compatible_new_herd_modules));
      dispatch(setUser(res_new_user.data));

      // Check local storage for a last selected herd
      if (localStorage.getItem("last_selected_herd")) {
        const found_herd = compatible_new_herd_modules.find(
          (hm) =>
            hm.herd.id.toString() === localStorage.getItem("last_selected_herd")
        )?.herd;

        // If herd is found then set it
        if (found_herd) {
          dispatch(setActiveHerdId(found_herd.id.toString()));
        }
      }
      // If there is no last selected herd then select the first one
      else if (compatible_new_herd_modules.length > 0) {
        localStorage.setItem(
          "last_selected_herd",
          compatible_new_herd_modules[0].herd.id.toString()
        );
        dispatch(
          setActiveHerdId(compatible_new_herd_modules[0].herd.id.toString())
        );
      }

      dispatch(setStatus(EnumScoutStateStatus.DONE_LOADING));
      onRefreshComplete?.();
    } catch (error) {
      console.error("Error refreshing scout data:", error);
      dispatch(setStatus(EnumScoutStateStatus.DONE_LOADING));
    }
  };

  useEffect(() => {
    if (autoRefresh) {
      handleRefresh();
    }
  }, [autoRefresh]);

  return {
    handleRefresh,
  };
}
