"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ProviderGrid } from "@/components/provider-status";
import { apiClient } from "@/lib/api-client";
import { useAuth } from "@/lib/auth-context";
import {
  FileText,
  TrendingUp,
  Wallet,
  Users,
  Shield,
  Clock,
} from "lucide-react";
import { ProviderInfo } from "@/lib/types";

export default function DashboardPage() {
  const { user } = useAuth();
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isLoadingProviders, setIsLoadingProviders] = useState(true);
  const [isLoadingStats, setIsLoadingStats] = useState(false);
  const [adminStats, setAdminStats] = useState<any>(null);

  useEffect(() => {
    async function loadProviders() {
      try {
        const data = await apiClient.getProviders();
        setProviders(data.providers);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Provider listesi alınamadı");
      } finally {
        setIsLoadingProviders(false);
      }
    }

    loadProviders();
  }, []);

  // Admin için stats yükle
  useEffect(() => {
    async function loadAdminStats() {
      if (user?.role === "admin") {
        setIsLoadingStats(true);
        try {
          const stats = await apiClient.getAdminStats();
          setAdminStats(stats);
        } catch (e) {
          console.error("Admin stats yüklenemedi:", e);
        } finally {
          setIsLoadingStats(false);
        }
      }
    }

    loadAdminStats();
  }, [user]);

  // Role-based stats
  const getStats = () => {
    if (user?.role === "admin" && adminStats) {
      return [
        {
          title: "Toplam Kullanıcı",
          value: adminStats.totalUsers || "0",
          icon: Users,
          description: "Kayıtlı kullanıcılar",
        },
        {
          title: "Toplam Teklif",
          value: adminStats.totalQuotes || "0",
          icon: FileText,
          description: "Tüm teklifler",
        },
        {
          title: "Toplam Poliçe",
          value: adminStats.totalPolicies || "0",
          icon: Shield,
          description: "Kesilen poliçeler",
        },
        {
          title: "Toplam Gelir",
          value: `₺${(adminStats.totalRevenue || 0).toLocaleString("tr-TR")}`,
          icon: Wallet,
          description: "Tüm zamanlar",
        },
      ];
    }

    // Normal kullanıcı stats
    return [
      {
        title: "Tekliflerim",
        value: "0",
        icon: FileText,
        description: "Toplam teklifler",
      },
      {
        title: "Poliçelerim",
        value: "0",
        icon: Shield,
        description: "Aktif poliçeler",
      },
      {
        title: "Bekleyen",
        value: "0",
        icon: Clock,
        description: "İşlem bekleyen",
      },
      {
        title: "Aktif Provider",
        value: providers.filter((p) => p.active).length.toString(),
        icon: TrendingUp,
        description: `${providers.length} toplam`,
      },
    ];
  };

  const stats = getStats();

  return (
    <div className="container mx-auto p-6 space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">
          {user?.role === "admin" ? "Admin Dashboard" : "Dashboard"}
        </h1>
        <p className="text-muted-foreground">
          {user?.role === "admin"
            ? "Sistem genelindeki istatistikleri görüntüleyin"
            : "Sigorta teklif platformuna hoş geldiniz"}
        </p>
      </div>

      {/* Error Alert */}
      {error && (
        <Card className="border-destructive">
          <CardContent className="pt-6">
            <p className="text-sm text-destructive">
              ⚠️ Backend bağlantısı kurulamadı: {error}
            </p>
            <p className="text-xs text-muted-foreground mt-2">
              Lütfen Rust server'ın çalıştığından emin olun (localhost:8099)
            </p>
          </CardContent>
        </Card>
      )}

      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => {
          const Icon = stat.icon;
          return (
            <Card key={stat.title}>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  {stat.title}
                </CardTitle>
                <Icon className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stat.value}</div>
                <p className="text-xs text-muted-foreground">
                  {stat.description}
                </p>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Provider Status Grid */}
      <div>
        <h2 className="text-xl font-semibold mb-4">Provider Durumu</h2>
        {isLoadingProviders ? (
          <Card>
            <CardContent className="pt-6">
              <div className="flex items-center justify-center gap-2">
                <div className="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                <p className="text-muted-foreground">
                  Provider bilgisi yükleniyor...
                </p>
              </div>
            </CardContent>
          </Card>
        ) : providers.length > 0 ? (
          <ProviderGrid providers={providers} />
        ) : (
          <Card>
            <CardContent className="pt-6">
              <p className="text-center text-muted-foreground">
                Provider bulunamadı
              </p>
            </CardContent>
          </Card>
        )}
      </div>

      {/* Quick Actions */}
      <div>
        <h2 className="text-xl font-semibold mb-4">Hızlı İşlemler</h2>
        <div className="grid gap-4 md:grid-cols-3">
          <a href="/trafik">
            <Card className="cursor-pointer hover:bg-accent transition-colors h-full">
              <CardHeader>
                <CardTitle className="text-base">Trafik Sigortası</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">
                  Yeni trafik sigortası teklifi al
                </p>
              </CardContent>
            </Card>
          </a>
          <a href="/kasko">
            <Card className="cursor-pointer hover:bg-accent transition-colors h-full">
              <CardHeader>
                <CardTitle className="text-base">Kasko</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">
                  Kasko sigortası teklifi al
                </p>
              </CardContent>
            </Card>
          </a>
          <a href="/teklifler">
            <Card className="cursor-pointer hover:bg-accent transition-colors h-full">
              <CardHeader>
                <CardTitle className="text-base">
                  Teklifleri Görüntüle
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">
                  Tüm teklifleri listele
                </p>
              </CardContent>
            </Card>
          </a>
        </div>
      </div>

      {/* Info Box */}
      <Card className="border-amber-200 bg-amber-50 dark:bg-amber-950 dark:border-amber-800">
        <CardContent className="pt-6">
          <div className="flex gap-3">
            <div className="text-amber-600 dark:text-amber-400">ℹ️</div>
            <div>
              <h3 className="font-semibold text-amber-900 dark:text-amber-100">
                Karşılaştırmalı Teklif Sistemi
              </h3>
              <p className="text-sm text-amber-800 dark:text-amber-200 mt-1">
                Trafik sigortası sayfasında "Karşılaştırmalı Teklif Al" seçeneği
                ile tüm aktif sigorta şirketlerinden paralel olarak teklif
                alabilirsiniz.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
