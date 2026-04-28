import { useState } from "react";
import "./Sidebar.css";

type SidebarProps = {
  currentPage: string;
  onNavigate: (page: string) => void;
  theme: string;
  onThemeToggle: () => void;
};

const navItems = [
  { key: "dashboard", label: "Dashboard" },
  { key: "analytics", label: "Analytics" },
  { key: "history", label: "History" },
  { key: "settings", label: "Settings" },
];

const Icon = ({ name }: { name: string }) => {
  if (name === "dashboard") {
    return <svg viewBox="0 0 24 24"><path d="M3 12.5h8.5V3H3v9.5Zm9.5 8.5H21v-6.5h-8.5V21Zm0-8.5H21V3h-8.5v9.5ZM3 21h8.5v-6.5H3V21Z" /></svg>;
  }
  if (name === "analytics") {
    return <svg viewBox="0 0 24 24"><path d="M4 20h16v-2H4v2Zm2-4h3V8H6v8Zm5 0h3V4h-3v12Zm5 0h3v-6h-3v6Z" /></svg>;
  }
  if (name === "history") {
    return <svg viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9h-2a7 7 0 1 1-2.05-4.95L14 10h7V3l-2.63 2.63A8.96 8.96 0 0 0 12 3Zm-1 5v5.25l4.5 2.67 1-1.64-3.5-2.08V8h-2Z" /></svg>;
  }
  if (name === "theme") {
    return <svg viewBox="0 0 24 24"><path d="M12 4a8 8 0 1 0 8 8 6.5 6.5 0 0 1-8-8Z" /></svg>;
  }
  return <svg viewBox="0 0 24 24"><path d="M19.14 12.94a7.5 7.5 0 0 0 .05-.94 7.5 7.5 0 0 0-.05-.94l2.03-1.58a.5.5 0 0 0 .12-.64l-1.92-3.32a.5.5 0 0 0-.6-.22l-2.39.96a7.22 7.22 0 0 0-1.63-.94l-.36-2.54a.5.5 0 0 0-.5-.42h-3.84a.5.5 0 0 0-.5.42l-.36 2.54c-.58.23-1.12.54-1.63.94l-2.39-.96a.5.5 0 0 0-.6.22L2.7 8.84a.5.5 0 0 0 .12.64l2.03 1.58a7.5 7.5 0 0 0-.05.94 7.5 7.5 0 0 0 .05.94l-2.03 1.58a.5.5 0 0 0-.12.64l1.92 3.32a.5.5 0 0 0 .6.22l2.39-.96c.51.4 1.05.71 1.63.94l.36 2.54a.5.5 0 0 0 .5.42h3.84a.5.5 0 0 0 .5-.42l.36-2.54c.58-.23 1.12-.54 1.63-.94l2.39.96a.5.5 0 0 0 .6-.22l1.92-3.32a.5.5 0 0 0-.12-.64l-2.03-1.58ZM12 15.5A3.5 3.5 0 1 1 12 8a3.5 3.5 0 0 1 0 7.5Z" /></svg>;
};

export const Sidebar = ({ currentPage, onNavigate, theme, onThemeToggle }: SidebarProps) => {
  const [collapsed, setCollapsed] = useState(false);

  return (
    <aside className={`sidebar ${collapsed ? "sidebar-collapsed" : ""}`}>
      <button className="sidebar-collapse-btn" onClick={() => setCollapsed((prev) => !prev)}>
        <span>{collapsed ? ">" : "<"}</span>
      </button>

      <nav className="sidebar-nav">
        {navItems.map((item) => (
          <button
            key={item.key}
            className={`sidebar-nav-item ${currentPage === item.key ? "active" : ""}`}
            onClick={() => onNavigate(item.key)}
            title={collapsed ? item.label : ""}
          >
            <Icon name={item.key} />
            {!collapsed && <span>{item.label}</span>}
          </button>
        ))}
      </nav>

      <button
        className="sidebar-theme-btn"
        onClick={onThemeToggle}
        title={collapsed ? "Toggle Theme" : ""}
      >
        <Icon name="theme" />
        {!collapsed && <span>{theme === "dark" ? "Dark mode" : "Light mode"}</span>}
      </button>
    </aside>
  );
};
