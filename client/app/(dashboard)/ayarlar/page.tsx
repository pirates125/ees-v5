"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { ThemeToggle } from "@/components/theme-toggle";
import { useAuth } from "@/lib/auth-context";
import { Loader2, Settings, User, Lock, Bell } from "lucide-react";
import { toast } from "sonner";
import { apiClient } from "@/lib/api-client";

const profileSchema = z.object({
  name: z.string().min(3, "Ad Soyad en az 3 karakter olmalıdır"),
  email: z.string().email("Geçerli bir e-posta adresi giriniz"),
  phone: z
    .string()
    .min(10, "Telefon numarası geçersiz")
    .optional()
    .or(z.literal("")),
});

const passwordSchema = z
  .object({
    currentPassword: z
      .string()
      .min(6, "Mevcut şifre en az 6 karakter olmalıdır"),
    newPassword: z.string().min(8, "Yeni şifre en az 8 karakter olmalıdır"),
    confirmPassword: z.string(),
  })
  .refine((data) => data.newPassword === data.confirmPassword, {
    message: "Şifreler eşleşmiyor",
    path: ["confirmPassword"],
  });

type ProfileFormData = z.infer<typeof profileSchema>;
type PasswordFormData = z.infer<typeof passwordSchema>;

export default function AyarlarPage() {
  const { user, updateUser } = useAuth();
  const [isLoadingProfile, setIsLoadingProfile] = useState(false);
  const [isLoadingPassword, setIsLoadingPassword] = useState(false);

  const {
    register: registerProfile,
    handleSubmit: handleSubmitProfile,
    formState: { errors: errorsProfile },
  } = useForm<ProfileFormData>({
    resolver: zodResolver(profileSchema),
    defaultValues: {
      name: user?.name || "",
      email: user?.email || "",
    },
  });

  const {
    register: registerPassword,
    handleSubmit: handleSubmitPassword,
    formState: { errors: errorsPassword },
    reset: resetPassword,
  } = useForm<PasswordFormData>({
    resolver: zodResolver(passwordSchema),
  });

  const onSubmitProfile = async (data: ProfileFormData) => {
    setIsLoadingProfile(true);
    try {
      const updatedUser = await apiClient.updateProfile({
        name: data.name,
        phone: data.phone || undefined,
      });

      // Update auth context
      updateUser(updatedUser as any);

      toast.success("Profil bilgileri güncellendi");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Profil güncellenemedi");
    } finally {
      setIsLoadingProfile(false);
    }
  };

  const onSubmitPassword = async (data: PasswordFormData) => {
    setIsLoadingPassword(true);
    try {
      await apiClient.changePassword(data.currentPassword, data.newPassword);
      toast.success("Şifre başarıyla değiştirildi");
      resetPassword();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Şifre değiştirilemedi");
    } finally {
      setIsLoadingPassword(false);
    }
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center gap-3">
        <Settings className="h-8 w-8" />
        <div>
          <h1 className="text-3xl font-bold">Ayarlar</h1>
          <p className="text-muted-foreground">Hesap ayarlarınızı yönetin</p>
        </div>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        {/* Profil Bilgileri */}
        <Card className="md:col-span-2">
          <CardHeader>
            <div className="flex items-center gap-2">
              <User className="h-5 w-5" />
              <CardTitle>Profil Bilgileri</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <form
              onSubmit={handleSubmitProfile(onSubmitProfile)}
              className="space-y-4"
            >
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <Label htmlFor="name">Ad Soyad</Label>
                  <Input
                    id="name"
                    {...registerProfile("name")}
                    placeholder="Ahmet Yılmaz"
                  />
                  {errorsProfile.name && (
                    <p className="text-sm text-destructive mt-1">
                      {errorsProfile.name.message}
                    </p>
                  )}
                </div>

                <div>
                  <Label htmlFor="email">E-posta</Label>
                  <Input
                    id="email"
                    type="email"
                    {...registerProfile("email")}
                    placeholder="ornek@email.com"
                  />
                  {errorsProfile.email && (
                    <p className="text-sm text-destructive mt-1">
                      {errorsProfile.email.message}
                    </p>
                  )}
                </div>

                <div>
                  <Label htmlFor="phone">Telefon (Opsiyonel)</Label>
                  <Input
                    id="phone"
                    {...registerProfile("phone")}
                    placeholder="5551234567"
                  />
                  {errorsProfile.phone && (
                    <p className="text-sm text-destructive mt-1">
                      {errorsProfile.phone.message}
                    </p>
                  )}
                </div>

                <div className="flex items-end">
                  <div className="w-full">
                    <Label>Rol</Label>
                    <Input
                      value={user?.role === "admin" ? "Admin" : "Kullanıcı"}
                      disabled
                    />
                  </div>
                </div>
              </div>

              <div className="flex justify-end">
                <Button type="submit" disabled={isLoadingProfile}>
                  {isLoadingProfile ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Kaydediliyor...
                    </>
                  ) : (
                    "Kaydet"
                  )}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        {/* Şifre Değiştirme */}
        <Card className="md:col-span-2">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Lock className="h-5 w-5" />
              <CardTitle>Şifre Değiştir</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <form
              onSubmit={handleSubmitPassword(onSubmitPassword)}
              className="space-y-4"
            >
              <div className="grid gap-4 md:grid-cols-3">
                <div>
                  <Label htmlFor="currentPassword">Mevcut Şifre</Label>
                  <Input
                    id="currentPassword"
                    type="password"
                    {...registerPassword("currentPassword")}
                    placeholder="••••••"
                  />
                  {errorsPassword.currentPassword && (
                    <p className="text-sm text-destructive mt-1">
                      {errorsPassword.currentPassword.message}
                    </p>
                  )}
                </div>

                <div>
                  <Label htmlFor="newPassword">Yeni Şifre</Label>
                  <Input
                    id="newPassword"
                    type="password"
                    {...registerPassword("newPassword")}
                    placeholder="••••••••"
                  />
                  {errorsPassword.newPassword && (
                    <p className="text-sm text-destructive mt-1">
                      {errorsPassword.newPassword.message}
                    </p>
                  )}
                </div>

                <div>
                  <Label htmlFor="confirmPassword">Şifre Tekrar</Label>
                  <Input
                    id="confirmPassword"
                    type="password"
                    {...registerPassword("confirmPassword")}
                    placeholder="••••••••"
                  />
                  {errorsPassword.confirmPassword && (
                    <p className="text-sm text-destructive mt-1">
                      {errorsPassword.confirmPassword.message}
                    </p>
                  )}
                </div>
              </div>

              <div className="flex justify-end">
                <Button type="submit" disabled={isLoadingPassword}>
                  {isLoadingPassword ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Değiştiriliyor...
                    </>
                  ) : (
                    "Şifreyi Değiştir"
                  )}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        {/* Tema Ayarları */}
        <Card>
          <CardHeader>
            <CardTitle>Tema Ayarları</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <Label>Görünüm Modu</Label>
                <p className="text-sm text-muted-foreground">
                  Karanlık veya aydınlık temayı seçin
                </p>
              </div>
              <ThemeToggle />
            </div>
          </CardContent>
        </Card>

        {/* Bildirim Ayarları */}
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Bell className="h-5 w-5" />
              <CardTitle>Bildirim Ayarları</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="text-sm text-muted-foreground">
              Bildirim ayarları yakında eklenecektir.
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Hesap Bilgileri */}
      <Card>
        <CardHeader>
          <CardTitle>Hesap Bilgileri</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Kullanıcı ID:</span>
            <span className="font-mono">{user?.id}</span>
          </div>
          <Separator />
          <div className="flex justify-between">
            <span className="text-muted-foreground">E-posta:</span>
            <span>{user?.email}</span>
          </div>
          <Separator />
          <div className="flex justify-between">
            <span className="text-muted-foreground">Rol:</span>
            <span>{user?.role === "admin" ? "Admin" : "Kullanıcı"}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
