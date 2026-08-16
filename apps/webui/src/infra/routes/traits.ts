import type { SecuritydeptInjectorTrait as Injector } from "@securitydept/client";
import type { LucideIcon } from "lucide-react";
import type { ProLinkProps } from "@/components/ui/pro-link";

export type RouterContext = {
  injector?: Injector;
};

export type RouteBreadcrumbItem = {
  label?: string;
  icon?: LucideIcon;
  link?: Omit<ProLinkProps, "aria-current" | "current">;
};

export interface RouteStateDataOption {
  breadcrumb?: RouteBreadcrumbItem;
}

export function requireRouterInjector(context: RouterContext): Injector {
  if (!context.injector) {
    throw new Error(
      "Application runtime must be attached before loading routes.",
    );
  }
  return context.injector;
}
