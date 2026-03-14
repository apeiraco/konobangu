import { type ComponentProps, useMemo } from "react";
import { useInject } from "@/infra/di/inject";
import { DOCUMENT } from "@/infra/platform/injection";

const URL_PARSE_REGEX = /^([^?#]*)(\?[^#]*)?(#.*)?$/;

function parseURL(url: string) {
  const match = url.match(URL_PARSE_REGEX);

  if (!match) {
    return { other: url, search: "", hash: "" };
  }

  return {
    other: match[1] || "",
    search: match[2] || "",
    hash: match[3] || "",
  };
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
    const baseURI = document?.baseURI;
    if (!propsSrc || !baseURI) {
      return propsSrc;
    }
    const { other, search, hash } = parseURL(propsSrc);
    const searchParams = new URLSearchParams(search);
    searchParams.set("optimize", optimize);
    return `${other}?${searchParams.toString()}${hash}`;
  }, [propsSrc, optimize, document?.baseURI]);

  return <img {...props} alt={props.alt} src={src} />;
};
