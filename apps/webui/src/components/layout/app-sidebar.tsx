import type { ComponentPropsWithoutRef } from "react";
import { AppNavMainData } from "@/app/config/nav";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar";
import { AppIcon } from "./app-icon";
import { NavMain } from "./nav-main";
import { NavUser } from "./nav-user";

const data = {
  user: {
    name: "shadcn",
    email: "m@example.com",
    avatar: "/avatars/shadcn.jpg",
  },
};

type AppSidebarRootProps = Omit<
  ComponentPropsWithoutRef<typeof Sidebar>,
  "collapsible"
>;

export const AppSidebar = (props: AppSidebarRootProps) => {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <AppIcon />
      </SidebarHeader>
      <SidebarContent>
        <NavMain groups={AppNavMainData} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
};
