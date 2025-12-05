// このファイル全体が保護されています
// 理由: 重要な定数や設定値を含むため

/// 計算の精度（小数点以下の桁数）
pub const PRECISION: usize = 2;

/// 計算結果の最大値（これを超えるとエラー）
pub const MAX_RESULT: f64 = 1_000_000.0;

/// 計算結果の最小値（これを下回るとエラー）
pub const MIN_RESULT: f64 = -1_000_000.0;

/// ゼロ除算の判定閾値
pub const EPSILON: f64 = 1e-10;

/// デフォルトの税率（消費税）
pub const TAX_RATE: f64 = 0.10;

/// エラーメッセージ
pub const ERROR_OVERFLOW: &str = "Error: Result exceeds maximum value";
pub const ERROR_UNDERFLOW: &str = "Error: Result is below minimum value";
pub const ERROR_DIVISION_BY_ZERO: &str = "Error: Division by zero";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(PRECISION, 2);
        assert_eq!(TAX_RATE, 0.10);
    }
}
