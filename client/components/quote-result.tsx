"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { QuoteResponse } from "@/lib/types";
import { formatCurrency } from "@/lib/utils";
import { CheckCircle2, Clock } from "lucide-react";

interface QuoteResultProps {
  quote: QuoteResponse;
}

export function QuoteResult({ quote }: QuoteResultProps) {
  // Tam şirket adını göster
  const displayCompany =
    quote.company === "Sompo"
      ? "Sompo Sigorta"
      : quote.company === "Quick"
      ? "Quick Sigorta"
      : quote.company === "Axa"
      ? "Axa Sigorta"
      : quote.company === "Anadolu"
      ? "Anadolu Sigorta"
      : quote.company;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <span>{displayCompany}</span>
          <span className="text-lg font-bold text-accent">
            {formatCurrency(quote.premium.gross)}
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Premium Detayı */}
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Net Prim:</span>
            <span>{formatCurrency(quote.premium.net)}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Vergiler:</span>
            <span>{formatCurrency(quote.premium.taxes)}</span>
          </div>
          <div className="flex justify-between border-t pt-2 font-medium">
            <span>Toplam:</span>
            <span>{formatCurrency(quote.premium.gross)}</span>
          </div>
        </div>

        {/* Taksit Seçenekleri */}
        {quote.installments.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium">Taksit Seçenekleri</h4>
            <div className="space-y-1">
              {quote.installments.map((inst, idx) => (
                <div
                  key={idx}
                  className="flex justify-between text-sm text-muted-foreground"
                >
                  <span>{inst.count} Taksit</span>
                  <span>{formatCurrency(inst.perInstallment)}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Teminatlar */}
        {quote.coverages.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium">Teminatlar</h4>
            <div className="space-y-1">
              {quote.coverages.map((cov, idx) => (
                <div key={idx} className="flex items-center gap-2 text-sm">
                  {cov.included && (
                    <CheckCircle2 className="h-4 w-4 text-green-600" />
                  )}
                  <span className="text-muted-foreground">{cov.name}</span>
                  {cov.limit && (
                    <span className="text-xs text-muted-foreground">
                      ({cov.limit})
                    </span>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Timing */}
        {quote.timings && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground pt-2 border-t">
            <Clock className="h-3 w-3" />
            <span>
              İşlem süresi: {(quote.timings.scrapeMs / 1000).toFixed(1)}s
            </span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
