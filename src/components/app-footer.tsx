import { SquareCode, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { cn } from "@/lib/utils";
import { useLogger } from "@/providers/logger-provider";
import LoggerViewer from "./logger-viewer";
import { Button } from "./ui/button";

export const LoggerViewerButton = () => {
  const { openLoggerViewer, setOpenLoggerViewer } = useLogger();

  const handleViewerToggle = () => {
    setOpenLoggerViewer(!openLoggerViewer);
  };

  return (
    <AnimatePresence>
      {!openLoggerViewer && (
        <Button
          className="fixed right-8 bottom-8"
          size="icon"
          variant="ghost"
          onClick={handleViewerToggle}
          asChild
        >
          <motion.button
            initial={{ scale: 0, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0, opacity: 0 }}
            transition={{
              type: "tween",
            }}
          >
            <SquareCode />
          </motion.button>
        </Button>
      )}
    </AnimatePresence>
  );
};

export default function AppFooter() {
  const { openLoggerViewer, setOpenLoggerViewer } = useLogger();

  return (
    <AnimatePresence>
      {openLoggerViewer && (
        <motion.div
          className={cn(
            "max-h-1/2 min-h-40 w-full border-t border-zinc-200 dark:border-zinc-800",
          )}
          initial={{ y: "100%" }}
          animate={{ y: 0 }}
          exit={{ y: "100%" }}
          transition={{
            type: "spring",
            damping: 20,
          }}
        >
          <div className="flex h-8 items-center justify-between border-b border-zinc-200 pr-1 pl-3 dark:border-zinc-800">
            <SquareCode className="size-3" />

            <Button
              className="size-6"
              size="icon"
              variant="ghost"
              onClick={() => setOpenLoggerViewer(false)}
            >
              <X />
            </Button>
          </div>

          <div
            className={cn("flex flex-col gap-0.5", "overflow-auto p-2")}
            style={{ height: "calc(100% - 2rem)" }}
          >
            <LoggerViewer />
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
