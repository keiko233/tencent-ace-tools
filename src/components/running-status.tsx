import { BadgeCheckIcon, Loader2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { useIsRunningAsAdmin } from "@/hooks/use-is-running-as-admin";
import { m } from "@/paraglide/messages";

export default function RunningStatus() {
  const { isPending, data: status } = useIsRunningAsAdmin();

  return (
    <div className="flex items-center gap-2">
      {isPending ? (
        <Badge variant="secondary" className="animate-pulse">
          <Loader2 className="animate-spin" />
          <span>{m.checking_for_running_status()}</span>
        </Badge>
      ) : status ? (
        <Badge
          variant="secondary"
          className="bg-blue-500 text-white dark:bg-blue-600"
        >
          <BadgeCheckIcon />
          <span>{m.running_as_admin()}</span>
        </Badge>
      ) : (
        <Badge variant="destructive">{m.not_running_as_admin()}</Badge>
      )}
    </div>
  );
}
