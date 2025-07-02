import { createLink, type LinkComponentProps } from "@tanstack/react-router";
import type { AnchorHTMLAttributes } from "react";

export interface BasicLinkProps
	extends Omit<AnchorHTMLAttributes<HTMLAnchorElement>, "href"> {
	href: string;
	to?: undefined;
}

const BasicLinkComponent = (props: BasicLinkProps) => {
	return <a {...props} />;
};

const CreatedLinkComponent = createLink(BasicLinkComponent);

export const ProLink = (
	props: LinkComponentProps<typeof BasicLinkComponent> | BasicLinkProps,
) => {
	if (props.href) {
		return <BasicLinkComponent {...(props as any)} />;
	}
	return (
		<CreatedLinkComponent
			preload={"intent"}
			{...(props as LinkComponentProps<typeof BasicLinkComponent>)}
		/>
	);
};

export type ProLinkProps =
	| LinkComponentProps<typeof BasicLinkComponent>
	| BasicLinkProps;
