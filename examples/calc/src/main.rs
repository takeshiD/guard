mod calc;
mod config;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // ========== GUARD PROTECTED START (Lines 10-25) ==========
    // 理由: エラーハンドリングとプログラムの基本構造は
    //       アプリの安定性に関わるため保護
    
    if args.len() < 4 {
        eprintln!("Usage: {} <operation> <num1> <num2>", args[0]);
        eprintln!("Operations: add, sub, mul, div, tax");
        process::exit(1);
    }

    let operation = &args[1];
    let num1: f64 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error: Invalid number '{}'", args[2]);
        process::exit(1);
    });

    // ========== GUARD PROTECTED END ==========

    // この部分は変更OK - 新しい操作の追加など
    let result = match operation.as_str() {
        "add" => {
            let num2: f64 = args[3].parse().unwrap_or(0.0);
            calc::add(num1, num2)
        }
        "sub" => {
            let num2: f64 = args[3].parse().unwrap_or(0.0);
            calc::subtract(num1, num2)
        }
        "mul" => {
            let num2: f64 = args[3].parse().unwrap_or(0.0);
            calc::multiply(num1, num2)
        }
        "div" => {
            let num2: f64 = args[3].parse().unwrap_or(1.0);
            calc::divide(num1, num2)
        }
        "tax" => {
            calc::with_tax(num1)
        }
        _ => {
            eprintln!("Error: Unknown operation '{}'", operation);
            process::exit(1);
        }
    };

    match result {
        Ok(value) => println!("{}", value),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
