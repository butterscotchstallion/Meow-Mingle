import { Link } from "react-router-dom";
import { UserMenu } from "./UserMenu";
import { DebugMenu } from "./DebugMenu";

interface AppLayoutProps {
  children: React.ReactNode;
  mainClassName?: string;
}

export function AppLayout({ children, mainClassName }: AppLayoutProps) {
  return (
    <div className="flex flex-col min-h-screen bg-[#12071f]">
      <header className="flex items-center justify-between px-6 py-4 border-b border-purple-950">
        <Link
          to="/matches"
          className="text-xl font-bold text-purple-100 hover:text-purple-500 transition-colors no-underline"
        >
          🐱 Meow Mingle
        </Link>
        <div className="flex items-center gap-2">
          <DebugMenu />
          <UserMenu />
        </div>
      </header>
      <main className={mainClassName ?? "flex-1"}>
        {children}
      </main>
    </div>
  );
}
