import { useQuery } from "@tanstack/react-query";
import { useCallback } from "react";
import { commands } from "@/bindings";
import { unwrapResult } from "@/lib/result";
import {
  ACE_GUARD_PRIVILEGES_QUERY_KEY,
  ACE_PROCESS_CONTROLLER_QUERY_KEY,
} from "./consts";

export function useAceProcessController() {
  const guard = useQuery({
    queryKey: [ACE_PROCESS_CONTROLLER_QUERY_KEY],
    queryFn: async () => {
      return unwrapResult(await commands.getAceGuardProcesses());
    },
  });

  const privileges = useQuery({
    queryKey: [ACE_GUARD_PRIVILEGES_QUERY_KEY],
    queryFn: async () => {
      return unwrapResult(await commands.getControllerPrivilegesStatus());
    },
  });

  const tryOptimizeProcesses = useCallback(async () => {
    const result = unwrapResult(await commands.optimizeAceGuardProcesses());

    await guard.refetch();

    return result;
  }, [guard]);

  return {
    guard,
    privileges,
    tryOptimizeProcesses,
  };
}
