import { createLink, type LinkComponent } from "@tanstack/react-router";
import type { AnchorHTMLAttributes, ComponentProps } from "react";

export interface BasicLinkProps
	extends AnchorHTMLAttributes<HTMLAnchorElement> {}

const BasicLinkComponent = (props: ComponentProps<"a">) => {
	return <a {...props} />;
};

const CreatedLinkComponent = createLink(BasicLinkComponent);

export const ProLink: LinkComponent<typeof BasicLinkComponent> = (props) => {
	if (props.href) {
		return <BasicLinkComponent {...(props as any)} />;
	}
	return <CreatedLinkComponent preload={"intent"} {...props} />;
};

export type ProLinkProps = ComponentProps<typeof ProLink>;
