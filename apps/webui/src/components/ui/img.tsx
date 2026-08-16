import { type ComponentProps, useMemo } from "react";
import { useInject } from "@/infra/di/inject";
import { DOCUMENT } from "@/infra/platform/injection";

export function optimizedImageUrl(
  src: string | undefined,
  baseURI: string | undefined,
  optimize: "accept",
) {
  if (!src || !baseURI) return src;
  try {
    const base = new URL(baseURI);
    const url = new URL(src, base);
    if (
      url.origin !== base.origin ||
      !url.pathname.startsWith("/api/static/") ||
      !["http:", "https:"].includes(url.protocol)
    )
      return src;
    url.searchParams.set("optimize", optimize);
    return url.toString();
  } catch {
    return src;
  }
}

export type ImgProps = Omit<ComponentProps<"img">, "alt"> &
  Required<Pick<ComponentProps<"img">, "alt">> & {
    optimize?: "accept";
  };

export const Img = ({
  src: propsSrc,
  optimize = "accept",
  ...props
}: ImgProps) => {
  const document = useInject(DOCUMENT);
  const src = useMemo(() => {
    return optimizedImageUrl(propsSrc, document?.baseURI, optimize);
  }, [propsSrc, optimize, document?.baseURI]);

  return <img {...props} alt={props.alt} src={src} />;
};
