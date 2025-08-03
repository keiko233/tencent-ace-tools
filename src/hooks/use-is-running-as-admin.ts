import { useQuery, UseQueryOptions } from "@tanstack/react-query";
import { commands } from "@/bindings";
import { unwrapResult } from "@/lib/result";
import { IS_RUNNING_AS_ADMIN_QUERY_KEY } from "./consts";

export function useIsRunningAsAdmin(
  options?: Omit<UseQueryOptions, "queryKey" | "queryFn">,
) {
  return useQuery({
    queryKey: [IS_RUNNING_AS_ADMIN_QUERY_KEY],
    queryFn: async () => {
      return unwrapResult(await commands.isRunningAsAdmin());
    },
    ...options,
  });
}
