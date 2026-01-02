import {
  type LucideIcon,
  NetworkIcon,
  DatabaseIcon,
  CloudSync,
} from "lucide-react";
import type { LinkProps } from "next/link";
import Link from "next/link";
import type { ReactElement, ReactNode } from "react";
import { cn } from "@/lib/cn";
import { LogoServer } from "@/components/logo-server";

export default function HomePage() {
  return (
    <div className="flex flex-col justify-center text-center flex-1 px-4">
      <div className="flex flex-col items-center mb-8">
        <LogoServer width={120} height={120} className="mb-6" />
        <h1 className="text-5xl font-bold mb-6 bg-linear-to-r from-fd-foreground to-fd-muted-foreground bg-clip-text text-transparent">
          Build with Scout
        </h1>
      </div>
      <p className="max-w-2xl mx-auto text-lg text-fd-muted-foreground leading-relaxed">
        Learn how to monitor and maintain remote hardware with Scout&apos;s
        powerful real-time synchronization platform.
      </p>
      <div className="mt-12 grid grid-cols-1 gap-6 text-left md:grid-cols-3 max-w-5xl mx-auto">
        <DocumentationItem
          description="Realtime state updates in your web applications."
          href="/docs/scout-core"
          icon={{ icon: NetworkIcon, id: "scout-core" }}
          title="Scout Core"
        />
        <DocumentationItem
          description="Rust client for synchronizing local state with a remote database."
          href="/docs/scout-rs"
          icon={{ icon: CloudSync, id: "scout-rs" }}
          title="Scout RS"
        />
        <DocumentationItem
          description="Database management and configuration for Scout applications."
          href="/docs/scout-db"
          icon={{ icon: DatabaseIcon, id: "scout-db" }}
          title="Scout DB"
        />
      </div>
    </div>
  );
}

function DocumentationItem({
  title,
  description,
  icon: { icon: ItemIcon, id },
  href,
}: {
  title: string;
  description: string;
  icon: {
    icon: LucideIcon;
    id: string;
  };
  href: string;
}): ReactElement {
  return (
    <Item href={href}>
      <IconWrapper className={id}>
        <ItemIcon className="size-full" />
      </IconWrapper>
      <h2 className="mb-3 font-semibold text-xl text-fd-foreground">{title}</h2>
      <p className="text-fd-muted-foreground text-sm leading-relaxed">
        {description}
      </p>
    </Item>
  );
}

function IconWrapper({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}): ReactElement {
  return (
    <div
      className={cn(
        "mb-4 size-10 rounded-xl border border-fd-border p-2 shadow-sm transition-all duration-200",
        "bg-gradient-to-br from-fd-card to-fd-muted",
        "group-hover:shadow-lg group-hover:scale-105",
        className,
      )}
      style={{
        boxShadow: "0 2px 8px hsla(210, 85%, 45%, 0.1)",
      }}
    >
      {children}
    </div>
  );
}

function Item(
  props: LinkProps & { className?: string; children: ReactNode },
): ReactElement {
  const { className, children, ...rest } = props;
  return (
    <Link
      {...rest}
      className={cn(
        "group rounded-2xl border border-fd-border bg-fd-card/50 p-8 shadow-sm backdrop-blur-sm transition-all duration-300",
        "hover:bg-fd-card hover:shadow-xl hover:border-fd-primary/30 hover:-translate-y-1",
        "focus-visible:ring-2 focus-visible:ring-fd-ring focus-visible:outline-none",
        className,
      )}
    >
      {children}
    </Link>
  );
}
