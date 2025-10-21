"use client";

import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { QuoteResponse } from "@/lib/types";
import { formatCurrency } from "@/lib/utils";
import { Loader2 } from "lucide-react";
import { toast } from "sonner";

interface PolicyDialogProps {
  quote: QuoteResponse | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function PolicyDialog({ quote, open, onOpenChange }: PolicyDialogProps) {
  const [isCreating, setIsCreating] = useState(false);
  const [paymentMethod, setPaymentMethod] = useState("credit_card");
  const [installmentCount, setInstallmentCount] = useState("1");

  if (!quote) return null;

  const handleCreatePolicy = async () => {
    setIsCreating(true);

    try {
      const token = localStorage.getItem("token");

      if (!token) {
        throw new Error("Oturum bulunamadı. Lütfen giriş yapın.");
      }

      const response = await fetch(
        `${process.env.NEXT_PUBLIC_API_URL}/api/v1/policies`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify({
            quoteId: quote.requestId,
            paymentMethod,
            installmentCount: parseInt(installmentCount),
          }),
        }
      );

      if (!response.ok) {
        throw new Error("Poliçe oluşturulamadı");
      }

      const policy = await response.json();

      toast.success("Poliçe başarıyla oluşturuldu!");
      toast.info(`Poliçe No: ${policy.policy_number}`);

      onOpenChange(false);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Poliçe oluşturulamadı");
    } finally {
      setIsCreating(false);
    }
  };

  const selectedInstallment = quote.installments.find(
    (i) => i.count === parseInt(installmentCount)
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Poliçe Kes</DialogTitle>
          <DialogDescription>
            {quote.company} - {formatCurrency(quote.premium.gross)}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Ödeme Yöntemi */}
          <div className="space-y-2">
            <Label>Ödeme Yöntemi</Label>
            <Select value={paymentMethod} onValueChange={setPaymentMethod}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="credit_card">Kredi Kartı</SelectItem>
                <SelectItem value="bank_transfer">Havale/EFT</SelectItem>
                <SelectItem value="cash">Nakit</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Taksit Sayısı */}
          <div className="space-y-2">
            <Label>Taksit Sayısı</Label>
            <Select
              value={installmentCount}
              onValueChange={setInstallmentCount}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {quote.installments.map((inst) => (
                  <SelectItem key={inst.count} value={inst.count.toString()}>
                    {inst.count} Taksit - {formatCurrency(inst.perInstallment)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Özet */}
          <div className="rounded-lg border p-4 space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Toplam Prim:</span>
              <span className="font-medium">
                {formatCurrency(quote.premium.gross)}
              </span>
            </div>
            {selectedInstallment && selectedInstallment.count > 1 && (
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Taksit Başına:</span>
                <span className="font-medium">
                  {formatCurrency(selectedInstallment.perInstallment)}
                </span>
              </div>
            )}
            <div className="flex justify-between text-sm pt-2 border-t">
              <span className="text-muted-foreground">Komisyon (10%):</span>
              <span className="font-medium text-green-600">
                {formatCurrency(quote.premium.gross * 0.1)}
              </span>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            İptal
          </Button>
          <Button onClick={handleCreatePolicy} disabled={isCreating}>
            {isCreating ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Oluşturuluyor...
              </>
            ) : (
              "Poliçe Kes"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
