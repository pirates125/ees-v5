"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { FileText, Download, Search, Calendar, Shield } from "lucide-react";
import { toast } from "sonner";
import Link from "next/link";

interface Policy {
  id: string;
  policyNumber: string;
  provider: string;
  productType: string;
  status: string;
  grossPremium: number;
  createdAt: string;
}

export default function PoliciesPage() {
  const [policies, setPolicies] = useState<Policy[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState("");

  useEffect(() => {
    loadPolicies();
  }, []);

  const loadPolicies = async () => {
    try {
      setIsLoading(true);
      const token = localStorage.getItem("auth_token");

      if (!token) {
        toast.error("Oturum süresi dolmuş, lütfen giriş yapın");
        return;
      }

      const response = await fetch("http://localhost:8099/api/v1/policies", {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setPolicies(data);
      } else {
        toast.error("Poliçeler yüklenemedi");
      }
    } catch (error) {
      toast.error("Bağlantı hatası");
      console.error(error);
    } finally {
      setIsLoading(false);
    }
  };

  const filteredPolicies = policies.filter(
    (p) =>
      p.policyNumber.toLowerCase().includes(searchTerm.toLowerCase()) ||
      p.provider.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "active":
        return <Badge variant="success">Aktif</Badge>;
      case "expired":
        return <Badge variant="secondary">Süresi Dolmuş</Badge>;
      case "cancelled":
        return <Badge variant="destructive">İptal Edilmiş</Badge>;
      default:
        return <Badge>{status}</Badge>;
    }
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">Poliçelerim</h1>
        <p className="text-muted-foreground">
          Tüm sigorta poliçelerinizi görüntüleyin
        </p>
      </div>

      {/* Search */}
      <div className="flex items-center gap-2">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Poliçe numarası veya şirket ara..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-9"
          />
        </div>
      </div>

      {/* Policies List */}
      {isLoading ? (
        <Card>
          <CardContent className="pt-6">
            <p className="text-center text-muted-foreground">
              Poliçeler yükleniyor...
            </p>
          </CardContent>
        </Card>
      ) : filteredPolicies.length === 0 ? (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center py-8">
              <Shield className="h-16 w-16 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold mb-2">Henüz poliçe yok</h3>
              <p className="text-sm text-muted-foreground mb-4">
                İlk poliçenizi oluşturmak için bir teklif alın
              </p>
              <Button asChild>
                <Link href="/trafik">Teklif Al</Link>
              </Button>
            </div>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {filteredPolicies.map((policy) => (
            <Card key={policy.id}>
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <div>
                    <CardTitle className="text-lg flex items-center gap-2">
                      <FileText className="h-5 w-5" />
                      {policy.policyNumber}
                    </CardTitle>
                    <p className="text-sm text-muted-foreground mt-1">
                      {policy.provider} - {policy.productType}
                    </p>
                  </div>
                  {getStatusBadge(policy.status)}
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div>
                    <p className="text-sm text-muted-foreground">Prim Tutarı</p>
                    <p className="text-lg font-semibold">
                      {policy.grossPremium.toLocaleString("tr-TR")} ₺
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-muted-foreground">
                      Oluşturulma Tarihi
                    </p>
                    <p className="text-sm flex items-center gap-1">
                      <Calendar className="h-3 w-3" />
                      {new Date(policy.createdAt).toLocaleDateString("tr-TR")}
                    </p>
                  </div>
                  <div className="flex items-end justify-end gap-2">
                    <Button variant="outline" size="sm">
                      <Download className="h-4 w-4 mr-2" />
                      PDF İndir
                    </Button>
                    <Button variant="outline" size="sm">
                      Detaylar
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Stats */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Toplam Poliçe
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{policies.length}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Aktif Poliçe
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {policies.filter((p) => p.status === "active").length}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Toplam Prim
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {policies
                .reduce((sum, p) => sum + p.grossPremium, 0)
                .toLocaleString("tr-TR")}{" "}
              ₺
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
