export function AppSkeleton() {
  return (
    <div className="flex min-h-svh flex-1 flex-col gap-4 md:min-h-0">
      <div className="grid auto-rows-min gap-4 md:grid-cols-3">
        <div className="aspect-video rounded-xl bg-muted/50" />
        <div className="aspect-video rounded-xl bg-muted/50" />
        <div className="aspect-video rounded-xl bg-muted/50" />
      </div>
      <div className="min-h-svh flex-1 rounded-xl bg-muted/50 md:min-h-0" />
    </div>
  );
}
