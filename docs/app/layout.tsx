import { RootProvider } from "fumadocs-ui/provider/next";
import "./global.css";
import { Inter } from "next/font/google";
import type { Metadata } from "next";

const inter = Inter({
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  title: {
    template: "%s | Scout Docs",
    default: "Scout Docs",
  },
  description:
    "Documentation for Scout - Monitor and maintain remote hardware with real-time synchronization.",
  metadataBase: new URL("https://docs.adventurelabs.com"),
  icons: {
    icon: [
      { url: "/icon-light.svg", type: "image/svg+xml" },
      { url: "/favicon-32x32.png", sizes: "32x32", type: "image/png" },
      { url: "/favicon-16x16.png", sizes: "16x16", type: "image/png" },
    ],
    apple: "/apple-touch-icon.png",
  },
  openGraph: {
    title: "Scout Documentation",
    description:
      "Learn how to build with Scout for hardware monitoring and real-time state management.",
    type: "website",
    images: [
      {
        url: "/apple-touch-icon.png",
        width: 180,
        height: 180,
        alt: "Adventure Labs Scout Logo",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Scout Documentation",
    description:
      "Learn how to build with Scout for hardware monitoring and real-time state management.",
    images: ["/apple-touch-icon.png"],
  },
};

export default function Layout({ children }: LayoutProps<"/">) {
  return (
    <html lang="en" className={inter.className} suppressHydrationWarning>
      <body className="flex flex-col min-h-screen">
        <RootProvider
          theme={{
            attribute: "class",
            defaultTheme: "system",
            enableSystem: true,
          }}
        >
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
