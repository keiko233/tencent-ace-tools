import { createFileRoute } from "@tanstack/react-router";
import { Loader2 } from "lucide-react";
import { useCallback, useState, useTransition } from "react";
import { commands, type WindowInfo } from "@/bindings";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { unwrapResult } from "@/lib/result";

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

const Windows = () => {
  const [windows, setWindows] = useState<WindowInfo[]>([]);

  const handleGetAllWindows = async () => {
    const windows = unwrapResult(await commands.getAllWindows());
    setWindows(windows || []);
    console.log("Windows:", windows);
  };

  const [windowId, setWindowId] = useState<string>("");

  const [imageBase64, setImageBase64] = useState<string>("");

  const [isPending, startTransition] = useTransition();

  const handleCaptureByWindowId = useCallback(() => {
    startTransition(async () => {
      if (!windowId) {
        console.error("Window ID is required");
        return;
      }
      const result = unwrapResult(
        await commands.tryCaptureImageByWindowId(parseInt(windowId)),
      );

      if (result) {
        setImageBase64(result.image_base64);
        console.log("Captured image:", result);
      } else {
        console.error("Failed to capture image");
      }
    });
  }, [windowId]);

  return (
    <div className="flex flex-col gap-4">
      <Button onClick={handleGetAllWindows}>Get All Windows</Button>

      <pre className="max-h-96 overflow-auto text-sm">
        {JSON.stringify(windows, null, 2)}
      </pre>

      <div className="flex gap-2">
        <Input
          placeholder="Enter window title"
          value={windowId}
          onChange={(e) => setWindowId(e.target.value)}
        />

        <Button onClick={handleCaptureByWindowId} disabled={isPending}>
          {isPending && <Loader2 className="mr-2 animate-spin" />}
          <span>Capture by Window ID</span>
        </Button>
      </div>

      {imageBase64 && (
        <img src={`data:image/png;base64,${imageBase64}`} alt="Captured" />
      )}
    </div>
  );
};

function RouteComponent() {
  return (
    <div className="flex flex-col gap-4 p-4">
      <TestGreet />
      <Windows />
    </div>
  );
}
