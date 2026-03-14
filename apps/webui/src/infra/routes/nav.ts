import { type } from "arktype";
import type { LucideIcon } from "lucide-react";
import type { ProLinkProps } from "@/components/ui/pro-link";

export interface NavMainItem {
  link?: ProLinkProps;
  title: string;
  icon?: LucideIcon;
  children?: { title: string; link: ProLinkProps }[];
}

export interface NavMainGroup {
  group: string;
  items: NavMainItem[];
}

export const CreateCompleteAction = {
  Back: "back",
  Detail: "detail",
} as const;

export type CreateCompleteAction =
  (typeof CreateCompleteAction)[keyof typeof CreateCompleteAction];

export const CreateCompleteActionSchema = type.enumerated(
  ...Object.values(CreateCompleteAction),
);
