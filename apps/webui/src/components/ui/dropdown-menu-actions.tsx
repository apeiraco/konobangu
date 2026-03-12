"use client";

import type * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { Eye, MoreHorizontal } from "lucide-react";
import { ComponentProps, PropsWithChildren } from "react";
import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuShortcut,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface DropdownMenuActionsProps<Id>
	extends ComponentProps<typeof DropdownMenuPrimitive.Root> {
	id: Id;
	showDetail?: boolean | "dropdown-menu";
	showEdit?: boolean;
	showDelete?: boolean;
	onDetail?: (id: Id) => void;
	onDelete?: (id: Id) => void;
	onEdit?: (id: Id) => void;
}

export function DropdownMenuActions<Id>({
	id,
	showDetail,
	showDelete,
	showEdit,
	onDetail,
	onDelete,
	onEdit,
	children,
	...rest
}: PropsWithChildren<DropdownMenuActionsProps<Id>>) {
	return (
		<div className="flex gap-2 items-center justify-center">
			{showDetail === true && (
				<Button
					variant="ghost"
					className="flex h-8 w-8 p-0 data-[state=open]:bg-muted"
					onClick={() => onDetail?.(id)}
				>
					<Eye />
					<span className="sr-only">Detail</span>
				</Button>
			)}
			<DropdownMenu {...rest}>
				<DropdownMenuTrigger asChild>
					<Button
						variant="ghost"
						className="flex h-8 w-8 p-0 data-[state=open]:bg-muted"
					>
						<MoreHorizontal />
						<span className="sr-only">Open menu</span>
					</Button>
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end" className="w-[160px]">
					{children}
					{showDetail === "dropdown-menu" && (
						<DropdownMenuItem onClick={() => onDetail?.(id)}>
							Detail
						</DropdownMenuItem>
					)}
					{showEdit && (
						<DropdownMenuItem onClick={() => onEdit?.(id)}>
							Edit
						</DropdownMenuItem>
					)}
					{(showDetail === "dropdown-menu" || showEdit || children) &&
						showDelete && <DropdownMenuSeparator />}
					{showDelete && (
						<DropdownMenuItem onClick={() => onDelete?.(id)}>
							Delete
							<DropdownMenuShortcut>⌘⌫</DropdownMenuShortcut>
						</DropdownMenuItem>
					)}
				</DropdownMenuContent>
			</DropdownMenu>
		</div>
	);
}
