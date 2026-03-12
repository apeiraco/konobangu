"use client";

import { useMatches } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ProLink, type ProLinkProps } from "@/components/ui/pro-link";
import {
	SidebarGroup,
	SidebarGroupLabel,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
	SidebarMenuSub,
	SidebarMenuSubButton,
	SidebarMenuSubItem,
	useSidebar,
} from "@/components/ui/sidebar";
import type { NavMainGroup, NavMainItem } from "@/infra/routes/nav";

export function NavMain({ groups }: { groups: NavMainGroup[] }) {
	const matches = useMatches();
	const { state } = useSidebar();

	const isMenuMatch = (link: ProLinkProps | undefined) => {
		const linkTo = link?.to;
		if (!linkTo) {
			return false;
		}
		return matches.some((match) => match.pathname.startsWith(linkTo));
	};

	const renderSidebarMenuItemButton = (item: NavMainItem) => {
		return (
			<>
				{item.icon && <item.icon className="h-4 w-4" aria-hidden="true" />}
				<span className="truncate">{item.title}</span>
				<ChevronRight className="ml-auto transition-transform duration-200 group-data-[state=open]:rotate-90" />
			</>
		);
	};

	const renderCollapsedSubMenu = (item: NavMainItem, itemIndex: number) => {
		return (
			<DropdownMenu key={itemIndex}>
				<DropdownMenuTrigger asChild>
					<SidebarMenuButton
						tooltip={item.title}
						isActive={isMenuMatch(item.link)}
						aria-label={`${item.title} (expandable)`}
					>
						{item.icon && <item.icon className="h-4 w-4" aria-hidden="true" />}
					</SidebarMenuButton>
				</DropdownMenuTrigger>
				<DropdownMenuContent side="right" align="start" className="min-w-48">
					<DropdownMenuItem asChild>
						<span className="font-medium">{item.title}</span>
					</DropdownMenuItem>
					<DropdownMenuSeparator />
					{item.children?.map((subItem, index) => (
						<DropdownMenuItem key={index} asChild>
							<ProLink {...subItem.link}>{subItem.title}</ProLink>
						</DropdownMenuItem>
					))}
				</DropdownMenuContent>
			</DropdownMenu>
		);
	};

	const renderExpandedSubMenu = (item: NavMainItem, itemIndex: number) => {
		return (
			<Collapsible
				key={itemIndex}
				asChild
				className="group/collapsible"
				defaultOpen={isMenuMatch(item.link)}
			>
				<SidebarMenuItem>
					<CollapsibleTrigger asChild>
						<SidebarMenuButton
							tooltip={item.title}
							aria-label={`${item.title} (expandable)`}
						>
							{renderSidebarMenuItemButton(item)}
						</SidebarMenuButton>
					</CollapsibleTrigger>
					<CollapsibleContent>
						<SidebarMenuSub>
							{(item.children || []).map((subItem, subItemIndex) => {
								return (
									<SidebarMenuSubItem key={subItemIndex}>
										<SidebarMenuSubButton
											asChild
											isActive={isMenuMatch(subItem.link)}
										>
											<ProLink
												{...subItem.link}
												activeProps={{ className: "" }}
												aria-label={`${subItem.title}`}
											>
												<span>{subItem.title}</span>
											</ProLink>
										</SidebarMenuSubButton>
									</SidebarMenuSubItem>
								);
							})}
						</SidebarMenuSub>
					</CollapsibleContent>
				</SidebarMenuItem>
			</Collapsible>
		);
	};

	return groups.map((group, groupIndex) => {
		return (
			<SidebarGroup key={groupIndex}>
				<SidebarGroupLabel>{group.group}</SidebarGroupLabel>
				<SidebarMenu>
					{group.items.map((item, itemIndex) => {
						if (!item.children?.length) {
							return (
								<SidebarMenuItem key={itemIndex}>
									<SidebarMenuButton
										asChild
										tooltip={item.title}
										isActive={isMenuMatch(item.link)}
									>
										<ProLink {...item.link} tabIndex={0}>
											{renderSidebarMenuItemButton(item)}
										</ProLink>
									</SidebarMenuButton>
								</SidebarMenuItem>
							);
						}
						return state === "collapsed"
							? renderCollapsedSubMenu(item, itemIndex)
							: renderExpandedSubMenu(item, itemIndex);
					})}
				</SidebarMenu>
			</SidebarGroup>
		);
	});
}
