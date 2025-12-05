// このファイルは保護されていません
// 理由: 計算機能の追加・変更は自由に行えます

use crate::config::*;

/// 加算
pub fn add(a: f64, b: f64) -> Result<f64, String> {
    let result = a + b;
    validate_result(result)?;
    Ok(round(result))
}

/// 減算
pub fn subtract(a: f64, b: f64) -> Result<f64, String> {
    let result = a - b;
    validate_result(result)?;
    Ok(round(result))
}

/// 乗算
pub fn multiply(a: f64, b: f64) -> Result<f64, String> {
    let result = a * b;
    validate_result(result)?;
    Ok(round(result))
}

/// 除算
pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b.abs() < EPSILON {
        return Err(ERROR_DIVISION_BY_ZERO.to_string());
    }
    let result = a / b;
    validate_result(result)?;
    Ok(round(result))
}

/// 税込価格を計算
pub fn with_tax(price: f64) -> Result<f64, String> {
    let result = price * (1.0 + TAX_RATE);
    validate_result(result)?;
    Ok(round(result))
}

// ヘルパー関数（エージェントが追加機能を実装しやすいように）

fn validate_result(value: f64) -> Result<(), String> {
    if value > MAX_RESULT {
        return Err(ERROR_OVERFLOW.to_string());
    }
    if value < MIN_RESULT {
        return Err(ERROR_UNDERFLOW.to_string());
    }
    Ok(())
}

fn round(value: f64) -> f64 {
    let multiplier = 10_f64.powi(PRECISION as i32);
    (value * multiplier).round() / multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2.0, 3.0).unwrap(), 5.0);
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(10.0, 2.0).unwrap(), 5.0);
    }

    #[test]
    fn test_divide_by_zero() {
        assert!(divide(10.0, 0.0).is_err());
    }

    #[test]
    fn test_with_tax() {
        assert_eq!(with_tax(100.0).unwrap(), 110.0);
    }
}
