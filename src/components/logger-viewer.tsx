import dayjs from "dayjs";
import { ComponentProps } from "react";
import type { LogLevel } from "@/bindings";
import { cn } from "@/lib/utils";
import { useLogger } from "@/providers/logger-provider";
import { Badge as OriginBadge } from "./ui/badge";

const Badge = ({ className, ...props }: ComponentProps<typeof OriginBadge>) => {
  return (
    <OriginBadge
      className={cn("px-1.5 py-0 text-[10px]", className)}
      {...props}
    />
  );
};

export default function LoggerViewer() {
  const { logs } = useLogger();

  const logColors: Record<LogLevel, string> = {
    TRACE: "bg-gray-400 text-gray-800",
    ERROR: "bg-red-500 text-white",
    WARN: "bg-yellow-500 text-black",
    INFO: "bg-green-500 text-white",
    DEBUG: "bg-blue-500 text-white",
  };

  // TODO: virtual scroll for performance
  return (
    <>
      {logs.map((log, index) => (
        <p
          key={index}
          className={cn(
            "flex w-full items-start gap-1 font-mono text-xs whitespace-nowrap",
          )}
          data-log-level={log.level}
          data-log-timestamp={log.timestamp}
        >
          <Badge variant="secondary">
            {dayjs(log.timestamp).format("HH:mm:ss")}
          </Badge>

          <Badge className={cn("px-1.5 py-0", logColors[log.level])}>
            {log.level}
          </Badge>

          <span className="text-zinc-500">{log.target}:</span>

          <span className="text-zinc-900 dark:text-zinc-100">
            {log.message}
          </span>

          {Object.entries(log.fields).map(([key, value]) => (
            <span key={key} className="field">
              <strong>{key}:</strong> {value}
            </span>
          ))}
          {index < logs.length - 1 && <hr className="log-separator" />}
        </p>
      ))}
    </>
  );
}
