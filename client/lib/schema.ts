import { z } from "zod";

export const trafikFormSchema = z.object({
  // Araç bilgileri
  plate: z.string().min(7, "Geçerli bir plaka giriniz").max(9),
  brand: z.string().min(2, "Marka seçiniz"),
  model: z.string().min(2, "Model seçiniz"),
  year: z
    .number()
    .min(1990)
    .max(new Date().getFullYear() + 1),
  usage: z.enum(["hususi", "ticari"]),

  // Sigortalı bilgileri
  tckn: z.string().length(11, "TC Kimlik No 11 haneli olmalıdır"),
  name: z.string().min(3, "Ad Soyad giriniz"),
  birthDate: z.string().min(10, "Doğum tarihi giriniz"),
  phone: z.string().min(10, "Geçerli telefon numarası giriniz"),
  email: z.string().email("Geçerli bir e-posta adresi giriniz"),

  // Teminat bilgileri
  startDate: z.string().min(10, "Başlangıç tarihi giriniz"),
  addons: z.array(z.string()).optional().default([]),
});

export type TrafikFormData = z.infer<typeof trafikFormSchema>;

export const kaskoFormSchema = trafikFormSchema.extend({
  vin: z.string().optional(),
  vehicleValue: z.number().min(0).optional(),
});

export type KaskoFormData = z.infer<typeof kaskoFormSchema>;
