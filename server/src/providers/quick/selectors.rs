pub struct QuickSelectors;

impl QuickSelectors {
    // Login selectors (Python backend'den referans)
    pub const USERNAME_INPUTS: &'static [&'static str] = &[
        "input[name='username']",
        "input[name='email']",
        "input[name='user']",
        "input[type='email']",
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
        "input[type='submit']",
        "button:has-text('Giriş')",
        "button:has-text('Login')",
        ".login-btn",
    ];
    
    // Form selectors
    pub const PLATE_INPUTS: &'static [&'static str] = &[
        "input[name='plaka']",
        "input[name='plate']",
        "input[placeholder*='plaka']",
        "#plaka",
    ];
    
    pub const FORM_SUBMIT_BUTTONS: &'static [&'static str] = &[
        "button[type='submit']",
        "button:has-text('Teklif Al')",
        "button:has-text('Sorgula')",
    ];
    
    // Price selectors
    pub const PRICE_ELEMENTS: &'static [&'static str] = &[
        ".price",
        ".fiyat",
        ".premium",
        ".prim",
        "[class*='price']",
        "[class*='fiyat']",
        "td:has-text('TL')",
        "span:has-text('TL')",
    ];
}

