export interface ProviderInfo {
  name: string;
  active: boolean;
  reason?: string;
  supported_products: string[];
}

export interface ProvidersResponse {
  providers: ProviderInfo[];
  total: number;
  active_count: number;
}

export interface QuoteRequest {
  insured: {
    tckn: string;
    name: string;
    birthDate: string;
    phone: string;
    email: string;
  };
  vehicle: {
    plate: string;
    vin?: string;
    brand: string;
    model: string;
    year: number;
    usage: "hususi" | "ticari";
  };
  coverage: {
    productType: "trafik" | "kasko" | "konut" | "saglik";
    startDate: string;
    addons: string[];
  };
  quoteMeta?: {
    requestId?: string;
    webhookUrl?: string;
  };
}

export interface QuoteResponse {
  requestId: string;
  company: string;
  productType: string;
  premium: {
    net: number;
    gross: number;
    taxes: number;
    currency: string;
  };
  installments: Array<{
    count: number;
    perInstallment: number;
    total: number;
  }>;
  coverages: Array<{
    code: string;
    name: string;
    limit?: string;
    included: boolean;
  }>;
  warnings: string[];
  raw?: {
    htmlSnapshotPath?: string;
    fieldsEcho?: Record<string, unknown>;
  };
  timings?: {
    queuedMs: number;
    scrapeMs: number;
  };
}

export interface ErrorResponse {
  requestId?: string;
  error: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
}

export interface DashboardStats {
  totalQuotes: number;
  totalPolicies: number;
  totalRevenue: number;
  activeProviders: number;
}
