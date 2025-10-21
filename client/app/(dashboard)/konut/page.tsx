"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Loader2, Home } from "lucide-react";
import { toast } from "sonner";

const konutFormSchema = z.object({
  tckn: z.string().min(11, "TC Kimlik No 11 karakter olmalıdır"),
  name: z.string().min(3, "Ad Soyad en az 3 karakter olmalıdır"),
  phone: z.string().min(10, "Telefon numarası geçersiz"),
  email: z.string().email("Geçerli bir e-posta adresi giriniz"),
  address: z.string().min(10, "Adres en az 10 karakter olmalıdır"),
  city: z.string().min(2, "Şehir giriniz"),
  district: z.string().min(2, "İlçe giriniz"),
  buildingType: z.enum(["apartman", "mustakil", "villa"]),
  squareMeters: z.number().min(20, "Minimum 20 m² olmalıdır"),
  buildingYear: z.number().min(1900).max(new Date().getFullYear()),
  numberOfFloors: z.number().min(1).max(50),
  floor: z.number().min(0).max(50),
});

type KonutFormData = z.infer<typeof konutFormSchema>;

export default function KonutPage() {
  const [isLoading, setIsLoading] = useState(false);
  const [coverages, setCoverages] = useState({
    fire: true,
    earthquake: false,
    flood: false,
    theft: false,
  });

  const {
    register,
    handleSubmit,
    setValue,
    formState: { errors },
  } = useForm<KonutFormData>({
    resolver: zodResolver(konutFormSchema),
    defaultValues: {
      buildingType: "apartman",
      buildingYear: 2010,
      numberOfFloors: 5,
      floor: 2,
      squareMeters: 100,
    },
  });

  const onSubmit = async (data: KonutFormData) => {
    setIsLoading(true);
    try {
      toast.info("Konut sigortası teklif sistemi hazırlanıyor...");
      // TODO: API entegrasyonu
      await new Promise((resolve) => setTimeout(resolve, 2000));
      toast.success("Teklif alındı!");
      console.log("Konut Form Data:", data, "Coverages:", coverages);
    } catch (e) {
      toast.error("Teklif alınamadı");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center gap-3">
        <Home className="h-8 w-8" />
        <div>
          <h1 className="text-3xl font-bold">Konut Sigortası</h1>
          <p className="text-muted-foreground">
            Eviniz için sigorta teklifi alın
          </p>
        </div>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Sigortalı Bilgileri */}
        <Card>
          <CardHeader>
            <CardTitle>Sigortalı Bilgileri</CardTitle>
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

        {/* Konut Bilgileri */}
        <Card>
          <CardHeader>
            <CardTitle>Konut Bilgileri</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div className="md:col-span-2">
              <Label htmlFor="address">Adres</Label>
              <Input
                id="address"
                {...register("address")}
                placeholder="Sokak, Mahalle, No"
              />
              {errors.address && (
                <p className="text-sm text-destructive mt-1">
                  {errors.address.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="city">Şehir</Label>
              <Input id="city" {...register("city")} placeholder="İstanbul" />
              {errors.city && (
                <p className="text-sm text-destructive mt-1">
                  {errors.city.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="district">İlçe</Label>
              <Input
                id="district"
                {...register("district")}
                placeholder="Kadıköy"
              />
              {errors.district && (
                <p className="text-sm text-destructive mt-1">
                  {errors.district.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="buildingType">Bina Tipi</Label>
              <Select
                onValueChange={(value) =>
                  setValue(
                    "buildingType",
                    value as "apartman" | "mustakil" | "villa"
                  )
                }
                defaultValue="apartman"
              >
                <SelectTrigger>
                  <SelectValue placeholder="Seçiniz" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="apartman">Apartman</SelectItem>
                  <SelectItem value="mustakil">Müstakil</SelectItem>
                  <SelectItem value="villa">Villa</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label htmlFor="squareMeters">M² (Metrekare)</Label>
              <Input
                id="squareMeters"
                type="number"
                {...register("squareMeters", { valueAsNumber: true })}
                placeholder="100"
              />
              {errors.squareMeters && (
                <p className="text-sm text-destructive mt-1">
                  {errors.squareMeters.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="buildingYear">Bina Yapım Yılı</Label>
              <Input
                id="buildingYear"
                type="number"
                {...register("buildingYear", { valueAsNumber: true })}
                placeholder="2010"
              />
              {errors.buildingYear && (
                <p className="text-sm text-destructive mt-1">
                  {errors.buildingYear.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="numberOfFloors">Toplam Kat Sayısı</Label>
              <Input
                id="numberOfFloors"
                type="number"
                {...register("numberOfFloors", { valueAsNumber: true })}
                placeholder="5"
              />
              {errors.numberOfFloors && (
                <p className="text-sm text-destructive mt-1">
                  {errors.numberOfFloors.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="floor">Bulunduğu Kat</Label>
              <Input
                id="floor"
                type="number"
                {...register("floor", { valueAsNumber: true })}
                placeholder="2"
              />
              {errors.floor && (
                <p className="text-sm text-destructive mt-1">
                  {errors.floor.message}
                </p>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Teminat Seçenekleri */}
        <Card>
          <CardHeader>
            <CardTitle>Teminat Seçenekleri</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="fire"
                checked={coverages.fire}
                onCheckedChange={(checked) =>
                  setCoverages({ ...coverages, fire: checked as boolean })
                }
              />
              <Label
                htmlFor="fire"
                className="text-sm font-normal cursor-pointer"
              >
                Yangın (Zorunlu)
              </Label>
            </div>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="earthquake"
                checked={coverages.earthquake}
                onCheckedChange={(checked) =>
                  setCoverages({ ...coverages, earthquake: checked as boolean })
                }
              />
              <Label
                htmlFor="earthquake"
                className="text-sm font-normal cursor-pointer"
              >
                Deprem
              </Label>
            </div>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="flood"
                checked={coverages.flood}
                onCheckedChange={(checked) =>
                  setCoverages({ ...coverages, flood: checked as boolean })
                }
              />
              <Label
                htmlFor="flood"
                className="text-sm font-normal cursor-pointer"
              >
                Sel/Su Baskını
              </Label>
            </div>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="theft"
                checked={coverages.theft}
                onCheckedChange={(checked) =>
                  setCoverages({ ...coverages, theft: checked as boolean })
                }
              />
              <Label
                htmlFor="theft"
                className="text-sm font-normal cursor-pointer"
              >
                Hırsızlık
              </Label>
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
      <Card className="border-green-200 bg-green-50 dark:bg-green-950 dark:border-green-800">
        <CardContent className="pt-6">
          <p className="text-sm text-green-800 dark:text-green-200">
            ℹ️ Konut sigortası teklif sistemi yakında aktif olacaktır. Şu anda
            test aşamasındadır.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
