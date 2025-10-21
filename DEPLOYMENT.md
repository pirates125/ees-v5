# EE Sigorta - VDS Production Deployment Guide

Production ortamında VDS sunucusuna kurulum rehberi.

## 🖥️ Sunucu Gereksinimleri

### Minimum Gereksinimler

- **CPU**: 2 vCPU
- **RAM**: 4 GB
- **Disk**: 40 GB SSD
- **OS**: Ubuntu 22.04 LTS veya üzeri
- **Network**: 100 Mbps bağlantı

### Önerilen Gereksinimler

- **CPU**: 4 vCPU
- **RAM**: 8 GB
- **Disk**: 80 GB SSD
- **OS**: Ubuntu 22.04 LTS

## 📦 Kurulum Adımları

### 1. Sistem Güncellemesi

```bash
sudo apt update && sudo apt upgrade -y
```

### 2. Docker Kurulumu

```bash
# Docker GPG anahtarını ekle
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# Docker repository'sini ekle
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Docker'ı kur
sudo apt update
sudo apt install docker-ce docker-ce-cli containerd.io docker-compose-plugin -y

# Docker servisini başlat
sudo systemctl start docker
sudo systemctl enable docker

# Kullanıcıyı docker grubuna ekle
sudo usermod -aG docker $USER
newgrp docker
```

### 3. Proje Dosyalarını Yükle

```bash
# Ana dizini oluştur
mkdir -p /opt/sigorta-platform
cd /opt/sigorta-platform

# Git ile projeyi klonla (veya FTP/SCP ile yükle)
git clone <your-repo-url> .

# Veya manuel upload
# scp -r v5/ user@server:/opt/sigorta-platform/
```

### 4. Environment Dosyalarını Yapılandır

#### Backend .env

```bash
cd /opt/sigorta-platform/server
cp env.example .env
nano .env
```

**server/.env** içeriği:

```env
# Database
DATABASE_URL=postgresql://sigorta_user:STRONG_PASSWORD_HERE@postgres:5432/sigorta_db

# JWT Secret (güçlü bir secret oluştur)
JWT_SECRET=your-super-secret-jwt-key-min-32-characters-long

# Server
HTTP_ADDR=0.0.0.0:8099
RUST_LOG=info

# Sompo Credentials (gerçek bilgileriniz)
SOMPO_BASE_URL=https://ejento.somposigorta.com.tr/dashboard/login
SOMPO_USERNAME=your_username
SOMPO_PASSWORD=your_password
SOMPO_SECRET_KEY=your_google_auth_secret

# Browser
WEBDRIVER_URL=http://chromedriver:9515
HEADLESS=true
ACCEPT_LANGUAGE=tr-TR,tr;q=0.9
TIMEZONE=Europe/Istanbul

# Timeouts
REQUEST_TIMEOUT_MS=90000
LOGIN_TIMEOUT_MS=45000

# Session
SESSION_DIR=/data/sessions
```

#### Frontend .env

```bash
cd /opt/sigorta-platform/client
cp .env.example .env.production
nano .env.production
```

**client/.env.production** içeriği:

```env
NEXT_PUBLIC_APP_NAME=EE Sigorta
NEXT_PUBLIC_API_URL=https://api.yourdomain.com
```

### 5. Nginx Reverse Proxy Kurulumu

```bash
sudo apt install nginx -y
```

**nginx.conf** dosyası oluştur:

```bash
sudo nano /etc/nginx/sites-available/sigorta-platform
```

```nginx
# Frontend (Next.js)
server {
    listen 80;
    server_name yourdomain.com www.yourdomain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Backend (Rust API)
server {
    listen 80;
    server_name api.yourdomain.com;

    client_max_body_size 10M;

    location / {
        proxy_pass http://localhost:8099;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # CORS headers
        add_header Access-Control-Allow-Origin * always;
        add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Authorization, Content-Type" always;
    }
}
```

```bash
# Site'ı aktifleştir
sudo ln -s /etc/nginx/sites-available/sigorta-platform /etc/nginx/sites-enabled/

# Nginx'i test et ve yeniden başlat
sudo nginx -t
sudo systemctl restart nginx
```

### 6. SSL Sertifikası (Let's Encrypt)

```bash
# Certbot kurulumu
sudo apt install certbot python3-certbot-nginx -y

# SSL sertifikası al
sudo certbot --nginx -d yourdomain.com -d www.yourdomain.com -d api.yourdomain.com

# Otomatik yenileme testi
sudo certbot renew --dry-run
```

### 7. Docker Compose ile Servisleri Başlat

