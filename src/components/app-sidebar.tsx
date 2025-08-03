import GameSwitcher from "@/components/game-switcher";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
} from "@/components/ui/sidebar";
import RunningStatus from "./running-status";
import SideNavigation from "./side-navigation";

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <GameSwitcher />
      </SidebarHeader>

      <SidebarContent>
        <SideNavigation />
      </SidebarContent>

      <SidebarFooter>
        <RunningStatus />
      </SidebarFooter>
    </Sidebar>
  );
}
