"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CheckCircle2, PauseCircle, Clock } from "lucide-react";
import { ProviderInfo } from "@/lib/types";
import { cn } from "@/lib/utils";

interface ProviderStatusProps {
  provider: ProviderInfo;
}

export function ProviderStatus({ provider }: ProviderStatusProps) {
  const Icon = provider.active ? CheckCircle2 : PauseCircle;
  const statusColor = provider.active ? "text-green-600" : "text-zinc-400";

  // Tam isimleri göster
  const displayName =
    provider.name === "Sompo"
      ? "Sompo Sigorta"
      : provider.name === "Quick"
      ? "Quick Sigorta"
      : provider.name === "Axa"
      ? "Axa Sigorta"
      : provider.name === "Anadolu"
      ? "Anadolu Sigorta"
      : provider.name;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{displayName}</CardTitle>
        <Icon className={cn("h-5 w-5", statusColor)} />
      </CardHeader>
      <CardContent>
        <div className="flex flex-col gap-1">
          <div className="flex items-center gap-2">
            <span
              className={cn(
                "inline-flex items-center rounded-full px-2 py-1 text-xs font-medium",
                provider.active
                  ? "bg-green-100 text-green-700"
                  : "bg-zinc-100 text-zinc-600"
              )}
            >
              {provider.active ? "Aktif" : "Pasif"}
            </span>
          </div>

          {provider.reason && (
            <p className="text-xs text-muted-foreground mt-2">
              {provider.reason}
            </p>
          )}

          {provider.active && (
            <div className="flex items-center gap-1 mt-2 text-xs text-muted-foreground">
              <Clock className="h-3 w-3" />
              <span>2 dakika önce</span>
            </div>
          )}

          <div className="mt-2">
            <p className="text-xs text-muted-foreground">
              Ürünler: {provider.supported_products.join(", ")}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

interface ProviderGridProps {
  providers: ProviderInfo[];
}

export function ProviderGrid({ providers }: ProviderGridProps) {
  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {providers.map((provider) => (
        <ProviderStatus key={provider.name} provider={provider} />
      ))}
    </div>
  );
}
