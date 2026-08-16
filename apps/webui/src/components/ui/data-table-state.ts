import {
  type Column,
  type ColumnDef,
  type ColumnVisibilityState,
  columnPinningFeature,
  columnVisibilityFeature,
  functionalUpdate,
  type PaginationState,
  type ReactTable,
  type Row,
  type RowSelectionState,
  rowPaginationFeature,
  rowSelectionFeature,
  rowSortingFeature,
  type SortingState,
  tableFeatures,
  type Updater,
} from "@tanstack/react-table";
import { useCallback, useEffect, useState } from "react";

export const dataTableFeatures = tableFeatures({
  rowPaginationFeature,
  rowSortingFeature,
  columnVisibilityFeature,
  columnPinningFeature,
  rowSelectionFeature,
});
export type DataTableFeatures = typeof dataTableFeatures;
export type DataTable<TData extends object> = ReactTable<
  DataTableFeatures,
  TData
>;
export type DataTableColumn<TData extends object, TValue = unknown> = Column<
  DataTableFeatures,
  TData,
  TValue
>;
export type DataTableColumnDef<
  TData extends object,
  TValue = unknown,
> = ColumnDef<DataTableFeatures, TData, TValue>;

export function serverOrder<K extends string>(
  sorting: SortingState,
  fields: readonly K[],
  fallback: K,
) {
  const order: Partial<Record<K, "ASC" | "DESC">> = {};
  for (const sort of sorting) {
    const field = fields.find((field) => field === sort.id);
    if (field) order[field] = sort.desc ? "DESC" : "ASC";
  }
  if (!Object.keys(order).length) order[fallback] = "DESC";
  return order;
}

/** React owns presentation state; Apollo owns each server page of entities. */
export function useServerTableState(
  initialVisibility: ColumnVisibilityState = {},
) {
  const [columnVisibility, setColumnVisibility] = useState(initialVisibility);
  const [sorting, setSorting] = useState<SortingState>([]);
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  });
  const [rowSelection, setRowSelection] = useState<RowSelectionState>({});
  const [search, setSearch] = useState("");
  const onPaginationChange = useCallback(
    (updater: Updater<PaginationState>) => {
      setPagination((previous) => {
        const next = functionalUpdate(updater, previous);
        return next.pageSize !== previous.pageSize
          ? { ...next, pageIndex: 0 }
          : next;
      });
      setRowSelection({});
    },
    [],
  );
  const onSortingChange = useCallback((updater: Updater<SortingState>) => {
    setSorting((previous) => functionalUpdate(updater, previous));
    setPagination((previous) => ({ ...previous, pageIndex: 0 }));
    setRowSelection({});
  }, []);
  const onSearchChange = useCallback((value: string) => {
    setSearch(value);
    setPagination((previous) => ({ ...previous, pageIndex: 0 }));
    setRowSelection({});
  }, []);
  return {
    pagination,
    sorting,
    search,
    onSearchChange,
    tableOptions: {
      features: dataTableFeatures,
      getRowId: (row: { id: string | number }) => String(row.id),
      manualPagination: true,
      manualSorting: true,
      autoResetPageIndex: false,
      onPaginationChange,
      onSortingChange,
      onColumnVisibilityChange: setColumnVisibility,
      onRowSelectionChange: setRowSelection,
      state: { pagination, sorting, columnVisibility, rowSelection },
    },
    setPagination,
  };
}

export function useClampServerPage(
  state: ReturnType<typeof useServerTableState>,
  pages: number | null | undefined,
  loading: boolean,
) {
  const { pagination, setPagination } = state;
  useEffect(() => {
    if (
      !loading &&
      pages !== undefined &&
      pages !== null &&
      pagination.pageIndex >= Math.max(pages, 1)
    ) {
      setPagination((previous) => ({
        ...previous,
        pageIndex: Math.max(pages - 1, 0),
      }));
    }
  }, [loading, pages, pagination.pageIndex, setPagination]);
}

export type DataTableRow<TData extends object> = Row<DataTableFeatures, TData>;
