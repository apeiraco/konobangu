import type { Injector } from "@outposts/injection-js";
import type { LucideIcon } from "lucide-react";
import type { ProLinkProps } from "@/components/ui/pro-link";

export type RouterContext = {
  injector: Injector;
};

export type RouteBreadcrumbItem = {
  label?: string;
  icon?: LucideIcon;
  link?: Omit<ProLinkProps, "aria-current" | "current">;
};

export interface RouteStateDataOption {
  breadcrumb?: RouteBreadcrumbItem;
}