```bash
cd /opt/sigorta-platform

# Servisleri başlat (detached mode)
docker compose up -d --build

# Logları kontrol et
docker compose logs -f
```

### 8. Database Migration & Admin Kullanıcısı

```bash
# Backend otomatik migration yapar, logları kontrol et:
docker compose logs server | grep -i migration

# Admin kullanıcısı oluştur
curl -X POST http://localhost:8099/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@eesigorta.com",
    "password": "StrongPassword123!",
    "name": "Admin User"
  }'

# Kullanıcıyı admin yap
docker exec -it v5-postgres-1 psql -U sigorta_user -d sigorta_db -c \
  "UPDATE users SET role = 'admin' WHERE email = 'admin@eesigorta.com';"
```

## 🔧 Servis Yönetimi

### Servisleri Başlat/Durdur

```bash
cd /opt/sigorta-platform

# Başlat
docker compose up -d

# Durdur
docker compose down

# Yeniden başlat
docker compose restart

# Servisleri yeniden build et
docker compose up -d --build
```

### Logları Görüntüle

```bash
# Tüm servislerin logları
docker compose logs -f

# Sadece backend
docker compose logs -f server

# Sadece frontend
docker compose logs -f client

# Sadece PostgreSQL
docker compose logs -f postgres

# Son 100 satır
docker compose logs --tail=100
```

### Container Durumunu Kontrol Et

```bash
# Çalışan container'lar
docker compose ps

# Kaynak kullanımı
docker stats

# Container detayları
docker inspect <container_name>
```

## 🔍 Monitoring & Health Check

### Health Check Endpoints

```bash
# Backend health
curl https://api.yourdomain.com/health

# Beklenen response:
# {"ok":true,"version":"0.1.0","uptime_seconds":123,"timestamp":"2024-01-15T10:30:00Z"}

# Frontend (browser)
https://yourdomain.com
```

### Disk Kullanımı

```bash
# Disk durumu
df -h

# Docker disk kullanımı
docker system df

# Kullanılmayan image'leri temizle
docker system prune -a

# Sadece durmuş container'ları temizle
docker container prune
```

### PostgreSQL Backup

```bash
# Backup oluştur
docker exec v5-postgres-1 pg_dump -U sigorta_user sigorta_db > backup_$(date +%Y%m%d).sql

# Backup'ı sıkıştır
gzip backup_$(date +%Y%m%d).sql

# Backup'ı geri yükle
gunzip -c backup_20241021.sql.gz | docker exec -i v5-postgres-1 psql -U sigorta_user sigorta_db

# Otomatik backup scripti (crontab)
# 0 2 * * * /opt/sigorta-platform/scripts/backup.sh
```

### Log Rotation

```bash
# Docker loglarını temizle
docker compose logs --no-log-prefix > /dev/null

# Logrotate yapılandırması
sudo nano /etc/logrotate.d/docker-containers
```

```
/var/lib/docker/containers/*/*.log {
  rotate 7
  daily
  compress
  size=10M
  missingok
  delaycompress
  copytruncate
}
```

## 🔐 Güvenlik

### Firewall (UFW)

```bash
# UFW'yi aktifleştir
sudo ufw enable

# HTTP/HTTPS izin ver
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# SSH izin ver (port değiştirdiyseniz onu kullanın)
sudo ufw allow 22/tcp

# Gereksiz portları kapat
sudo ufw deny 3000/tcp  # Next.js direkt erişimi engelle
sudo ufw deny 8099/tcp  # Backend direkt erişimi engelle
sudo ufw deny 5432/tcp  # PostgreSQL direkt erişimi engelle

# Durumu kontrol et
sudo ufw status verbose
```

### Fail2Ban (Opsiyonel)

```bash
# Fail2ban kurulumu
sudo apt install fail2ban -y

# Nginx için jail oluştur
sudo nano /etc/fail2ban/jail.local
```

```ini
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 5

[nginx-limit-req]
enabled = true
filter = nginx-limit-req
logpath = /var/log/nginx/error.log

[nginx-botsearch]
enabled = true
filter = nginx-botsearch
logpath = /var/log/nginx/access.log
maxretry = 2
```

```bash
sudo systemctl restart fail2ban
sudo fail2ban-client status
```

### SSL/TLS Güvenlik

Nginx SSL yapılandırması (certbot otomatik ekler):

```nginx
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers HIGH:!aNULL:!MD5;
ssl_prefer_server_ciphers on;
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
```

## 🔄 Güncelleme Prosedürü

### Kod Güncellemesi

