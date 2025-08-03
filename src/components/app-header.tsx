import { Separator } from "@radix-ui/react-separator";
import { cn } from "@/lib/utils";
import { ThemeSwitcher } from "./theme-switcher";
import { SidebarTrigger } from "./ui/sidebar";

export default function AppHeader() {
  return (
    <header
      className={cn(
        "flex h-14 w-full shrink-0 items-center justify-between gap-2 px-4",
        "border-b border-zinc-200 dark:border-zinc-800",
      )}
    >
      <div className="flex items-center gap-2">
        <SidebarTrigger className="-ml-1" />

        <Separator
          orientation="vertical"
          className="mr-2 data-[orientation=vertical]:h-4"
        />

        <h1 className="text-lg font-semibold">Tencent ACE Tools</h1>

        {/* <Breadcrumb className="flex-1">
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbLink href="#">
                Building Your Application
              </BreadcrumbLink>
            </BreadcrumbItem>

            <BreadcrumbSeparator className="hidden md:block" />

            <BreadcrumbItem>
              <BreadcrumbPage>Data Fetching</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb> */}
      </div>

      <ThemeSwitcher />
    </header>
  );
}
