### Gereksinimler

- **Node.js** 20+
- **Rust** 1.75+
- **ChromeDriver** (browser automation için)

### Lokal Geliştirme

#### 1. ChromeDriver'ı Başlat

```bash
# Terminal 1
chromedriver --port=9515
```

#### 2. Rust Server'ı Başlat

```bash
# Terminal 2
cd server

# .env dosyasını yapılandır
cp env.example .env
# SOMPO_USERNAME, SOMPO_PASSWORD, SOMPO_SECRET_KEY'i düzenle

# Bağımlılıkları yükle ve çalıştır
cargo run

# Server http://localhost:8099'da başlar
```

#### 3. Next.js Client'ı Başlat

```bash
# Terminal 3
cd client

# Bağımlılıkları yükle
npm install

# Geliştirme modunda çalıştır
npm run dev

# Client http://localhost:3000'de başlar
```

### Docker ile Çalıştırma

```bash
# Tüm servisleri başlat
docker-compose up --build

# Frontend: http://localhost:3000
# Backend: http://localhost:8099
```

## ⚙️ Yapılandırma

### Server (.env)

```env
# Server Configuration
HTTP_ADDR=0.0.0.0:8099
RUST_LOG=info,sigorta_server=debug

# Sompo Credentials
SOMPO_BASE_URL=https://ejento.somposigorta.com.tr/dashboard/login
SOMPO_USERNAME=your_username
SOMPO_PASSWORD=your_password
SOMPO_SECRET_KEY=your_google_auth_secret  # Google Authenticator

# Browser
WEBDRIVER_URL=http://localhost:9515
HEADLESS=true
ACCEPT_LANGUAGE=tr-TR,tr;q=0.9
TIMEZONE=Europe/Istanbul

# Timeouts
REQUEST_TIMEOUT_MS=90000
LOGIN_TIMEOUT_MS=45000

# Session
SESSION_DIR=/data/sessions
```

### Client (.env)

```env
NEXT_PUBLIC_APP_NAME=EE Sigorta
NEXT_PUBLIC_API_URL=http://localhost:8099
```

## 📡 API Endpoints

### Backend (Rust) - http://localhost:8099

#### Public Endpoints

```
GET  /health                    → Sunucu durumu
GET  /metrics                   → Prometheus metrics
POST /api/v1/auth/login         → Kullanıcı girişi
POST /api/v1/auth/register      → Yeni kullanıcı kaydı
```

#### Protected Endpoints (JWT gerekli)

```
GET  /api/v1/providers          → Provider listesi ve durumları
POST /api/v1/quote/:provider    → Tek provider'dan teklif
POST /api/v1/quotes/compare     → Tüm provider'lardan karşılaştırmalı
GET  /api/v1/quotes             → Kullanıcının teklifleri
POST /api/v1/policies           → Poliçe kes
GET  /api/v1/policies           → Kullanıcının poliçeleri
```

#### Admin Endpoints (Admin yetkisi gerekli)

```
GET  /api/v1/admin/users        → Tüm kullanıcılar
GET  /api/v1/admin/users/:id    → Kullanıcı detay
GET  /api/v1/admin/logs         → İşlem logları
GET  /api/v1/admin/stats        → Sistem istatistikleri
```

### Örnek Request

```bash
curl -X POST http://localhost:8099/api/v1/quote/sompo \
  -H "Content-Type: application/json" \
  -d '{
    "insured": {
      "tckn": "12345678901",
      "name": "Ahmet Yılmaz",
      "birthDate": "1990-01-01",
      "phone": "5551234567",
      "email": "ahmet@example.com"
    },
    "vehicle": {
      "plate": "34ABC123",
      "brand": "Toyota",
      "model": "Corolla",
      "year": 2020,
      "usage": "hususi"
    },
    "coverage": {
      "productType": "trafik",
      "startDate": "2024-01-15",
      "addons": []
    }
  }'
```

### Örnek Response

```json
{
  "requestId": "req_1234567890_abc123",
  "company": "Sompo Sigorta",
  "productType": "trafik",
  "premium": {
    "net": 3686.44,
    "gross": 4350.0,
    "taxes": 663.56,
    "currency": "TRY"
  },
  "installments": [
    {
      "count": 1,
      "perInstallment": 4350.0,
      "total": 4350.0
    },
    {
      "count": 3,
      "perInstallment": 1450.0,
      "total": 4350.0
    }
  ],
  "coverages": [
    {
      "code": "TRAFIK_ZORUNLU",
      "name": "Zorunlu Trafik Sigortası",
      "limit": null,
      "included": true
    }
  ],
  "warnings": [],
  "timings": {
    "queuedMs": 0,
    "scrapeMs": 8400
  }
}
```

## 🧪 Test

### Rust Testleri

```bash
cd server
cargo test
```

### Parser Unit Test

```bash
# HTML fixture ile fiyat parse testi
cargo test --test parser_test
```

## 🔒 Güvenlik & Uyum

### KVKK & Veri Güvenliği

- Tüm credentials `.env` dosyasında
- Hassas veriler log'larda maskelenir
- Session cookie'leri güvenli şekilde cache'lenir

