use regex::Regex;

/// Türkçe TL formatını parse eder
/// Örnekler: "4.350,00 TL", "4350 TL", "₺4.350", "300.000 TL"
pub fn parse_tl_price(text: &str) -> Result<f64, String> {
    if text.is_empty() {
        return Err("Boş metin".to_string());
    }
    
    // TL ve ₺ işaretlerini temizle
    let cleaned = text
        .replace("₺", "")
        .replace("TL", "")
        .trim()
        .to_string();
    
    // Türkçe format kontrolü: 300.000,00 (nokta binlik, virgül ondalık)
    let normalized = if cleaned.contains('.') && cleaned.contains(',') {
        // Format: 300.000,00 -> 300000.00
        cleaned.replace('.', "").replace(',', ".")
    } else if cleaned.contains('.') && !cleaned.contains(',') {
        // Format: 300.000 (binlik ayırıcı olarak nokta)
        // Eğer noktadan sonra 3 rakam varsa binlik, 2 veya 1 rakam varsa ondalık
        if let Some(dot_pos) = cleaned.rfind('.') {
            let after_dot = &cleaned[dot_pos + 1..];
            if after_dot.len() == 3 {
                // Binlik ayırıcı
                cleaned.replace('.', "")
            } else {
                // Ondalık ayırıcı - olduğu gibi bırak
                cleaned
            }
        } else {
            cleaned
        }
    } else if cleaned.contains(',') && !cleaned.contains('.') {
        // Format: 300,00 -> 300.00
        cleaned.replace(',', ".")
    } else {
        cleaned
    };
    
    // Regex ile sayıyı çıkar
    let re = Regex::new(r"([0-9]+(?:\.[0-9]+)?)").map_err(|e| e.to_string())?;
    
    if let Some(captures) = re.captures(&normalized) {
        if let Some(matched) = captures.get(1) {
            let number_str = matched.as_str();
            return number_str.parse::<f64>()
                .map_err(|e| format!("Sayı parse hatası: {}", e));
        }
    }
    
    Err(format!("Fiyat parse edilemedi: '{}'", text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_turkish_format() {
        assert_eq!(parse_tl_price("4.350,00 TL").unwrap(), 4350.00);
        assert_eq!(parse_tl_price("300.000,50 TL").unwrap(), 300000.50);
        assert_eq!(parse_tl_price("4350 TL").unwrap(), 4350.0);
        assert_eq!(parse_tl_price("₺4.350").unwrap(), 4350.0);
        assert_eq!(parse_tl_price("1.234,56").unwrap(), 1234.56);
    }

    #[test]
    fn test_parse_simple_format() {
        assert_eq!(parse_tl_price("4350").unwrap(), 4350.0);
        assert_eq!(parse_tl_price("4350.50").unwrap(), 4350.50);
    }

    #[test]
    fn test_parse_errors() {
        assert!(parse_tl_price("").is_err());
        assert!(parse_tl_price("abc").is_err());
    }
}

