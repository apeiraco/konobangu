import { useMatches } from "@tanstack/react-router";
import {
	type DetailedHTMLProps,
	Fragment,
	type HTMLAttributes,
	useMemo,
} from "react";
import { AppSidebar } from "@/components/layout/app-sidebar";
import {
	Breadcrumb,
	BreadcrumbItem,
	BreadcrumbLink,
	BreadcrumbList,
	BreadcrumbPage,
	BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Separator } from "@/components/ui/separator";
import {
	SidebarInset,
	SidebarProvider,
	SidebarTrigger,
} from "@/components/ui/sidebar";
import type {
	RouteBreadcrumbItem,
	RouteStateDataOption,
} from "@/infra/routes/traits";
import { cn } from "@/presentation/utils";
import { ProLink } from "../ui/pro-link";

export type AppAsideBreadcrumbItem = RouteBreadcrumbItem;

export interface AppAsideProps
	extends DetailedHTMLProps<HTMLAttributes<HTMLDivElement>, HTMLDivElement> {
	breadcrumb?: AppAsideBreadcrumbItem[];
	extractBreadcrumbFromRoutes?: boolean;
}

export function AppAside({
	children,
	className,
	breadcrumb: propBreadcrumb,
	extractBreadcrumbFromRoutes,
	...other
}: AppAsideProps) {
	const matches = useMatches();

	const breadcrumb = useMemo(() => {
		if (propBreadcrumb) {
			return propBreadcrumb;
		}
		if (extractBreadcrumbFromRoutes) {
			return matches
				.map((m, i, arr) => {
					const staticData = m.staticData as RouteStateDataOption;
					if (staticData.breadcrumb) {
						return {
							link:
								i + 1 >= arr.length
									? undefined
									: {
											to: m.pathname,
										},
							...staticData.breadcrumb,
						} as AppAsideBreadcrumbItem;
					}
					return undefined;
				})
				.filter((b): b is AppAsideBreadcrumbItem => !!b);
		}
		return [];
	}, [matches, propBreadcrumb, extractBreadcrumbFromRoutes]);

	const breadcrumbLength = breadcrumb.length;

	return (
		<SidebarProvider>
			<AppSidebar />
			<SidebarInset>
				<header className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12">
					<div className="flex items-center gap-2 px-4">
						<SidebarTrigger className="-ml-1" />
						{breadcrumbLength > 0 && (
							<>
								<Separator orientation="vertical" className="mr-2 h-4" />
								<Breadcrumb>
									<BreadcrumbList>
										{breadcrumb.map((item, index) => {
											const iconEl = item.icon ? (
												<item.icon className="size-4" />
											) : null;

											const isCurrent = index + 1 === breadcrumbLength;

											const LinkChild = (
												item.link ? ProLink : Fragment
											) as typeof ProLink;
											return (
												<Fragment key={index}>
													{index > 0 && (
														<BreadcrumbSeparator className="hidden md:block" />
													)}
													<BreadcrumbItem className="hidden md:block">
														{isCurrent ? (
															<BreadcrumbPage>
																{iconEl}
																{item.label}
															</BreadcrumbPage>
														) : (
															<BreadcrumbLink
																className="text-[var(--foreground)] hover:text-inherit"
																asChild={!!item.link}
															>
																<LinkChild {...item?.link}>
																	{iconEl}
																	{item.label}
																</LinkChild>
															</BreadcrumbLink>
														)}
													</BreadcrumbItem>
												</Fragment>
											);
										})}
									</BreadcrumbList>
								</Breadcrumb>
							</>
						)}
					</div>
				</header>
				<div
					{...other}
					className={cn("flex min-h-0 flex-1 flex-col p-4 pt-0", className)}
				>
					{children}
				</div>
			</SidebarInset>
		</SidebarProvider>
	);
}
