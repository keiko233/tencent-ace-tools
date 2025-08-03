import { createFileRoute } from "@tanstack/react-router";
import AceProcessController from "@/components/ace-process-controller";

export const Route = createFileRoute("/")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <div className="flex flex-col gap-4 p-4">
      <AceProcessController />
    </div>
  );
}
