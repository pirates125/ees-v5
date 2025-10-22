"use client";

import { useState, useEffect } from "react";
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
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { FileText, Search, Download } from "lucide-react";
import { toast } from "sonner";
import Link from "next/link";

import { apiClient } from "@/lib/api-client";

export default function TekliflerPage() {
  const [quotes, setQuotes] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [filter, setFilter] = useState({
    search: "",
    provider: "all",
    status: "all",
  });

  useEffect(() => {
    async function loadQuotes() {
      try {
        setIsLoading(true);
        const data = await apiClient.getQuotes();
        // Backend {quotes: [], total: 0} formatında dönüyor
        const quotesArray = Array.isArray(data)
          ? data
          : (data as any)?.quotes || [];
        setQuotes(quotesArray);
      } catch (error) {
        console.error("Teklifler yüklenemedi:", error);
        toast.error("Teklifler yüklenirken bir hata oluştu");
      } finally {
        setIsLoading(false);
      }
    }

    loadQuotes();
  }, []);

  const filteredQuotes = quotes.filter((quote) => {
    if (
      filter.search &&
      !quote.plateNumber.toLowerCase().includes(filter.search.toLowerCase())
    ) {
      return false;
    }
    if (filter.provider !== "all" && quote.provider !== filter.provider) {
      return false;
    }
    if (filter.status !== "all" && quote.status !== filter.status) {
      return false;
    }
    return true;
  });

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <FileText className="h-8 w-8" />
          <div>
            <h1 className="text-3xl font-bold">Tekliflerim</h1>
            <p className="text-muted-foreground">
              Geçmiş tekliflerinizi görüntüleyin
            </p>
          </div>
        </div>
      </div>

      {/* Filters */}
      <Card>
        <CardHeader>
          <CardTitle>Filtrele</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            <div>
              <Input
                placeholder="Plaka veya teklif no ile ara..."
                value={filter.search}
                onChange={(e) =>
                  setFilter({ ...filter, search: e.target.value })
                }
                className="w-full"
              />
            </div>

            <Select
              value={filter.provider}
              onValueChange={(value) =>
                setFilter({ ...filter, provider: value })
              }
            >
              <SelectTrigger>
                <SelectValue placeholder="Sigorta Şirketi" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">Tüm Şirketler</SelectItem>
                <SelectItem value="Sompo Sigorta">Sompo Sigorta</SelectItem>
                <SelectItem value="Quick Sigorta">Quick Sigorta</SelectItem>
                <SelectItem value="Axa Sigorta">Axa Sigorta</SelectItem>
                <SelectItem value="Anadolu Sigorta">Anadolu Sigorta</SelectItem>
              </SelectContent>
            </Select>

            <Select
              value={filter.status}
              onValueChange={(value) => setFilter({ ...filter, status: value })}
            >
              <SelectTrigger>
                <SelectValue placeholder="Durum" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">Tümü</SelectItem>
                <SelectItem value="active">Aktif</SelectItem>
                <SelectItem value="expired">Süresi Dolmuş</SelectItem>
                <SelectItem value="converted">Poliçeye Dönüştürüldü</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      {/* Quotes Table */}
      <Card>
        <CardHeader>
          <CardTitle>Teklifler ({filteredQuotes.length})</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-center py-12">
              <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-current border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]" />
              <p className="mt-4 text-muted-foreground">
                Teklifler yükleniyor...
              </p>
            </div>
          ) : filteredQuotes.length === 0 ? (
            <div className="text-center py-12">
              <FileText className="mx-auto h-12 w-12 text-muted-foreground" />
              <h3 className="mt-4 text-lg font-semibold">Henüz teklif yok</h3>
              <p className="text-sm text-muted-foreground mt-2">
                Teklif almaya başlamak için sigorta formlarını doldurun
              </p>
              <div className="mt-6">
                <Button asChild>
                  <Link href="/trafik">Trafik Sigortası Teklifi Al</Link>
                </Button>
              </div>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Teklif No</TableHead>
                  <TableHead>Sigorta Şirketi</TableHead>
                  <TableHead>Ürün Tipi</TableHead>
                  <TableHead>Plaka</TableHead>
                  <TableHead>Prim</TableHead>
                  <TableHead>Durum</TableHead>
                  <TableHead>Tarih</TableHead>
                  <TableHead>İşlem</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredQuotes.map((quote) => (
                  <TableRow key={quote.id}>
                    <TableCell className="font-mono text-sm">
                      #{quote.id}
                    </TableCell>
                    <TableCell>{quote.provider}</TableCell>
                    <TableCell>{quote.productType}</TableCell>
                    <TableCell className="font-mono">
                      {quote.plateNumber}
                    </TableCell>
                    <TableCell className="font-semibold">
                      ₺{quote.grossPremium.toLocaleString("tr-TR")}
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={
                          quote.status === "active"
                            ? "default"
                            : quote.status === "expired"
                            ? "secondary"
                            : "outline"
                        }
                      >
                        {quote.status === "active"
                          ? "Aktif"
                          : quote.status === "expired"
                          ? "Süresi Dolmuş"
                          : "Poliçeye Dönüştürüldü"}
                      </Badge>
                    </TableCell>
                    <TableCell>{quote.createdAt}</TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          toast.info("Teklif detayı yakında eklenecek")
                        }
                      >
                        <Download className="h-4 w-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
