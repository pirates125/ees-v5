# Production Checklist ✅

## 🎯 Tamamlanan Özellikler

### ✅ Backend (Rust)

- [x] Multi-provider sistemi (Sompo, Quick, Axa, Anadolu)
- [x] Paralel quote aggregator
- [x] PostgreSQL database layer (SQLx)
- [x] JWT authentication sistem
- [x] Argon2 password hashing
- [x] Admin API endpoints
- [x] Activity logging
- [x] Quote & Policy CRUD
- [x] Session management
- [x] WebDriver automation (Fantoccini)
- [x] OTP/2FA support (TOTP)
- [x] Error handling & logging
- [x] Health & metrics endpoints

### ✅ Frontend (Next.js 15)

- [x] Dark mode support
- [x] Shadcn/ui components
- [x] Theme toggle
- [x] Login/Register pages
- [x] Admin dashboard
- [x] User management UI
- [x] Activity logs UI
- [x] Quote comparison UI
- [x] Multi-provider quote form
- [x] Responsive sidebar
- [x] Toast notifications
- [x] Modern card-based design

### ✅ DevOps

- [x] Docker Compose (dev)
- [x] Docker Compose (production)
- [x] PostgreSQL container
- [x] ChromeDriver container
- [x] Nginx configuration
- [x] Database migrations
- [x] Environment examples

## 🚀 Deployment Hazırlığı

### 1. Environment Variables

#### Server (.env)

```bash
cd server
cp env.example .env

# Düzenle:
# - Sompo credentials (BULUT1, ...)
# - DATABASE_URL
# - JWT_SECRET (güçlü key)
# - Quick/Axa/Anadolu credentials (varsa)
```

#### Client (.env)

```bash
cd client
cp env.example .env

# Production için:
# NEXT_PUBLIC_API_URL=https://yourdomain.com
```

### 2. Database Setup

#### Lokal Test

```bash
# PostgreSQL kur
brew install postgresql@16
brew services start postgresql@16

# Database oluştur
createdb sigorta_db

# .env'de DATABASE_URL'i ayarla
DATABASE_URL=postgresql://postgres@localhost:5432/sigorta_db
```

#### Docker ile

```bash
# PostgreSQL container'ı başlat
docker-compose up postgres -d

# Migration'lar otomatik çalışır
```

### 3. İlk Kullanıcı

Migration otomatik admin kullanıcısı oluşturur:

```
Email: admin@eesigorta.com
Password: admin123
```

**ÖNEMLİ:** İlk girişten sonra şifreyi değiştirin!

### 4. Provider Credentials

Her provider için credentials ekleyin:

```env
# Sompo (aktif)
SOMPO_USERNAME=BULUT1
SOMPO_PASSWORD=EEsigorta.2828
SOMPO_SECRET_KEY=DD3JCJB7E7H25MB6BZ5IKXLKLJBZDQAO

# Quick (opsiyonel)
QUICK_USERNAME=your_username
QUICK_PASSWORD=your_password

# Axa (opsiyonel)
AXA_USERNAME=your_username
AXA_PASSWORD=your_password

# Anadolu (opsiyonel)
ANADOLU_USERNAME=your_username
ANADOLU_PASSWORD=your_password
```

## 🧪 Test Senaryosu

### 1. Servisler Başlat

#### Lokal

```bash
# Terminal 1 - ChromeDriver
chromedriver --port=9515

# Terminal 2 - PostgreSQL (Docker)
docker-compose up postgres -d

# Terminal 3 - Rust Server
cd server
cargo run

# Terminal 4 - Next.js
cd client
npm run dev
```

#### Docker (Tümü)

```bash
docker-compose up --build
```

### 2. Test Adımları

#### A. Health Check

```bash
curl http://localhost:8099/health
# Beklenen: {"ok":true,...}
```

#### B. Provider Durumu

```bash
curl http://localhost:8099/api/v1/providers
# Beklenen: Sompo aktif, diğerleri pasif
```

#### C. Login (Database gerekli)

```bash
curl -X POST http://localhost:8099/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@eesigorta.com",
    "password": "admin123"
  }'
# Beklenen: {"token":"...","user":{...}}
```

