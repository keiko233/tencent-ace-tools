import { ChevronsUpDown } from "lucide-react";
import { useState } from "react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { GAME_ENUM } from "@/consts";
import { m } from "@/paraglide/messages";

const GAME_METADATA = {
  [GAME_ENUM.DeltaForce]: {
    name: "Delta Force",
    i18n: m.game_name_delta_force(),
    logo: (
      <img
        src="/icons/3rd/delta_force_logo.webp"
        alt="Delta Force Logo"
        className="size-4"
      />
    ),
  },
};

export default function GameSwitcher() {
  // TODO: support multiple games
  const [activeGame, setActiveGame] = useState<GAME_ENUM>(GAME_ENUM.DeltaForce);

  const currentGameMetadata = GAME_METADATA[activeGame];

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <div className="text-sidebar-primary-foreground border-sidebar-secondary flex aspect-square size-8 items-center justify-center rounded-lg border">
                {currentGameMetadata.logo}
              </div>

              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-medium">
                  {currentGameMetadata.name}
                </span>

                <span className="truncate text-xs">
                  {currentGameMetadata.i18n}
                </span>
              </div>
              <ChevronsUpDown className="ml-auto" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>

          <DropdownMenuContent
            className="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg"
            align="start"
            side="bottom"
            sideOffset={4}
          >
            <DropdownMenuLabel className="text-muted-foreground text-xs">
              Games
            </DropdownMenuLabel>

            {Object.entries(GAME_METADATA).map(([gameKey, game]) => (
              <DropdownMenuItem
                key={game.name}
                onClick={() => setActiveGame(Number(gameKey) as GAME_ENUM)}
                className="gap-2 p-2"
              >
                <div className="flex size-6 items-center justify-center rounded-md border">
                  {game.logo}
                </div>

                {game.name}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
