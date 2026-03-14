import type { CSSProperties } from "react";
import { Toaster as Sonner, type ToasterProps } from "sonner";
import { useTheme } from "@/infra/styles/context";

const Toaster = ({ ...props }: ToasterProps) => {
  const { colorTheme = "system" } = useTheme();

  return (
    <Sonner
      theme={colorTheme as ToasterProps["theme"]}
      className="toaster group"
      style={
        {
          "--normal-bg": "var(--popover)",
          "--normal-text": "var(--popover-foreground)",
          "--normal-border": "var(--border)",
        } as CSSProperties
      }
      {...props}
    />
  );
};

export { Toaster };
