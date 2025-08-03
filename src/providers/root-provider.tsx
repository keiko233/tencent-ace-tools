import { PropsWithChildren, useEffect } from "react";
import LoggerProvider from "./logger-provider";
import QueryProvider from "./query-provider";

const disabledGlobalRightClick = (e: MouseEvent) => {
  if (e.button === 2) {
    e.preventDefault();
  }
};

export default function RootProvider({ children }: PropsWithChildren) {
  useEffect(() => {
    document.addEventListener("contextmenu", disabledGlobalRightClick);

    return () => {
      document.removeEventListener("contextmenu", disabledGlobalRightClick);
    };
  }, []);

  return (
    <QueryProvider>
      <LoggerProvider>{children}</LoggerProvider>{" "}
    </QueryProvider>
  );
}
