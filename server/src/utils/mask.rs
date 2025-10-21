/// Maskeler hassas verileri log'larda
pub fn mask_sensitive(value: &str) -> String {
    if value.is_empty() {
        return "".to_string();
    }
    
    let len = value.len();
    if len <= 4 {
        return "*".repeat(len);
    }
    
    // İlk 2 ve son 2 karakter hariç hepsini maskele
    format!(
        "{}{}{}",
        &value[..2],
        "*".repeat(len - 4),
        &value[len - 2..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        assert_eq!(mask_sensitive("MyPassword123"), "My*********23");
        assert_eq!(mask_sensitive("abc"), "***");
        assert_eq!(mask_sensitive(""), "");
    }
}

