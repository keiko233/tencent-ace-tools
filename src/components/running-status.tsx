import { BadgeCheckIcon, BadgeX, Loader2 } from "lucide-react";
import { ComponentProps } from "react";
import { Badge as BadgePrimitive } from "@/components/ui/badge";
import { useIsRunningAsAdmin } from "@/hooks/use-is-running-as-admin";
import { cn } from "@/lib/utils";
import { m } from "@/paraglide/messages";
import { useSidebar } from "./ui/sidebar";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";

const Badge = ({
  icon,
  description,
  className,
  ...props
}: ComponentProps<typeof BadgePrimitive> & {
  icon?: boolean;
  description?: string;
}) => {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <BadgePrimitive
          className={cn("h-6 cursor-default", icon && "w-6 px-0", className)}
          {...props}
        />
      </TooltipTrigger>

      <TooltipContent>
        <p>{description}</p>
      </TooltipContent>
    </Tooltip>
  );
};

export default function RunningStatus() {
  const { isPending, data: status } = useIsRunningAsAdmin();

  const { open } = useSidebar();

  return (
    <div className="flex items-center gap-2">
      {isPending ? (
        <Badge
          variant="secondary"
          className="animate-pulse"
          icon={!open}
          description={m.checking_for_running_status()}
        >
          <Loader2 className="animate-spin" />
          {open && <span>{m.checking_for_running_status()}</span>}
        </Badge>
      ) : status ? (
        <Badge
          variant="secondary"
          className="h-6 bg-blue-500 text-white dark:bg-blue-600"
          icon={!open}
          description={m.running_as_admin_description()}
        >
          <BadgeCheckIcon />
          {open && <span>{m.running_as_admin()}</span>}
        </Badge>
      ) : (
        <Badge
          variant="destructive"
          icon={!open}
          description={m.not_running_as_admin_description()}
        >
          <BadgeX />
          {open && <span>{m.not_running_as_admin()}</span>}
        </Badge>
      )}
    </div>
  );
}
