import { createLink, type LinkComponentProps } from "@tanstack/react-router";
import type {
  AnchorHTMLAttributes,
  ComponentProps,
  ComponentType,
} from "react";

export interface BasicLinkProps
  extends AnchorHTMLAttributes<HTMLAnchorElement> {}

const BasicLinkComponent = (props: ComponentProps<"a">) => {
  return <a {...props} />;
};

const CreatedLinkComponent = createLink(BasicLinkComponent);

export type ProLinkProps =
  | LinkComponentProps<typeof BasicLinkComponent>
  | BasicLinkProps;

export const ProLink: ComponentType<ProLinkProps> = (props) => {
  if (props.href) {
    return <BasicLinkComponent {...(props as any)} />;
  }
  return <CreatedLinkComponent preload={"intent"} {...props} />;
};
