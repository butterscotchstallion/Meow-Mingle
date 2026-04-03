import { useRef } from "react";
import { Menu } from "primereact/menu";
import { Button } from "primereact/button";
import type { MenuItem } from "primereact/menuitem";
import { useAuthStore } from "../store/authStore";

export function DebugMenu() {
  const menuRef = useRef<Menu>(null);
  const cat = useAuthStore((s) => s.cat);
  const roles = useAuthStore((s) => s.roles);

  const isAdmin = cat !== null && roles.some((r) => r.slug === "cat-admin");
  if (!isAdmin) return null;

  const menuItems: MenuItem[] = [
    {
      label: "Debug Menu",
      items: [
        {
          label: "Clear Matches",
          icon: "pi pi-trash",
          command: () => {},
        },
      ],
    },
  ];

  return (
    <>
      <Menu
        ref={menuRef}
        model={menuItems}
        popup
        id="debug-menu"
        aria-label="Debug menu"
      />
      <Button
        icon="pi pi-wrench"
        rounded
        text
        aria-label="Open debug menu"
        aria-haspopup
        title="Debug menu"
        aria-controls="debug-menu"
        onClick={(e) => menuRef.current?.toggle(e)}
        className="text-purple-400 hover:text-purple-200 transition-colors"
      />
    </>
  );
}
