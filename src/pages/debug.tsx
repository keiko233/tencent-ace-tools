import { zodResolver } from "@hookform/resolvers/zod";
import { createFileRoute } from "@tanstack/react-router";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { Loader2 } from "lucide-react";
import { useCallback, useState, useTransition } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";
import {
  commands,
  OcrRegion,
  OcrResponse,
  ScreenShot,
  type WindowInfo,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { NumberInput } from "@/components/ui/number-input";
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

const FormSchema = z.object({
  x: z.coerce.number().min(0),
  y: z.coerce.number().min(0),
  width: z.coerce.number().min(1),
  height: z.coerce.number().min(1),
}) as z.ZodType<OcrRegion>;

const Windows = () => {
  const [windows, setWindows] = useState<WindowInfo[]>([]);

  const handleGetAllWindows = async () => {
    const windows = unwrapResult(await commands.getAllWindows());
    setWindows(windows || []);
    console.log("Windows:", windows);
  };

  const [windowId, setWindowId] = useState<string>("");

  const [screenShotResult, setScreenShotResult] = useState<ScreenShot>();

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
        setScreenShotResult(result);
        console.log("Captured image:", result);
      } else {
        console.error("Failed to capture image");
      }
    });
  }, [windowId]);

  const form = useForm<z.infer<typeof FormSchema>>({
    // FIXME: This is a workaround for the type issue with zodResolver
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    resolver: zodResolver(FormSchema as any),
    defaultValues: {
      x: 0,
      y: 0,
      width: 0,
      height: 0,
    },
  });

  const [ocrResult, setOcrResult] = useState<OcrResponse>();

  const handleOcrScreenRegion = form.handleSubmit((data) => {
    startTransition(async () => {
      if (!screenShotResult) {
        console.error("Image is required for OCR");
        return;
      }

      const result = unwrapResult(
        await commands.ocrImageRegion(screenShotResult.image_data, data),
      );

      if (!result) {
        console.error("Failed to perform OCR");
        return;
      }

      setOcrResult(result);
      console.log("OCR Result:", result);
    });
  });

  const handleSaveImage = useCallback(async () => {
    if (!screenShotResult) {
      console.error("No image to save");
      return;
    }

    try {
      const filePath = await save({
        filters: [
          {
            name: "PNG 图片",
            extensions: ["png"],
          },
        ],
        defaultPath: `screenshot_${Date.now()}.png`,
      });

      if (!filePath) {
        console.log("User cancelled save dialog");
        return;
      }

      // image_data is already binary data (PNG format)
      const bytes = new Uint8Array(screenShotResult.image_data);

      // write the file
      await writeFile(filePath, bytes);
      toast.success("Image saved successfully!", {
        description: `${filePath}`,
      });
    } catch (error) {
      toast.error("Failed to save image");
      console.error("Failed to save image:", error);
    }
  }, [screenShotResult]);

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

      <div className="grid grid-cols-2 gap-4">
        <Form {...form}>
          <FormField
            control={form.control}
            name="x"
            render={({ field }) => (
              <FormItem>
                <FormLabel>X</FormLabel>
                <FormControl>
                  <NumberInput placeholder="X" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="y"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Y</FormLabel>
                <FormControl>
                  <NumberInput placeholder="Y" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="width"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Width</FormLabel>
                <FormControl>
                  <NumberInput placeholder="Width" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="height"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Height</FormLabel>
                <FormControl>
                  <NumberInput placeholder="Height" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
        </Form>
      </div>

      <Button
        onClick={handleOcrScreenRegion}
        disabled={!screenShotResult || form.formState.isSubmitting}
      >
        {form.formState.isSubmitting && (
          <Loader2 className="mr-2 animate-spin" />
        )}
        <span>OCR Screen Region</span>
      </Button>

      {screenShotResult && (
        <div className="flex flex-col gap-2">
          <Button onClick={handleSaveImage}>Save Image</Button>

          <img
            src={
              screenShotResult.image_data
                ? URL.createObjectURL(
                    new Blob([new Uint8Array(screenShotResult.image_data)], {
                      type: "image/png",
                    }),
                  )
                : ""
            }
            alt="Captured"
          />
        </div>
      )}

      <pre className="max-h-96 overflow-auto text-sm">
        {JSON.stringify(
          {
            screenShotResult: screenShotResult
              ? {
                  width: screenShotResult.width,
                  height: screenShotResult.height,
                  format: screenShotResult.format,
                  dataSize: screenShotResult.image_data.length,
                }
              : null,
            ocrResult,
          },
          null,
          2,
        )}
      </pre>
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
