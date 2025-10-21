"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { QuoteComparison } from "@/components/quote-comparison";
import { PolicyDialog } from "@/components/policy-dialog";
import { trafikFormSchema, TrafikFormData } from "@/lib/schema";
import { apiClient } from "@/lib/api-client";
import { QuoteResponse } from "@/lib/types";
import { generateRequestId } from "@/lib/utils";
import { Loader2, Search } from "lucide-react";
import { toast } from "sonner";

export default function TrafikPage() {
  const [isLoading, setIsLoading] = useState(false);
  const [quotes, setQuotes] = useState<QuoteResponse[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [compareMode, setCompareMode] = useState(true);
  const [selectedQuote, setSelectedQuote] = useState<QuoteResponse | null>(
    null
  );
  const [policyDialogOpen, setPolicyDialogOpen] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<TrafikFormData>({
    resolver: zodResolver(trafikFormSchema),
    defaultValues: {
      usage: "hususi",
      year: new Date().getFullYear(),
      startDate: new Date().toISOString().split("T")[0],
      addons: [],
    },
  });

  const onSubmit = async (data: TrafikFormData) => {
    setIsLoading(true);
    setError(null);
    setQuotes([]);

    try {
      const request = {
        insured: {
          tckn: data.tckn,
          name: data.name,
          birthDate: data.birthDate,
          phone: data.phone,
          email: data.email,
        },
        vehicle: {
          plate: data.plate,
          brand: data.brand,
          model: data.model,
          year: data.year,
          usage: data.usage,
        },
        coverage: {
          productType: "trafik" as const,
          startDate: data.startDate,
          addons: data.addons || [],
        },
        quoteMeta: {
          requestId: generateRequestId(),
        },
      };

      if (compareMode) {
        // Tüm provider'lardan karşılaştırmalı teklif al
        toast.info("Tüm sigorta şirketlerinden teklif alınıyor...");
        const results = await apiClient.requestAllQuotes(request);
        setQuotes(Array.isArray(results) ? results : [results]);
        toast.success(
          `${Array.isArray(results) ? results.length : 1} teklif alındı!`
        );
      } else {
        // Sadece Sompo'dan teklif al
        const result = await apiClient.requestQuote("sompo", request);
        setQuotes([result]);
        toast.success("Teklif alındı!");
      }
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : "Teklif alınamadı";
      setError(errorMsg);
      toast.error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSelectQuote = (quote: QuoteResponse) => {
    setSelectedQuote(quote);
    setPolicyDialogOpen(true);
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Trafik Sigortası</h1>
        <p className="text-muted-foreground">
          Trafik sigortası teklifi almak için formu doldurun
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Form */}
        <Card>
          <CardHeader>
            <CardTitle>Teklif Formu</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
              {/* Araç Bilgileri */}
              <div className="space-y-4">
                <h3 className="text-sm font-semibold">Araç Bilgileri</h3>

                <div>
                  <Label htmlFor="plate">Plaka</Label>
                  <Input
                    id="plate"
                    placeholder="34ABC123"
                    {...register("plate")}
                  />
                  {errors.plate && (
                    <p className="text-sm text-destructive mt-1">
                      {errors.plate.message}
                    </p>
                  )}
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="brand">Marka</Label>
                    <Input
                      id="brand"
                      placeholder="Toyota"
                      {...register("brand")}
                    />
                    {errors.brand && (
                      <p className="text-sm text-destructive mt-1">
                        {errors.brand.message}
                      </p>
                    )}
                  </div>

                  <div>
                    <Label htmlFor="model">Model</Label>
                    <Input
                      id="model"
                      placeholder="Corolla"
                      {...register("model")}
                    />
                    {errors.model && (
                      <p className="text-sm text-destructive mt-1">
                        {errors.model.message}
                      </p>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="year">Model Yılı</Label>
                    <Input
                      id="year"
                      type="number"
                      {...register("year", { valueAsNumber: true })}
                    />
                    {errors.year && (
                      <p className="text-sm text-destructive mt-1">
                        {errors.year.message}
                      </p>
                    )}
                  </div>

                  <div>
                    <Label htmlFor="usage">Kullanım</Label>
                    <select
                      id="usage"
                      {...register("usage")}
                      className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                    >
                      <option value="hususi">Hususi</option>
                      <option value="ticari">Ticari</option>
                    </select>
                  </div>
                </div>
              </div>

              {/* Sigortalı Bilgileri */}
              <div className="space-y-4">
                <h3 className="text-sm font-semibold">Sigortalı Bilgileri</h3>

                <div>
                  <Label htmlFor="tckn">TC Kimlik No</Label>
                  <Input
                    id="tckn"
                    placeholder="12345678901"
                    maxLength={11}
                    {...register("tckn")}
                  />
                  {errors.tckn && (
                    <p className="text-sm text-destructive mt-1">
                      {errors.tckn.message}
                    </p>
                  )}
                </div>

                <div>
                  <Label htmlFor="name">Ad Soyad</Label>
                  <Input
                    id="name"
                    placeholder="Ahmet Yılmaz"
                    {...register("name")}
                  />
                  {errors.name && (
                    <p className="text-sm text-destructive mt-1">
                      {errors.name.message}
                    </p>
                  )}
                </div>

                <div>
                  <Label htmlFor="birthDate">Doğum Tarihi</Label>
                  <Input
                    id="birthDate"
                    type="date"
                    {...register("birthDate")}
                  />
                  {errors.birthDate && (
                    <p className="text-sm text-destructive mt-1">
                      {errors.birthDate.message}
                    </p>
                  )}
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="phone">Telefon</Label>
                    <Input
                      id="phone"
                      placeholder="5551234567"
                      {...register("phone")}
                    />
                    {errors.phone && (
                      <p className="text-sm text-destructive mt-1">
                        {errors.phone.message}
                      </p>
                    )}
                  </div>

                  <div>
                    <Label htmlFor="email">E-posta</Label>
                    <Input
                      id="email"
                      type="email"
                      placeholder="ornek@email.com"
                      {...register("email")}
                    />
                    {errors.email && (
                      <p className="text-sm text-destructive mt-1">
                        {errors.email.message}
                      </p>
                    )}
                  </div>
                </div>
              </div>

              {/* Teminat Bilgileri */}
              <div className="space-y-4">
                <h3 className="text-sm font-semibold">Teminat Bilgileri</h3>

                <div>
                  <Label htmlFor="startDate">Başlangıç Tarihi</Label>
                  <Input
                    id="startDate"
                    type="date"
                    {...register("startDate")}
                  />
                  {errors.startDate && (
                    <p className="text-sm text-destructive mt-1">
                      {errors.startDate.message}
                    </p>
                  )}
                </div>
              </div>

              {/* Compare Mode Toggle */}
              <div className="flex items-center gap-2 rounded-lg border p-3">
                <input
                  type="checkbox"
                  id="compareMode"
                  checked={compareMode}
                  onChange={(e) => setCompareMode(e.target.checked)}
                  className="h-4 w-4"
                />
                <Label htmlFor="compareMode" className="cursor-pointer">
                  Tüm sigorta şirketlerinden karşılaştırmalı teklif al
                </Label>
              </div>

              <Button type="submit" className="w-full" disabled={isLoading}>
                {isLoading ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {compareMode
                      ? "Teklifler Alınıyor..."
                      : "Teklif Alınıyor..."}
                  </>
                ) : (
                  <>
                    <Search className="mr-2 h-4 w-4" />
                    {compareMode ? "Karşılaştırmalı Teklif Al" : "Teklif Al"}
                  </>
                )}
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>

      {/* Results - Full Width */}
      {error && (
        <Card className="border-destructive">
          <CardContent className="pt-6">
            <p className="text-sm text-destructive">❌ {error}</p>
          </CardContent>
        </Card>
      )}

      {quotes.length > 0 && (
        <QuoteComparison quotes={quotes} onSelectQuote={handleSelectQuote} />
      )}

      {!quotes.length && !error && !isLoading && (
        <Card>
          <CardContent className="pt-6 text-center text-muted-foreground">
            <p>Teklif almak için yukarıdaki formu doldurun</p>
            {compareMode && (
              <p className="text-xs mt-2">
                Tüm aktif sigorta şirketlerinden paralel olarak teklif alınacak
              </p>
            )}
          </CardContent>
        </Card>
      )}

      {/* Policy Dialog */}
      <PolicyDialog
        quote={selectedQuote}
        open={policyDialogOpen}
        onOpenChange={setPolicyDialogOpen}
      />
    </div>
  );
}
