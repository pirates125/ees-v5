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
import { Loader2, Heart } from "lucide-react";
import { toast } from "sonner";

const saglikFormSchema = z.object({
  tckn: z.string().min(11, "TC Kimlik No 11 karakter olmalıdır"),
  name: z.string().min(3, "Ad Soyad en az 3 karakter olmalıdır"),
  birthDate: z.string().min(10, "Doğum tarihi giriniz"),
  gender: z.enum(["erkek", "kadin"]),
  phone: z.string().min(10, "Telefon numarası geçersiz"),
  email: z.string().email("Geçerli bir e-posta adresi giriniz"),
  height: z.number().min(100).max(250, "Boy geçerli aralıkta olmalıdır"),
  weight: z.number().min(30).max(200, "Kilo geçerli aralıkta olmalıdır"),
  smokingStatus: z.enum(["sigara-icmiyor", "sigara-iciyor", "birakti"]),
  occupation: z.string().min(2, "Meslek giriniz"),
});

type SaglikFormData = z.infer<typeof saglikFormSchema>;

export default function SaglikPage() {
  const [isLoading, setIsLoading] = useState(false);
  const [chronicDiseases, setChronicDiseases] = useState({
    diabetes: false,
    hypertension: false,
    heartDisease: false,
    asthma: false,
  });
  const [coverageLevel, setCoverageLevel] = useState<string>("standart");

  const {
    register,
    handleSubmit,
    setValue,
    formState: { errors },
  } = useForm<SaglikFormData>({
    resolver: zodResolver(saglikFormSchema),
    defaultValues: {
      gender: "erkek",
      smokingStatus: "sigara-icmiyor",
      height: 170,
      weight: 70,
    },
  });

  const onSubmit = async (data: SaglikFormData) => {
    setIsLoading(true);
    try {
      toast.info("Sağlık sigortası teklif sistemi hazırlanıyor...");
      // TODO: API entegrasyonu
      await new Promise((resolve) => setTimeout(resolve, 2000));
      toast.success("Teklif alındı!");
      console.log(
        "Sağlık Form Data:",
        data,
        "Chronic Diseases:",
        chronicDiseases,
        "Coverage:",
        coverageLevel
      );
    } catch (e) {
      toast.error("Teklif alınamadı");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center gap-3">
        <Heart className="h-8 w-8" />
        <div>
          <h1 className="text-3xl font-bold">Sağlık Sigortası</h1>
          <p className="text-muted-foreground">Sağlık sigortası teklifi alın</p>
        </div>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Kişisel Bilgiler */}
        <Card>
          <CardHeader>
            <CardTitle>Kişisel Bilgiler</CardTitle>
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
              <Label htmlFor="birthDate">Doğum Tarihi</Label>
              <Input
                id="birthDate"
                type="date"
                {...register("birthDate")}
                placeholder="1990-01-01"
              />
              {errors.birthDate && (
                <p className="text-sm text-destructive mt-1">
                  {errors.birthDate.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="gender">Cinsiyet</Label>
              <Select
                onValueChange={(value) =>
                  setValue("gender", value as "erkek" | "kadin")
                }
                defaultValue="erkek"
              >
                <SelectTrigger>
                  <SelectValue placeholder="Seçiniz" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="erkek">Erkek</SelectItem>
                  <SelectItem value="kadin">Kadın</SelectItem>
                </SelectContent>
              </Select>
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

            <div>
              <Label htmlFor="occupation">Meslek</Label>
              <Input
                id="occupation"
                {...register("occupation")}
                placeholder="Yazılım Geliştirici"
              />
              {errors.occupation && (
                <p className="text-sm text-destructive mt-1">
                  {errors.occupation.message}
                </p>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Sağlık Bilgileri */}
        <Card>
          <CardHeader>
            <CardTitle>Sağlık Bilgileri</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div>
              <Label htmlFor="height">Boy (cm)</Label>
              <Input
                id="height"
                type="number"
                {...register("height", { valueAsNumber: true })}
                placeholder="170"
              />
              {errors.height && (
                <p className="text-sm text-destructive mt-1">
                  {errors.height.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="weight">Kilo (kg)</Label>
              <Input
                id="weight"
                type="number"
                {...register("weight", { valueAsNumber: true })}
                placeholder="70"
              />
              {errors.weight && (
                <p className="text-sm text-destructive mt-1">
                  {errors.weight.message}
                </p>
              )}
            </div>

            <div>
              <Label htmlFor="smokingStatus">Sigara Kullanımı</Label>
              <Select
                onValueChange={(value) =>
                  setValue(
                    "smokingStatus",
                    value as "sigara-icmiyor" | "sigara-iciyor" | "birakti"
                  )
                }
                defaultValue="sigara-icmiyor"
              >
                <SelectTrigger>
                  <SelectValue placeholder="Seçiniz" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="sigara-icmiyor">İçmiyor</SelectItem>
                  <SelectItem value="sigara-iciyor">İçiyor</SelectItem>
                  <SelectItem value="birakti">Bıraktı</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        {/* Kronik Hastalıklar */}
        <Card>
          <CardHeader>
            <CardTitle>Kronik Hastalıklar</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="diabetes"
                checked={chronicDiseases.diabetes}
                onCheckedChange={(checked) =>
                  setChronicDiseases({
                    ...chronicDiseases,
                    diabetes: checked as boolean,
                  })
                }
              />
              <Label
                htmlFor="diabetes"
                className="text-sm font-normal cursor-pointer"
              >
                Şeker Hastalığı (Diyabet)
              </Label>
            </div>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="hypertension"
                checked={chronicDiseases.hypertension}
                onCheckedChange={(checked) =>
                  setChronicDiseases({
                    ...chronicDiseases,
                    hypertension: checked as boolean,
                  })
                }
              />
              <Label
                htmlFor="hypertension"
                className="text-sm font-normal cursor-pointer"
              >
                Yüksek Tansiyon (Hipertansiyon)
              </Label>
            </div>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="heartDisease"
                checked={chronicDiseases.heartDisease}
                onCheckedChange={(checked) =>
                  setChronicDiseases({
                    ...chronicDiseases,
                    heartDisease: checked as boolean,
                  })
                }
              />
              <Label
                htmlFor="heartDisease"
                className="text-sm font-normal cursor-pointer"
              >
                Kalp Hastalığı
              </Label>
            </div>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="asthma"
                checked={chronicDiseases.asthma}
                onCheckedChange={(checked) =>
                  setChronicDiseases({
                    ...chronicDiseases,
                    asthma: checked as boolean,
                  })
                }
              />
              <Label
                htmlFor="asthma"
                className="text-sm font-normal cursor-pointer"
              >
                Astım
              </Label>
            </div>
          </CardContent>
        </Card>

        {/* Teminat Seviyesi */}
        <Card>
          <CardHeader>
            <CardTitle>Teminat Seviyesi</CardTitle>
          </CardHeader>
          <CardContent>
            <Select onValueChange={setCoverageLevel} defaultValue="standart">
              <SelectTrigger>
                <SelectValue placeholder="Teminat seviyesi seçiniz" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="temel">Temel (100.000 TL)</SelectItem>
                <SelectItem value="standart">Standart (250.000 TL)</SelectItem>
                <SelectItem value="premium">Premium (500.000 TL)</SelectItem>
                <SelectItem value="platinum">
                  Platinum (1.000.000 TL)
                </SelectItem>
              </SelectContent>
            </Select>
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
      <Card className="border-red-200 bg-red-50 dark:bg-red-950 dark:border-red-800">
        <CardContent className="pt-6">
          <p className="text-sm text-red-800 dark:text-red-200">
            ℹ️ Sağlık sigortası teklif sistemi yakında aktif olacaktır. Şu anda
            test aşamasındadır.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
