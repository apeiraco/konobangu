import {
	BookOpen,
	Folders,
	KeyRound,
	ListTodo,
	Settings2,
	SquareTerminal,
	Telescope,
	Tv,
} from "lucide-react";
import type { NavMainGroup } from "@/infra/routes/nav";

export const AppNavMainData: NavMainGroup[] = [
	{
		group: "Dashboard",
		items: [
			{
				title: "Explore",
				icon: Telescope,
				link: {
					to: "/explore",
				},
			},
			{
				title: "Subscriptions",
				link: {
					to: "/subscriptions/manage",
				},
				icon: Folders,
				children: [
					{
						title: "Manage",
						link: {
							to: "/subscriptions/manage",
						},
					},
					{
						title: "Create",
						link: {
							to: "/subscriptions/create",
						},
					},
				],
			},
			{
				title: "Bangumi",
				icon: Tv,
				children: [
					{
						title: "Manage",
						link: {
							to: "/bangumi/recorder",
						},
					},
					{
						title: "Feed",
						link: {
							to: "/bangumi/feed",
						},
					},
				],
			},
			{
				title: "Tasks",
				icon: ListTodo,
				children: [
					{
						title: "Tasks",
						link: {
							to: "/tasks/manage",
						},
					},
					{
						title: "Crons",
						link: {
							to: "/tasks/cron/manage",
						},
					},
				],
			},
			{
				title: "Credential 3rd",
				link: {
					to: "/credential3rd/manage",
				},
				icon: KeyRound,
				children: [
					{
						title: "Manage",
						link: {
							to: "/credential3rd/manage",
						},
					},
					{
						title: "Create",
						link: {
							to: "/credential3rd/create",
						},
					},
				],
			},
			{
				title: "Playground",
				icon: SquareTerminal,
				link: {
					to: "/playground",
				},
				children: [
					{
						title: "GraphQL Api",
						link: {
							to: "/playground/graphql-api",
						},
					},
				],
			},
			{
				title: "Documentation",
				link: {
					href: "https://github.com/apeiraco/konobangu/wiki",
					target: "_blank",
				},
				icon: BookOpen,
			},
			{
				title: "Settings",
				link: {
					to: "/settings",
				},
				icon: Settings2,
				children: [
					{
						title: "Downloader",
						link: {
							to: "/settings/downloader",
						},
					},
				],
			},
		],
	},
];
