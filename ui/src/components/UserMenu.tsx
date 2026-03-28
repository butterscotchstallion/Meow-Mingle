import { useRef } from "react";
import { useNavigate } from "react-router-dom";
import { Menu } from "primereact/menu";
import { Button } from "primereact/button";
import { Avatar } from "primereact/avatar";
import type { MenuItem } from "primereact/menuitem";
import { useAuthStore } from "../store/authStore";

export function UserMenu() {
  const menuRef = useRef<Menu>(null);
  const navigate = useNavigate();
  const cat = useAuthStore((s) => s.cat);
  const clearAuth = useAuthStore((s) => s.clearAuth);

  function handleSignOut() {
    clearAuth();
    navigate("/signin");
  }

  const menuItems: MenuItem[] = [
    {
      label: cat?.name ?? "My Account",
      items: [
        {
          label: "Edit Profile",
          icon: "pi pi-user-edit",
          command: () => navigate("/edit-profile"),
        },
        {
          label: "Sign Out",
          icon: "pi pi-sign-out",
          command: handleSignOut,
        },
      ],
    },
  ];

  if (!cat) {
    return (
      <Button
        label="Sign In"
        icon="pi pi-sign-in"
        size="small"
        onClick={() => navigate("/signin")}
        className="p-button-outlined"
      />
    );
  }

  const avatarUrl = cat.avatarFilename
    ? `/images/cats/${cat.avatarFilename}`
    : undefined;

  return (
    <>
      <Menu
        ref={menuRef}
        model={menuItems}
        popup
        id="user-menu"
        aria-label="User menu"
      />
      <button
        type="button"
        aria-label="Open user menu"
        aria-haspopup
        aria-controls="user-menu"
        onClick={(e) => menuRef.current?.toggle(e)}
        className="rounded-full p-0 border-2 border-transparent hover:border-purple-500 focus:border-purple-500 focus:outline-none transition-colors cursor-pointer bg-transparent"
        style={{ lineHeight: 0 }}
      >
        {avatarUrl ? (
          <Avatar
            image={avatarUrl}
            shape="circle"
            size="normal"
            style={{ width: 40, height: 40 }}
          />
        ) : (
          <Avatar
            label={cat.name.charAt(0).toUpperCase()}
            shape="circle"
            size="normal"
            style={{
              width: 40,
              height: 40,
              backgroundColor: "#6b21a8",
              color: "#fff",
            }}
          />
        )}
      </button>
    </>
  );
}
