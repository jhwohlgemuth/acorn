# Quick test to verify ROR API deserialization fix
Write-Host "Testing ROR API schema fix..." -ForegroundColor Cyan

# Create a minimal test program
$testCode = @'
use acorn_lib::io::api::ror::SearchResponse;

fn main() {
    let json_data = std::fs::read_to_string("untitled-1.json")
        .expect("Failed to read file");
    
    match serde_json::from_str::<SearchResponse>(&json_data) {
        Ok(response) => {
            println!("✓ Successfully deserialized ROR search response");
            println!("  - Number of results: {}", response.number_of_results);
            println!("  - Time taken: {}ms", response.time_taken);
            println!("  - Items: {}", response.items.len());
            println!("  - Meta types: {}", response.meta.types.len());
            println!("  - Meta countries: {}", response.meta.countries.len());
        }
        Err(e) => {
            eprintln!("✗ Deserialization failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
'@

Set-Content -Path "test_ror.rs" -Value $testCode

# Compile and run
Write-Host "Compiling test..." -ForegroundColor Yellow
rustc --edition 2021 test_ror.rs -L target/debug/deps --extern acorn_lib=target/debug/libacorn.rlib 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Compilation successful" -ForegroundColor Green
} else {
    Write-Host "✗ Compilation failed - running full build" -ForegroundColor Red
}

# Cleanup
Remove-Item test_ror.rs -ErrorAction SilentlyContinue
Remove-Item test_ror.exe -ErrorAction SilentlyContinue
Remove-Item test_ror.pdb -ErrorAction SilentlyContinue
