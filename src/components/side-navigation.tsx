import { Link, useLocation } from "@tanstack/react-router";
import { Bug, ChevronRight, Gamepad2 } from "lucide-react";
import { ReactNode } from "react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
} from "@/components/ui/sidebar";
import { GAME_ENUM } from "@/consts";
import { m } from "@/paraglide/messages";

type NavigationItem = {
  title: string;
  url?: string;
  icon?: ReactNode;
  items?: NavigationItem[];
};

const NAVBAR_LIST: Record<GAME_ENUM, Record<string, NavigationItem[]>> = {
  [GAME_ENUM.DeltaForce]: {
    [m.navigation_game_tools()]: [
      {
        title: m.navigation_game_tools_optimization(),
        url: "/",
        icon: <Gamepad2 />,
      },
    ],
    [m.navigation_developer_tools()]: [
      {
        title: m.navigation_developer_tools_debug(),
        url: "/debug",
        icon: <Bug />,
      },
    ],
  },
};

const SideNavigationItem = ({ title, url, icon, items }: NavigationItem) => {
  const location = useLocation();

  const isActive = location.pathname === url;

  return (
    <SidebarMenuButton tooltip={title} isActive={isActive}>
      {icon}

      <span>{title}</span>

      {items && items.length > 0 && (
        <ChevronRight className="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
      )}
    </SidebarMenuButton>
  );
};

const SideNavigationRender = ({ items }: { items: NavigationItem[] }) => {
  return items.map((item) => {
    if (item.items && item.items.length > 0) {
      return (
        <Collapsible key={item.title} className="group/collapsible" asChild>
          <SidebarMenuItem>
            <CollapsibleTrigger className="w-full">
              <SideNavigationItem {...item} />
            </CollapsibleTrigger>

            <CollapsibleContent>
              <SidebarMenuSub>
                <SideNavigationRender items={item.items} />
              </SidebarMenuSub>
            </CollapsibleContent>
          </SidebarMenuItem>
        </Collapsible>
      );
    }

    return (
      <Link to={item.url} key={item.title}>
        <SideNavigationItem {...item} />
      </Link>
    );
  });
};

export default function SideNavigation() {
  // TODO: support multiple games
  const currentGame = GAME_ENUM.DeltaForce; // This should be dynamic based on the current game context

  return Object.entries(NAVBAR_LIST[currentGame]).map(([gameKey, item]) => (
    <SidebarGroup key={gameKey}>
      <SidebarGroupLabel>{gameKey}</SidebarGroupLabel>

      <SidebarMenu>
        <SideNavigationRender items={item} />
      </SidebarMenu>
    </SidebarGroup>
  ));
}