```bash
cd /opt/sigorta-platform

# Git ile güncelle
git pull origin main

# Docker compose'u yeniden build et
docker compose down
docker compose up -d --build

# Logları kontrol et
docker compose logs -f

# Health check
curl http://localhost:8099/health
```

### Database Migration

```bash
# Backend otomatik migration yapar
# Migration loglarını kontrol et:
docker compose logs server | grep -i migration

# Manuel migration gerekirse:
docker exec -it v5-postgres-1 psql -U sigorta_user -d sigorta_db -f /path/to/migration.sql
```

### Zero-Downtime Deployment (Gelişmiş)

```bash
# Blue-green deployment için iki set container kullan
docker compose -f docker-compose.blue.yml up -d
# Test et
docker compose -f docker-compose.green.yml up -d
# Nginx'i yeni container'lara yönlendir
# Eski container'ları kapat
```

## ⚡ Performance Tuning

### Docker Resource Limits

**docker-compose.yml** içinde:

```yaml
services:
  server:
    deploy:
      resources:
        limits:
          cpus: "2"
          memory: 2G
        reservations:
          cpus: "1"
          memory: 1G

  client:
    deploy:
      resources:
        limits:
          cpus: "1"
          memory: 1G
        reservations:
          cpus: "0.5"
          memory: 512M
```

### PostgreSQL Tuning

```bash
docker exec -it v5-postgres-1 psql -U sigorta_user -d sigorta_db

-- 4GB RAM için önerilen ayarlar:
ALTER SYSTEM SET shared_buffers = '1GB';
ALTER SYSTEM SET effective_cache_size = '3GB';
ALTER SYSTEM SET maintenance_work_mem = '256MB';
ALTER SYSTEM SET work_mem = '32MB';
ALTER SYSTEM SET max_connections = '100';
ALTER SYSTEM SET checkpoint_completion_target = '0.9';
ALTER SYSTEM SET wal_buffers = '16MB';

-- Restart gerekli
docker compose restart postgres
```

### Nginx Caching

```nginx
# Static asset caching
location ~* \.(jpg|jpeg|png|gif|ico|css|js|woff|woff2)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

# API response caching (dikkatli kullan)
proxy_cache_path /var/cache/nginx levels=1:2 keys_zone=api_cache:10m max_size=100m;
proxy_cache api_cache;
proxy_cache_valid 200 1m;
```

## 🐛 Troubleshooting

### Backend 500 Hatası

```bash
# Backend loglarını kontrol et
docker compose logs server --tail=100

# Database bağlantısını test et
docker exec -it v5-postgres-1 psql -U sigorta_user -d sigorta_db -c "SELECT 1;"

# Environment değişkenlerini kontrol et
docker compose config

# Container'ı yeniden başlat
docker compose restart server
```

### ChromeDriver Bağlantı Hatası

```bash
# ChromeDriver konteynerını kontrol et
docker compose logs chromedriver

# ChromeDriver'ın çalıştığını test et
curl http://localhost:9515/status

# Yeniden başlat
docker compose restart chromedriver server
```

### Nginx 502 Bad Gateway

```bash
# Backend'in çalıştığını kontrol et
docker compose ps
curl http://localhost:8099/health

# Nginx loglarını kontrol et
sudo tail -f /var/log/nginx/error.log

# Nginx'i test et
sudo nginx -t

# Nginx'i yeniden başlat
sudo systemctl restart nginx
```

### Frontend Sayfa Yüklenmiyor

```bash
# Next.js container logları
docker compose logs client --tail=50

# Port çakışması kontrolü
sudo netstat -tulpn | grep :3000

# Container'ı yeniden başlat
docker compose restart client
```

### Database Bağlantı Hatası

```bash
# PostgreSQL loglarını kontrol et
docker compose logs postgres

# Database'e manuel bağlan
docker exec -it v5-postgres-1 psql -U sigorta_user -d sigorta_db

# Bağlantı sayısını kontrol et
docker exec -it v5-postgres-1 psql -U sigorta_user -d sigorta_db -c \
  "SELECT count(*) FROM pg_stat_activity;"

# DATABASE_URL'i kontrol et
docker compose exec server env | grep DATABASE_URL
```

### Disk Dolu Hatası

```bash
# Disk kullanımını kontrol et
df -h

# Docker volumes temizle
docker volume prune

# Eski log dosyalarını temizle
sudo journalctl --vacuum-time=7d

# Eski backup dosyalarını temizle
find /opt/sigorta-platform/backups -mtime +30 -delete
```

## 📊 Port Listesi