#### D. Teklif Al (Login token ile)

```bash
curl -X POST http://localhost:8099/api/v1/quotes/compare \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <TOKEN>" \
  -d '{
    "insured": {...},
    "vehicle": {...},
    "coverage": {...}
  }'
```

### 3. UI Testi

1. **Login**: http://localhost:3000/login

   - Email: admin@eesigorta.com
   - Password: admin123

2. **Dashboard**: Provider durumlarını gör

3. **Dark Mode**: Sağ üst köşedeki toggle

4. **Trafik Formu**:

   - Formu doldur
   - "Karşılaştırmalı Teklif Al" seç
   - Submit et
   - Tüm aktif provider'lardan paralel teklif gelir

5. **Admin Panel**: /admin
   - Kullanıcı listesi
   - İşlem logları
   - İstatistikler

## 🔐 Güvenlik Kontrolleri

- [ ] JWT_SECRET üretim değeri kullanılıyor
- [ ] POSTGRES_PASSWORD güçlü şifre
- [ ] .env dosyaları .gitignore'da
- [ ] HTTPS aktif (production)
- [ ] Firewall yapılandırılmış
- [ ] Default admin şifresi değiştirilmiş
- [ ] Rate limiting aktif
- [ ] CORS doğru yapılandırılmış

## 📊 Monitoring

### Loglar

```bash
# Rust server
docker-compose logs -f server

# PostgreSQL
docker-compose logs -f postgres

# All services
docker-compose logs -f
```

### Metrics

```bash
curl http://localhost:8099/metrics
```

### Database

```bash
# PostgreSQL'e bağlan
docker exec -it <postgres-container> psql -U postgres sigorta_db

# Kullanıcı sayısı
SELECT COUNT(*) FROM users;

# Teklif sayısı
SELECT COUNT(*) FROM quotes;

# Poliçe sayısı
SELECT COUNT(*) FROM policies;
```

## 🐛 Bilinen Sorunlar

### Sompo Login Hatası

- **Sorun**: "Login başarısız - hala login sayfasında"
- **Çözüm**:
  - HEADLESS=false yapıp tarayıcıda gözlemle
  - XPath selector'lar güncel mi kontrol et
  - OTP SECRET_KEY doğru mu kontrol et

### CSS Yüklenmiyor

- **Sorun**: Tailwind CSS aktif değil
- **Çözüm**:
  - `.next` klasörünü sil
  - `npm run dev` yeniden başlat
  - Incognito mode kullan (extension'lar)

### Database Bağlantı Hatası

- **Sorun**: "Connection refused"
- **Çözüm**:
  - PostgreSQL çalışıyor mu kontrol et
  - DATABASE_URL doğru mu kontrol et
  - Migration'lar çalıştı mı kontrol et

## 📈 Performance Optimizasyonu

### Tarayıcı Otomasyonu

- **Headless Mode**: Production'da HEADLESS=true
- **User Agent Rotation**: Farklı UA'lar kullan
- **Proxy**: Rate limiting'den kaçınmak için

### Database

- **Connection Pool**: Max 10 connection
- **Index'ler**: Tüm foreign key'lerde index var
- **JSONB**: Flexible data storage

### Caching (Gelecek)

- **Redis**: Session ve quote cache
- **TTL**: Quote'lar 2 saat geçerli

## 🎓 Ekip Eğitimi

### Yeni Provider Ekleme

1. `server/src/providers/yeni_provider/` klasörü oluştur
2. `mod.rs`, `login.rs`, `quote.rs`, `parser.rs`, `selectors.rs` ekle
3. `InsuranceProvider` trait'ini implement et
4. `registry.rs`'ye ekle
5. Env variable'ları ekle

### Yeni Sayfa Ekleme

1. `client/app/(dashboard)/yeni_sayfa/` oluştur
2. `page.tsx` ekle
3. Sidebar'a navigation item ekle
4. Server action / API call ekle

## 📞 Destek

- **Dokümantasyon**: README.md, DEPLOYMENT.md
- **Issues**: GitHub Issues
- **Email**: info@eesigorta.com
