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
  | (LinkComponentProps<typeof BasicLinkComponent> & { href?: never })
  | (BasicLinkProps & { href: string } & Pick<
        LinkComponentProps<typeof BasicLinkComponent>,
        "activeProps" | "inactiveProps"
      >);

export const ProLink: ComponentType<ProLinkProps> = (props) => {
  if (props.href !== undefined) {
    const {
      activeProps: _activeProps,
      inactiveProps: _inactiveProps,
      ...anchorProps
    } = props;
    return <BasicLinkComponent {...anchorProps} />;
  }
  return <CreatedLinkComponent preload={"intent"} {...props} />;
};
