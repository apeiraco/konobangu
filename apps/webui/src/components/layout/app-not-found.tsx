import {
  type AnyRoute,
  type ParsedLocation,
  redirect,
} from "@tanstack/react-router";
import { ProLink } from "../ui/pro-link";

export function guardRouteIndexAsNotFound(
  this: AnyRoute,
  { location }: { location: ParsedLocation<any> },
) {
  if (location.pathname.replace(/\/+$/, "") === this.id) {
    throw redirect({
      href: "/404",
      replace: true,
      reloadDocument: true,
    });
  }
}

export function AppNotFoundComponent() {
  return (
    <div className="flex h-svh items-center px-4 py-12 sm:px-6 md:px-8 lg:px-12 xl:px-16">
      <div className="w-full space-y-6 text-center">
        <div className="space-y-3">
          <h1 className="font-bold text-4xl tracking-tighter sm:text-5xl">
            404 Page Not Found
          </h1>
          <p className="text-gray-500">
            Sorry, we couldn&#x27;t find the page you&#x27;re looking for.
          </p>
        </div>
        <ProLink
          to="/"
          className="inline-flex h-10 items-center rounded-md border border-gray-20 bg-white px-8 font-medium text-xs shadow-xs transition-colors hover:bg-gray-100 hover:text-gray-900 dark:border-gray-800 dark:bg-gray-950 dark:focus-visible:ring-gray-300 dark:hover:bg-gray-800 dark:hover:text-gray-50"
        >
          Return to website
        </ProLink>
      </div>
    </div>
  );
}
