use crate::http::ApiError;

/// Email notification service
/// TODO: Implement actual email sending with SMTP (e.g., using lettre crate)
pub struct EmailService {
    from_email: String,
    smtp_configured: bool,
}

impl EmailService {
    pub fn new() -> Self {
        let from_email = std::env::var("EMAIL_FROM")
            .unwrap_or_else(|_| "noreply@eesigorta.com".to_string());
        
        let smtp_configured = std::env::var("SMTP_HOST").is_ok() 
            && std::env::var("SMTP_USER").is_ok();

        if !smtp_configured {
            tracing::warn!("📧 Email servisi yapılandırılmamış (SMTP credentials eksik)");
        } else {
            tracing::info!("✅ Email servisi hazır");
        }

        Self {
            from_email,
            smtp_configured,
        }
    }

    /// Send quote ready notification
    pub async fn send_quote_ready(&self, to_email: &str, quote_id: &str, provider: &str) -> Result<(), ApiError> {
        if !self.smtp_configured {
            tracing::debug!("Email gönderimi atlandı (SMTP yapılandırılmamış)");
            return Ok(());
        }

        // TODO: Implement actual email sending
        tracing::info!(
            "📧 [MOCK] Quote ready email: {} -> {} (Quote: {}, Provider: {})",
            self.from_email,
            to_email,
            quote_id,
            provider
        );

        Ok(())
    }

    /// Send policy created notification
    pub async fn send_policy_created(&self, to_email: &str, policy_number: &str, provider: &str) -> Result<(), ApiError> {
        if !self.smtp_configured {
            tracing::debug!("Email gönderimi atlandı (SMTP yapılandırılmamış)");
            return Ok(());
        }

        // TODO: Implement actual email sending
        tracing::info!(
            "📧 [MOCK] Policy created email: {} -> {} (Policy: {}, Provider: {})",
            self.from_email,
            to_email,
            policy_number,
            provider
        );

        Ok(())
    }

    /// Send welcome email
    pub async fn send_welcome(&self, to_email: &str, name: &str) -> Result<(), ApiError> {
        if !self.smtp_configured {
            tracing::debug!("Email gönderimi atlandı (SMTP yapılandırılmamış)");
            return Ok(());
        }

        // TODO: Implement actual email sending
        tracing::info!(
            "📧 [MOCK] Welcome email: {} -> {} (Name: {})",
            self.from_email,
            to_email,
            name
        );

        Ok(())
    }
}

impl Default for EmailService {
    fn default() -> Self {
        Self::new()
    }
}

