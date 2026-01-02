import Image from "next/image";

interface LogoServerProps {
  width?: number;
  height?: number;
  className?: string;
}

export function LogoServer({ width = 32, height = 32, className }: LogoServerProps) {
  return (
    <div className={`relative ${className || ""}`} style={{ width, height }}>
      {/* Light theme logo (black) - visible by default and in light mode */}
      <Image
        src="/icon-light.svg"
        alt="Adventure Labs Logo"
        width={width}
        height={height}
        className="block dark:hidden"
        priority
      />

      {/* Dark theme logo (white) - visible only in dark mode */}
      <Image
        src="/icon-dark.svg"
        alt="Adventure Labs Logo"
        width={width}
        height={height}
        className="hidden dark:block"
        priority
      />
    </div>
  );
}

export default LogoServer;
