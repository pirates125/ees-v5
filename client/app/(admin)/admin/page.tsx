"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Users, FileText, Shield, TrendingUp, Loader2 } from "lucide-react";
import { apiClient } from "@/lib/api-client";
import { toast } from "sonner";

export default function AdminDashboardPage() {
  const [stats, setStats] = useState({
    totalUsers: 0,
    totalQuotes: 0,
    totalPolicies: 0,
    totalRevenue: 0,
    totalCommission: 0,
  });
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const data = await apiClient.getAdminStats();
        setStats(
          data as any as {
            totalUsers: number;
            totalQuotes: number;
            totalPolicies: number;
            totalRevenue: number;
            totalCommission: number;
          }
        );
      } catch (error) {
        console.error("Failed to fetch stats:", error);
        toast.error("İstatistikler yüklenemedi");
      } finally {
        setIsLoading(false);
      }
    };

    fetchStats();
  }, []);

  const statsCards = [
    {
      title: "Toplam Kullanıcı",
      value: stats.totalUsers,
      icon: Users,
      trend: "+12%",
    },
    {
      title: "Toplam Teklif",
      value: stats.totalQuotes,
      icon: FileText,
      trend: "+8%",
    },
    {
      title: "Toplam Poliçe",
      value: stats.totalPolicies,
      icon: Shield,
      trend: "+15%",
    },
    {
      title: "Toplam Gelir",
      value: `₺${stats.totalRevenue.toLocaleString("tr-TR")}`,
      icon: TrendingUp,
      trend: "+23%",
    },
  ];

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Admin Dashboard</h1>
        <p className="text-muted-foreground">
          Sistem genelindeki istatistikleri ve kullanıcı işlemlerini
          görüntüleyin
        </p>
      </div>

      {/* Stats Grid */}
      {isLoading ? (
        <div className="flex items-center justify-center p-12">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
          <span className="ml-2 text-muted-foreground">Yükleniyor...</span>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {statsCards.map((stat) => {
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
                    <span className="text-green-600">{stat.trend}</span> önceki
                    aya göre
                  </p>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      {/* Recent Activity */}
      <Card>
        <CardHeader>
          <CardTitle>Son Kullanıcı İşlemleri</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Kullanıcı</TableHead>
                <TableHead>İşlem</TableHead>
                <TableHead>Provider</TableHead>
                <TableHead>Tutar</TableHead>
                <TableHead>Durum</TableHead>
                <TableHead>Tarih</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow>
                <TableCell
                  className="text-center text-muted-foreground"
                  colSpan={6}
                >
                  Henüz işlem kaydı yok
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}
