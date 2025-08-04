import { createRootRoute, Outlet } from "@tanstack/react-router";
import { PropsWithChildren } from "react";
import AppFooter, { LoggerViewerButton } from "@/components/app-footer";
import AppHeader from "@/components/app-header";
import { AppSidebar } from "@/components/app-sidebar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";

export const Route = createRootRoute({
  component: RootComponent,
});

const Layout = ({ children }: PropsWithChildren) => {
  return (
    <SidebarProvider>
      <AppSidebar />

      <SidebarInset>
        <AppHeader />

        <div className="h-full overflow-auto">{children}</div>

        <AppFooter />

        <LoggerViewerButton />
      </SidebarInset>
    </SidebarProvider>
  );
};

function RootComponent() {
  return (
    <>
      <Toaster position="top-center" richColors />

      <Layout>
        <Outlet />
      </Layout>
    </>
  );
}
