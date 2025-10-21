"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { ThemeToggle } from "@/components/theme-toggle";
import { useAuth } from "@/lib/auth-context";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  LayoutDashboard,
  FileText,
  Car,
  Shield,
  Home,
  Heart,
  Settings,
  UserCog,
  LogOut,
  Menu,
  X,
} from "lucide-react";

const navigation = [
  { name: "Dashboard", href: "/dashboard", icon: LayoutDashboard },
  { name: "Teklifler", href: "/teklifler", icon: FileText },
  { name: "Poliçelerim", href: "/policeler", icon: Shield },
  { name: "Trafik Sigortası", href: "/trafik", icon: Car },
  { name: "Kasko", href: "/kasko", icon: Shield },
  { name: "Konut", href: "/konut", icon: Home },
  { name: "Sağlık", href: "/saglik", icon: Heart },
  { name: "Admin", href: "/admin", icon: UserCog, adminOnly: true },
  { name: "Ayarlar", href: "/ayarlar", icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();
  const { user, logout } = useAuth();
  const [isOpen, setIsOpen] = useState(false);

  const toggleSidebar = () => setIsOpen(!isOpen);
  const closeSidebar = () => setIsOpen(false);

  return (
    <>
      {/* Mobile Hamburger Button */}
      <button
        onClick={toggleSidebar}
        className="fixed top-4 left-4 z-50 lg:hidden p-2 rounded-md bg-card border shadow-sm hover:bg-accent"
        aria-label="Toggle menu"
      >
        {isOpen ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
      </button>

      {/* Overlay for mobile */}
      {isOpen && (
        <div
          className="fixed inset-0 bg-black/50 z-40 lg:hidden"
          onClick={closeSidebar}
        />
      )}

      {/* Sidebar */}
      <aside
        className={cn(
          "fixed lg:relative top-0 left-0 z-40 h-full w-64 flex flex-col border-r bg-card shadow-lg lg:shadow-none transition-transform duration-300 lg:translate-x-0",
          isOpen ? "translate-x-0" : "-translate-x-full"
        )}
      >
        {/* Logo & Theme Toggle */}
        <div className="flex h-16 items-center justify-between border-b px-6">
          <h1 className="text-xl font-bold">EE Sigorta</h1>
          <ThemeToggle />
        </div>

        {/* User Info */}
        {user && (
          <div className="border-b p-4">
            <div className="flex items-center gap-3">
              <Avatar className="h-10 w-10">
                <AvatarFallback className="bg-primary text-primary-foreground">
                  {user.name
                    .split(" ")
                    .map((n) => n[0])
                    .join("")
                    .toUpperCase()
                    .slice(0, 2)}
                </AvatarFallback>
              </Avatar>
              <div className="flex-1 overflow-hidden">
                <p className="truncate text-sm font-medium">{user.name}</p>
                <p className="truncate text-xs text-muted-foreground">
                  {user.email}
                </p>
              </div>
            </div>
            {user.role === "admin" && (
              <div className="mt-2">
                <span className="inline-flex rounded-full bg-amber-500 px-2 py-0.5 text-xs text-white">
                  Admin
                </span>
              </div>
            )}
          </div>
        )}

        {/* Navigation */}
        <nav className="flex-1 space-y-1 p-4">
          {navigation.map((item) => {
            // Hide admin link if not admin
            if (item.adminOnly && user?.role !== "admin") {
              return null;
            }

            const isActive = pathname === item.href;
            const Icon = item.icon;

            return (
              <Link
                key={item.name}
                href={item.href}
                onClick={closeSidebar}
                className={cn(
                  "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                <Icon className="h-5 w-5" />
                {item.name}
              </Link>
            );
          })}
        </nav>

        {/* Logout Button */}
        <div className="border-t p-4 space-y-3">
          <Button
            variant="outline"
            className="w-full justify-start gap-3"
            onClick={logout}
          >
            <LogOut className="h-4 w-4" />
            Çıkış Yap
          </Button>
          <p className="text-xs text-muted-foreground">
            © 2024 EE Sigorta
            <br />
            v0.1.0
          </p>
        </div>
      </aside>
    </>
  );
}
