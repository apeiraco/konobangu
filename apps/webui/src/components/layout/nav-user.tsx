import { ChevronsUpDown, LogOut } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { useAuth } from "@/app/auth/hooks";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar";
import { AUTH_METHOD } from "@/infra/auth/defs";

export function NavUser() {
  const { isMobile } = useSidebar();
  const { authData, authService, type } = useAuth();
  const [pending, setPending] = useState(false);
  const principal = authData?.principal;
  const name = principal?.displayName ?? principal?.subject ?? "Signed out";
  async function logout() {
    setPending(true);
    try {
      await authService.logout();
      window.location.assign("/");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Sign out failed");
    } finally {
      setPending(false);
    }
  }
  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton size="lg">
              <Avatar className="h-8 w-8 rounded-lg">
                <AvatarImage src={principal?.picture ?? undefined} alt={name} />
                <AvatarFallback>{name.slice(0, 2)}</AvatarFallback>
              </Avatar>
              <div className="grid flex-1 text-left text-xs">
                <span className="truncate font-semibold">{name}</span>
                <span className="truncate">{principal?.issuer}</span>
              </div>
              <ChevronsUpDown className="ml-auto size-4" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent side={isMobile ? "bottom" : "right"} align="end">
            <DropdownMenuLabel>{name}</DropdownMenuLabel>
            <DropdownMenuSeparator />
            {type === AUTH_METHOD.BASIC && (
              <DropdownMenuLabel className="max-w-64 whitespace-normal text-xs font-normal">
                Basic credentials are cached by your browser. Close the
                authenticated browser session to sign out.
              </DropdownMenuLabel>
            )}
            <DropdownMenuItem
              disabled={pending || type === AUTH_METHOD.BASIC}
              onSelect={() => {
                void logout();
              }}
            >
              <LogOut />
              Sign out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
