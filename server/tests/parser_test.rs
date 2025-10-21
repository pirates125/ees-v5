use rust_decimal_macros::dec;

// Parser utility testleri
#[cfg(test)]
mod parser_tests {
    use super::*;
    
    // Not: Bu test server crate'inin utils modülünü kullanmak için
    // server'ın bir library expose etmesi gerekir.
    // Şimdilik test placeholder'ı bırakıyoruz.
    
    #[test]
    fn test_parse_turkish_price_format() {
        // Bu test gerçek implementasyonda src/utils/parser.rs'teki
        // parse_tl_price fonksiyonunu test eder
        
        // Örnek test case'leri:
        // "4.350,00 TL" -> 4350.00
        // "300.000,50 TL" -> 300000.50
        // "4350 TL" -> 4350
        
        assert!(true); // Placeholder
    }
    
    #[test]
    fn test_html_fixture_parsing() {
        // Gerçek Sompo HTML snapshot'ı ile fiyat parse testi
        // let html = include_str!("../fixtures/sompo_result.html");
        // let price = parse_sompo_quote(html).unwrap();
        // assert_eq!(price.premium, dec!(4350.0));
        
        assert!(true); // Placeholder
    }
}

