// build.rs
use std::process::Command;
use std::env;
use std::path::Path;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    
    // Only run module tests if explicitly requested
    if env::var("TEST_MODULES").is_ok() {
        test_modules();
    }
}

fn test_modules() {
    println!("Testing individual modules...");
    
    let modules = vec![
        "database",
        "healing",
        "logging",
        "ml",
        "network",
        "password",
        "threat",
    ];
    
    let mut failures = Vec::new();
    
    for module in modules {
        println!("Testing module: {}", module);
        
        // Create a temporary file to test just this module
        let test_file = format!("
            mod {};
            
            fn main() {{
                println!(\"Module {{}} compiles successfully\", \"{}\");
            }}
        ", module, module);
        
        let test_path = Path::new("target").join("module_test.rs");
        fs::write(&test_path, test_file).expect("Failed to write test file");
        
        // Try to compile just this module
        let output = Command::new("rustc")
            .arg("--crate-type=bin")
            .arg("--edition=2021")
            .arg("-o")
            .arg(format!("target/module_test_{}", module))
            .arg(&test_path)
            .arg("--extern")
            .arg(format!("lobsterpot=target/debug/liblobsterpot.rlib"))
            .output()
            .expect("Failed to execute rustc");
        
        if !output.status.success() {
            println!("Module {} failed to compile:", module);
            println!("{}", String::from_utf8_lossy(&output.stderr));
            failures.push(module);
        } else {
            println!("Module {} compiles successfully", module);
        }
    }
    
    if !failures.is_empty() {
        println!("The following modules failed to compile:");
        for module in failures {
            println!("  - {}", module);
        }
    } else {
        println!("All modules compile successfully!");
    }
}