| Servis       | Internal Port | External Port | Açıklama                           |
| ------------ | ------------- | ------------- | ---------------------------------- |
| Frontend     | 3000          | 80/443        | Next.js (Nginx reverse proxy)      |
| Backend      | 8099          | 80/443        | Rust API (Nginx reverse proxy)     |
| PostgreSQL   | 5432          | -             | Database (internal only)           |
| ChromeDriver | 9515          | -             | Browser automation (internal only) |

## 📝 Checklist - İlk Kurulum

### Ön Hazırlık

- [ ] VDS sunucusu hazır (Ubuntu 22.04)
- [ ] Domain name'ler DNS'e tanımlandı
- [ ] SSH erişimi aktif
- [ ] Root veya sudo yetkisi var

### Kurulum

- [ ] Sistem güncellendi
- [ ] Docker ve Docker Compose kuruldu
- [ ] Proje dosyaları yüklendi
- [ ] `server/.env` dosyası yapılandırıldı
- [ ] `client/.env.production` dosyası yapılandırıldı
- [ ] Nginx kuruldu ve yapılandırıldı
- [ ] SSL sertifikası alındı (Let's Encrypt)

### Deployment

- [ ] Docker servisleri başlatıldı
- [ ] Database migration çalıştırıldı
- [ ] Admin kullanıcısı oluşturuldu
- [ ] Health check başarılı (backend)
- [ ] Frontend erişilebilir (browser)
- [ ] Login çalışıyor

### Güvenlik

- [ ] Firewall yapılandırıldı (UFW)
- [ ] Fail2ban kuruldu
- [ ] SSL/TLS aktif
- [ ] `.env` dosyaları güvenli (chmod 600)

### Monitoring

- [ ] Backup stratejisi kuruldu
- [ ] Log rotation aktif
- [ ] Disk kullanımı izleniyor
- [ ] Health check monitoring kuruldu

## 🔧 Maintenance Script'leri

### Otomatik Backup Script

**scripts/backup.sh**:

```bash
#!/bin/bash
BACKUP_DIR="/opt/sigorta-platform/backups"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# Database backup
docker exec v5-postgres-1 pg_dump -U sigorta_user sigorta_db | gzip > $BACKUP_DIR/db_$DATE.sql.gz

# .env dosyalarını backup
tar -czf $BACKUP_DIR/env_$DATE.tar.gz server/.env client/.env.production

# 30 günden eski backup'ları sil
find $BACKUP_DIR -type f -mtime +30 -delete

echo "Backup completed: $DATE"
```

```bash
chmod +x scripts/backup.sh

# Crontab'a ekle (her gün 02:00)
crontab -e
# 0 2 * * * /opt/sigorta-platform/scripts/backup.sh >> /var/log/sigorta-backup.log 2>&1
```

### Health Check Script

**scripts/health-check.sh**:

```bash
#!/bin/bash

# Backend health check
BACKEND_HEALTH=$(curl -s http://localhost:8099/health | jq -r '.ok')

if [ "$BACKEND_HEALTH" != "true" ]; then
    echo "Backend unhealthy, restarting..."
    cd /opt/sigorta-platform
    docker compose restart server
fi

# Frontend check
FRONTEND_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000)

if [ "$FRONTEND_STATUS" != "200" ]; then
    echo "Frontend unhealthy, restarting..."
    cd /opt/sigorta-platform
    docker compose restart client
fi
```

## 📞 Destek & Sorun Giderme

### Log Dosyaları Konumları

- **Nginx**: `/var/log/nginx/`
- **Docker**: `docker compose logs`
- **Sistem**: `/var/log/syslog`

### Yararlı Komutlar

```bash
# Tüm servislerin durumu
docker compose ps

# Kaynak kullanımı
docker stats

# Disk kullanımı
du -sh /opt/sigorta-platform/*

# Aktif bağlantılar
sudo netstat -tulpn

# Sistem kaynakları
htop
```

### İletişim

- **Email**: info@eesigorta.com
- **Logs**: `/var/log/sigorta-platform/`

---

## ⚠️ Önemli Notlar

1. **Credentials Güvenliği**: `.env` dosyalarını asla commit etmeyin veya paylaşmayın
2. **Backup**: Günlük otomatik backup alın
3. **SSL**: Let's Encrypt sertifikaları 90 günde bir yenilenir (otomatik)
4. **Monitoring**: Health check'leri düzenli çalıştırın
5. **Updates**: Güvenlik güncellemelerini takip edin

**Production ortamında test etmeden değişiklik yapmayın!**
