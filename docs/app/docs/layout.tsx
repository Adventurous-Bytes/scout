import { source } from "@/lib/source";
import { DocsLayout } from "fumadocs-ui/layouts/docs";
import { baseOptions } from "@/lib/layout.shared";
import { NetworkIcon, DatabaseIcon, CloudSync } from "lucide-react";

export default function Layout({ children }: LayoutProps<"/docs">) {
  return (
    <DocsLayout
      tree={source.getPageTree()}
      {...baseOptions()}
      sidebar={{
        tabs: [
          {
            title: "Scout Core",
            description: "Realtime state updates in your web applications",
            url: "/docs/scout-core",
            icon: <NetworkIcon className="size-4" />,
          },
          {
            title: "Scout RS",
            description:
              "Rust client for synchronizing local state with a remote database",
            url: "/docs/scout-rs",
            icon: <CloudSync className="size-4" />,
          },
          {
            title: "Scout DB",
            description: "Database management and configuration for Scout",
            url: "/docs/scout-db",
            icon: <DatabaseIcon className="size-4" />,
          },
        ],
      }}
    >
      {children}
    </DocsLayout>
  );
}
