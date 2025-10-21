"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Loader2, Car } from "lucide-react";
import { toast } from "sonner";

const kaskoFormSchema = z.object({
  tckn: z.string().min(11, "TC Kimlik No 11 karakter olmalıdır"),
  name: z.string().min(3, "Ad Soyad en az 3 karakter olmalıdır"),
  phone: z.string().min(10, "Telefon numarası geçersiz"),
  email: z.string().email("Geçerli bir e-posta adresi giriniz"),
  plate: z.string().min(6, "Plaka en az 6 karakter olmalıdır"),
  brand: z.string().min(2, "Marka giriniz"),
  model: z.string().min(2, "Model giriniz"),
  year: z
    .number()
    .min(1990)
    .max(new Date().getFullYear() + 1),
  kaskoType: z.enum(["tam", "kismi"]),
  vehicleValue: z.number().min(10000, "Araç değeri en az 10.000 TL olmalıdır"),
  usage: z.enum(["hususi", "ticari"]),
});

type KaskoFormData = z.infer<typeof kaskoFormSchema>;

export default function KaskoPage() {
  const [isLoading, setIsLoading] = useState(false);

  const {
    register,
    handleSubmit,
    setValue,
    formState: { errors },
  } = useForm<KaskoFormData>({
    resolver: zodResolver(kaskoFormSchema),
    defaultValues: {
      usage: "hususi",
      kaskoType: "tam",
      year: new Date().getFullYear(),
    },
  });

  const onSubmit = async (data: KaskoFormData) => {
    setIsLoading(true);
    try {
      toast.info("Kasko sigortası teklif sistemi hazırlanıyor...");
      // TODO: API entegrasyonu
      await new Promise((resolve) => setTimeout(resolve, 2000));
      toast.success("Teklif alındı!");
      console.log("Kasko Form Data:", data);
    } catch (e) {
      toast.error("Teklif alınamadı");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center gap-3">
        <Car className="h-8 w-8" />
        <div>
          <h1 className="text-3xl font-bold">Kasko Sigortası</h1>
          <p className="text-muted-foreground">
            Araç kasko sigortası teklifi alın
          </p>
        </div>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Araç Sahibi Bilgileri */}
        <Card>
          <CardHeader>
            <CardTitle>Araç Sahibi Bilgileri</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div>
              <Label htmlFor="tckn">TC Kimlik No</Label>
              <Input
                id="tckn"
                {...register("tckn")}
                placeholder="12345678901"
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
                {...register("name")}
                placeholder="Ahmet Yılmaz"
              />
              {errors.name && (
                <p className="text-sm text-destructive mt-1">
                  {errors.name.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="phone">Telefon</Label>
              <Input
                id="phone"
                {...register("phone")}
                placeholder="5551234567"
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
                {...register("email")}
                placeholder="ornek@email.com"
              />
              {errors.email && (
                <p className="text-sm text-destructive mt-1">
                  {errors.email.message}
                </p>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Araç Bilgileri */}
        <Card>
          <CardHeader>
            <CardTitle>Araç Bilgileri</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div>
              <Label htmlFor="plate">Plaka</Label>
              <Input id="plate" {...register("plate")} placeholder="34ABC123" />
              {errors.plate && (
                <p className="text-sm text-destructive mt-1">
                  {errors.plate.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="brand">Marka</Label>
              <Input id="brand" {...register("brand")} placeholder="Toyota" />
              {errors.brand && (
                <p className="text-sm text-destructive mt-1">
                  {errors.brand.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="model">Model</Label>
              <Input id="model" {...register("model")} placeholder="Corolla" />
              {errors.model && (
                <p className="text-sm text-destructive mt-1">
                  {errors.model.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="year">Model Yılı</Label>
              <Input
                id="year"
                type="number"
                {...register("year", { valueAsNumber: true })}
                placeholder="2023"
              />
              {errors.year && (
                <p className="text-sm text-destructive mt-1">
                  {errors.year.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="vehicleValue">Araç Değeri (TL)</Label>
              <Input
                id="vehicleValue"
                type="number"
                {...register("vehicleValue", { valueAsNumber: true })}
                placeholder="500000"
              />
              {errors.vehicleValue && (
                <p className="text-sm text-destructive mt-1">
                  {errors.vehicleValue.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="usage">Kullanım Şekli</Label>
              <Select
                onValueChange={(value) =>
                  setValue("usage", value as "hususi" | "ticari")
                }
                defaultValue="hususi"
              >
                <SelectTrigger>
                  <SelectValue placeholder="Seçiniz" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="hususi">Hususi</SelectItem>
                  <SelectItem value="ticari">Ticari</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        {/* Kasko Teminatları */}
        <Card>
          <CardHeader>
            <CardTitle>Kasko Teminatları</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-4">
            <div>
              <Label htmlFor="kaskoType">Kasko Tipi</Label>
              <Select
                onValueChange={(value) =>
                  setValue("kaskoType", value as "tam" | "kismi")
                }
                defaultValue="tam"
              >
                <SelectTrigger>
                  <SelectValue placeholder="Seçiniz" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="tam">Tam Kasko</SelectItem>
                  <SelectItem value="kismi">Kısmi Kasko</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        {/* Submit Button */}
        <div className="flex justify-end gap-3">
          <Button type="submit" size="lg" disabled={isLoading}>
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Teklif Alınıyor...
              </>
            ) : (
              "Teklif Al"
            )}
          </Button>
        </div>
      </form>

      {/* Info */}
      <Card className="border-blue-200 bg-blue-50 dark:bg-blue-950 dark:border-blue-800">
        <CardContent className="pt-6">
          <p className="text-sm text-blue-800 dark:text-blue-200">
            ℹ️ Kasko sigortası teklif sistemi yakında aktif olacaktır. Şu anda
            test aşamasındadır.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
