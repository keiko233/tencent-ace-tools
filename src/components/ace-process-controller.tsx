import {
  BadgeCheckIcon,
  BadgeMinus,
  Loader2,
  Rocket,
  RotateCcw,
} from "lucide-react";
import { useAceProcessController } from "@/hooks/use-ace-process-controller";
import { cn } from "@/lib/utils";
import { m } from "@/paraglide/messages";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "./ui/card";

export default function AceProcessController() {
  const { guard, tryOptimizeProcesses } = useAceProcessController();

  const isSuccess = guard.data?.some((process) => process.priority_modified);

  return (
    <Card>
      <CardHeader>
        <CardTitle>{m.game_tools_ace_process_controller_title()} </CardTitle>

        <CardDescription>
          {m.game_tools_ace_process_controller_description()}
        </CardDescription>
      </CardHeader>

      <CardContent>
        {guard.isLoading ? (
          <div className="flex items-center gap-2">
            <Loader2 className="size-4 animate-spin" />
            <span>Loading processes...</span>
          </div>
        ) : (
          <div className="space-y-2">
            {guard.data && guard.data.length > 0 ? (
              guard.data.map((process, index) => (
                <div key={index}>
                  {Object.entries(process).map(([key, value]) => (
                    <div key={key} className="flex justify-between text-sm">
                      <span className="font-medium">{key}:</span>
                      <span className="text-muted-foreground">{value}</span>
                    </div>
                  ))}
                </div>
              ))
            ) : (
              <div className="text-muted-foreground">
                {m.game_tools_ace_process_controller_no_processes()}
              </div>
            )}

            {guard.data && guard.data.length > 0 && (
              <Badge
                variant="default"
                className={cn(
                  "text-white",
                  isSuccess ? "bg-green-500" : "bg-red-500",
                )}
              >
                {isSuccess ? (
                  <BadgeCheckIcon className="size-3" />
                ) : (
                  <BadgeMinus className="size-3" />
                )}

                <span>
                  {isSuccess
                    ? m.game_tools_ace_process_controller_optimize_success()
                    : m.game_tools_ace_process_controller_optimize_failed()}
                </span>
              </Badge>
            )}
          </div>
        )}
      </CardContent>

      <CardFooter className="gap-2">
        <Button variant="outline" onClick={() => guard.refetch()}>
          <RotateCcw />
          <span>{m.game_tools_ace_process_controller_refresh()}</span>
        </Button>

        <Button onClick={tryOptimizeProcesses}>
          <Rocket />
          <span>
            {isSuccess
              ? m.game_tools_ace_process_controller_try_optimize_again()
              : m.game_tools_ace_process_controller_try_optimize()}
          </span>
        </Button>
      </CardFooter>
    </Card>
  );
}
