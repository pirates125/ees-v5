/// Sompo portal için CSS/XPath selector'ları
/// Python kodundan referans alınarak oluşturulmuştur

pub struct SompoSelectors;

impl SompoSelectors {
    // Login selectors (CSS ve XPath kombinasyonu)
    // Python backend'den alınan spesifik XPath'ler
    pub const USERNAME_XPATH: &'static str = "/html/body/div[1]/div/div[1]/div[2]/form/div[1]/div/input";
    pub const PASSWORD_XPATH: &'static str = "/html/body/div[1]/div/div[1]/div[2]/form/div[2]/div/div/input";
    
    pub const USERNAME_INPUTS: &'static [&'static str] = &[
        "input[type='text']",
        "input[name='username']",
        "input[name='email']",
        "#username",
        "#email",
    ];
    
    pub const PASSWORD_INPUTS: &'static [&'static str] = &[
        "input[type='password']",
        "input[name='password']",
        "#password",
    ];
    
    pub const LOGIN_BUTTONS: &'static [&'static str] = &[
        "button[type='submit']",
        "button:has-text('Giriş')",
        "button:has-text('GİRİŞ')",
        "button:has-text('Login')",
        "input[type='submit']",
    ];
    
    // OTP selectors
    pub const OTP_INPUTS: &'static [&'static str] = &[
        "input[placeholder*='OTP']",
        "input[placeholder*='Kod']",
        "input[placeholder*='Doğrulama']",
        "input[autocomplete='one-time-code']",
        "input[type='tel']",
        "input[inputmode='numeric']",
        "#otp",
        "input[name='otp']",
    ];
    
    pub const OTP_SUBMIT_BUTTONS: &'static [&'static str] = &[
        "button[type='submit']",
        "button:has-text('Doğrula')",
        "button:has-text('Onayla')",
        "button:has-text('Gönder')",
    ];
    
    // Dashboard/success indicators
    pub const DASHBOARD_INDICATORS: &'static [&'static str] = &[
        "[class*='dashboard']",
        "[class*='home']",
        "a:has-text('Çıkış')",
        "a:has-text('Profil')",
    ];
    
    // Product navigation
    pub const TRAFIK_LINKS: &'static [&'static str] = &[
        "a:has-text('Trafik')",
        "a:has-text('Trafik Sigortası')",
        "a[href*='trafik']",
        ".trafik-link",
        "#trafik",
    ];
    
    pub const KASKO_LINKS: &'static [&'static str] = &[
        "a:has-text('Kasko')",
        "a:has-text('Kasko Sigortası')",
        "a[href*='kasko']",
        ".kasko-link",
        "#kasko",
    ];
    
    // Form inputs
    pub const PLATE_INPUTS: &'static [&'static str] = &[
        "input[name='plaka']",
        "input[name='plate']",
        "input[placeholder*='plaka']",
        "input[placeholder*='Plaka']",
        "#plaka",
        "#plate",
    ];
    
    pub const TCKN_INPUTS: &'static [&'static str] = &[
        "input[name='tckn']",
        "input[name='kimlikNo']",
        "input[placeholder*='TC']",
        "input[placeholder*='Kimlik']",
        "#tckn",
        "#kimlikNo",
    ];
    
    pub const RUHSAT_SERI_INPUTS: &'static [&'static str] = &[
        "input[name='ruhsatSeri']",
        "input[name='ruhsat']",
        "input[placeholder*='ruhsat']",
        "input[placeholder*='Seri']",
        "#ruhsatSeri",
    ];
    
    pub const FORM_SUBMIT_BUTTONS: &'static [&'static str] = &[
        "button[type='submit']",
        "input[type='submit']",
        "button:has-text('Teklif Al')",
        "button:has-text('TEKLİF AL')",
        "button:has-text('Sorgula')",
        "button:has-text('SORGULA')",
        ".submit-btn",
        "#submit-btn",
    ];
    
    // Price/result selectors
    pub const PRICE_ELEMENTS: &'static [&'static str] = &[
        ".premium",
        ".prim",
        ".amount",
        ".cost",
        ".price",
        ".fiyat",
        "[class*='premium']",
        "[class*='prim']",
        "[class*='amount']",
        "[class*='fiyat']",
        "td:has-text('TL')",
        "span:has-text('TL')",
        "div:has-text('TL')",
    ];
    
    // Loading/waiting indicators
    pub const LOADING_INDICATORS: &'static [&'static str] = &[
        ".loading",
        ".spinner",
        "[class*='loading']",
        "[class*='spinner']",
    ];
}

