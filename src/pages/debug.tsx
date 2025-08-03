import { createFileRoute } from "@tanstack/react-router";
import { commands } from "@/bindings";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/debug")({
  component: RouteComponent,
});

const TestGreet = () => {
  return (
    <div>
      <Button
        onClick={() => {
          commands.greet("mike");
        }}
      >
        Greet
      </Button>
    </div>
  );
};

function RouteComponent() {
  return (
    <div className="flex flex-col gap-4 p-4">
      <TestGreet />
    </div>
  );
}