### Browser Automation

- Resmi API yoksa headless browser scraping
- User-Agent ve timezone ayarları
- Captcha/anti-bot durumunda `HUMAN_ACTION_REQUIRED` hatası
- Rate limiting ve retry mekanizması

### OTP/2FA

- Google Authenticator TOTP desteği
- `SOMPO_SECRET_KEY` ile otomatik kod üretimi
- Manuel OTP girişi gerektiğinde açık hata mesajı

## 🎯 Özellikler

### Mevcut

- ✅ Sompo login + OTP
- ✅ Trafik sigortası teklif alma
- ✅ Session cache (1 saat geçerlilik)
- ✅ Multi-provider mimari
- ✅ Responsive sidebar UI
- ✅ Provider status monitoring
- ✅ Error handling & logging
- ✅ Docker deployment

### Yakında

- 🔜 Kasko sigortası formu
- 🔜 Konut sigortası formu
- 🔜 Sağlık sigortası formu
- 🔜 Teklif karşılaştırma
- 🔜 Poliçe yönetimi
- 🔜 Quick Sigorta entegrasyonu
- 🔜 Webhook desteği
- 🔜 Redis session store

## 🛠️ Geliştirme

### Yeni Provider Ekleme

1. `server/src/providers/` altında yeni klasör oluştur
2. `InsuranceProvider` trait'ini implement et
3. `registry.rs`'ye ekle

```rust
// server/src/providers/yeni_provider/mod.rs
use crate::providers::base::InsuranceProvider;

pub struct YeniProvider {
    config: Arc<Config>,
}

#[async_trait]
impl InsuranceProvider for YeniProvider {
    fn name(&self) -> &str {
        "Yeni Sigorta"
    }

    fn is_active(&self) -> bool {
        true
    }

    async fn fetch_quote(&self, request: QuoteRequest) -> Result<QuoteResponse, ApiError> {
        // Implementation
    }
}
```

### Selector Güncelleme

Sompo portal değiştiğinde:

```rust
// server/src/providers/sompo/selectors.rs
pub const LOGIN_BUTTONS: &'static [&'static str] = &[
    "button[type='submit']",
    "button:has-text('Giriş')",
    // Yeni selector'lar ekle
];
```

## 📊 Monitoring

### Health Check

```bash
curl http://localhost:8099/health
```

### Metrics

```bash
curl http://localhost:8099/metrics
```

### Logs

```bash
# Rust server logs
cd server
RUST_LOG=debug cargo run

# Docker logs
docker-compose logs -f server
```

## 🐛 Sorun Giderme

### ChromeDriver Bağlantı Hatası

```bash
# ChromeDriver'ın çalıştığından emin olun
chromedriver --port=9515

# Docker'da
docker-compose logs chromedriver
```

### Session Kaydetme Hatası

```bash
# Session dizinini oluştur
mkdir -p /data/sessions
chmod 777 /data/sessions
```

### OTP Hatası

```env
# .env dosyasında SOMPO_SECRET_KEY doğru mu?
SOMPO_SECRET_KEY=DD3JCJB7E7H25MB6BZ5IKXLKLJBZDQAO
```

### Frontend Backend Bağlantı Hatası

```bash
# Backend'in çalıştığından emin olun
curl http://localhost:8099/health

# CORS ayarları doğru mu kontrol et
# server/src/main.rs -> CorsLayer::new().allow_origin(Any)
```

## 📝 Notlar

- **Sahte Data Yok**: Pasif provider'lar "Henüz kayıtlı değil" gösterir, mock data döndürmez
- **Session Management**: Cookie cache ile gereksiz login'ler önlenir
- **Selector Dayanıklılığı**: Fallback selector listeleri ile DOM değişikliklerine karşı koruma
- **Error Codes**: LOGIN_FAILED, FORM_VALIDATION, BLOCKED, HUMAN_ACTION_REQUIRED, TIMEOUT, PARSE_ERROR

## 🤝 Katkı

Yeni provider eklemek veya iyileştirme yapmak için:

1. Fork edin
2. Feature branch oluşturun (`git checkout -b feature/yeni-provider`)
3. Commit edin (`git commit -m 'feat: Yeni Provider eklendi'`)
4. Push edin (`git push origin feature/yeni-provider`)
5. Pull Request açın

## 📄 Lisans

Bu proje özel kullanım için geliştirilmiştir.

## 📧 İletişim

Sorular için: info@eesigorta.com

---

## 🚀 Production Deployment

**VDS sunucusuna deploy etmek için**: [DEPLOYMENT.md](./DEPLOYMENT.md) dosyasındaki detaylı kurulum rehberini takip edin.

**Kısa özet**:

1. Ubuntu 22.04 LTS sunucu hazırlayın
2. Docker & Docker Compose kurun
3. Nginx reverse proxy yapılandırın
4. SSL sertifikası alın (Let's Encrypt)
5. `.env` dosyalarını production değerleri ile doldurun
6. `docker compose up -d --build` ile başlatın

**Not**: Bu sistem gerçek sigorta işlemleri için kullanılır. Credentials ve hassas verileri korumaya özen gösterin.
