import { useSet } from "@uidotdev/usehooks";
import {
  createContext,
  useContext,
  useEffect,
  useState,
  type PropsWithChildren,
} from "react";
import { events, type LogEvent } from "@/bindings";

type LoggerContextType = {
  logs: LogEvent[];
  length: number;
  openLoggerViewer: boolean;
  setOpenLoggerViewer: (open: boolean) => void;
};

const LoggerContext = createContext<LoggerContextType | null>(null);

export const useLogger = () => {
  const context = useContext(LoggerContext);

  if (!context) {
    throw new Error("useLogger must be used within a LoggerProvider");
  }

  return context;
};

export default function LoggerProvider({ children }: PropsWithChildren) {
  const logs = useSet<LogEvent>();

  const [openLoggerViewer, setOpenLoggerViewer] = useState(false);

  useEffect(() => {
    const unlisten = events.logEvent.listen(({ payload }) => {
      logs.add(payload);
    });

    return () => {
      unlisten.then((f) => f());
    };
    // eslint-disable-next-line react-compiler/react-compiler
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <LoggerContext.Provider
      value={{
        logs: Array.from(logs),
        length: logs.size,
        openLoggerViewer,
        setOpenLoggerViewer,
      }}
    >
      {children}
    </LoggerContext.Provider>
  );
}
