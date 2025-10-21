#!/usr/bin/env python3
"""
Flask Backend Runner
Bu script Flask backend'ini çalıştırır
"""

import os
import sys
from pathlib import Path

# Backend modüllerini import etmek için path ekle
sys.path.append(str(Path(__file__).parent))

from app.main import app

if __name__ == "__main__":
    print("🚀 Flask Backend Başlatılıyor...")
    print("📡 API Endpoints:")
    print("   - Dashboard: http://localhost:5001/api/dashboard")
    print("   - Teklifler: http://localhost:5001/api/offers")
    print("   - Trafik: http://localhost:5001/api/trafik")
    print("   - Kasko: http://localhost:5001/api/kasko")
    print("=" * 50)
    
    # Flask uygulamasını çalıştır
    app.run(
        host="0.0.0.0",
        port=5001,
        debug=True,
        threaded=True
    )
