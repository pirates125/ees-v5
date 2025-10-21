"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { QuoteResponse } from "@/lib/types";
import { formatCurrency, cn } from "@/lib/utils";
import { CheckCircle2, Clock, TrendingDown, Zap, Award } from "lucide-react";

interface QuoteComparisonProps {
  quotes: QuoteResponse[];
  onSelectQuote?: (quote: QuoteResponse) => void;
}

export function QuoteComparison({
  quotes,
  onSelectQuote,
}: QuoteComparisonProps) {
  if (quotes.length === 0) {
    return null;
  }

  // En ucuz ve en hızlı teklifleri bul
  const cheapest = quotes.reduce((min, q) =>
    q.premium.gross < min.premium.gross ? q : min
  );
  const fastest = quotes.reduce((min, q) =>
    (q.timings?.scrapeMs || 0) < (min.timings?.scrapeMs || 0) ? q : min
  );

  return (
    <div className="space-y-4">
      {/* Summary */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Teklif Karşılaştırma</h2>
          <p className="text-sm text-muted-foreground">
            {quotes.length} sigorta şirketinden teklif alındı
          </p>
        </div>
        <div className="flex gap-2">
          <Badge variant="success">
            <TrendingDown className="mr-1 h-3 w-3" />
            En ucuz: {formatCurrency(cheapest.premium.gross)}
          </Badge>
          <Badge variant="outline">
            <Zap className="mr-1 h-3 w-3" />
            En hızlı: {((fastest.timings?.scrapeMs || 0) / 1000).toFixed(1)}s
          </Badge>
        </div>
      </div>

      {/* Quote Cards Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {quotes
          .sort((a, b) => Number(a.premium.gross) - Number(b.premium.gross))
          .map((quote) => {
            const isCheapest = quote.requestId === cheapest.requestId;
            const isFastest = quote.requestId === fastest.requestId;
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
              <Card
                key={quote.requestId}
                className={cn(
                  "transition-all hover:shadow-lg",
                  isCheapest && "ring-2 ring-green-500"
                )}
              >
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <CardTitle className="text-lg">{displayCompany}</CardTitle>
                    <div className="flex flex-col gap-1">
                      {isCheapest && (
                        <Badge variant="success" className="text-xs">
                          <Award className="mr-1 h-3 w-3" />
                          En Ucuz
                        </Badge>
                      )}
                      {isFastest && (
                        <Badge variant="outline" className="text-xs">
                          <Zap className="mr-1 h-3 w-3" />
                          En Hızlı
                        </Badge>
                      )}
                    </div>
                  </div>
                  <div className="text-3xl font-bold text-accent">
                    {formatCurrency(quote.premium.gross)}
                  </div>
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
                  </div>

                  {/* Taksit Seçenekleri */}
                  {quote.installments.length > 1 && (
                    <div className="space-y-2">
                      <h4 className="text-sm font-medium">
                        Taksit Seçenekleri
                      </h4>
                      <div className="space-y-1">
                        {quote.installments.slice(0, 3).map((inst, idx) => (
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
                  <div className="space-y-2">
                    <h4 className="text-sm font-medium">Teminatlar</h4>
                    <div className="space-y-1">
                      {quote.coverages.slice(0, 3).map((cov, idx) => (
                        <div
                          key={idx}
                          className="flex items-center gap-2 text-xs"
                        >
                          {cov.included && (
                            <CheckCircle2 className="h-3 w-3 text-green-600" />
                          )}
                          <span className="text-muted-foreground">
                            {cov.name}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* Timing */}
                  {quote.timings && (
                    <div className="flex items-center gap-2 text-xs text-muted-foreground pt-2 border-t">
                      <Clock className="h-3 w-3" />
                      <span>{(quote.timings.scrapeMs / 1000).toFixed(1)}s</span>
                    </div>
                  )}

                  {/* Action Button */}
                  {onSelectQuote && (
                    <Button
                      className="w-full"
                      variant={isCheapest ? "default" : "outline"}
                      onClick={() => onSelectQuote(quote)}
                    >
                      {isCheapest ? "Bu Teklifi Seç" : "Seç"}
                    </Button>
                  )}
                </CardContent>
              </Card>
            );
          })}
      </div>
    </div>
  );
}
