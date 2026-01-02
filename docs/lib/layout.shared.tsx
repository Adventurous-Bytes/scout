import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { LogoServer } from "@/components/logo-server";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <div className="flex items-center gap-3">
          <LogoServer width={52} height={52} className="pb-16" />
          <span className="font-semibold">Scout Docs</span>
        </div>
      ),
      transparentMode: "top",
    },
    githubUrl: "https://github.com/Adventurous-Bytes/scout",
    links: [],
  };
}
