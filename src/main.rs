use anyhow::Result;
use wasmparser::{Parser, ValidPayload, Validator};

fn main() -> Result<()> {
    // Step 1: read file. (XXX: in the future, stream input to Parser instead of assembling whole module in memory)
    let wasm_bytes = std::io::read_to_string(std::io::stdin())?;

    // Step 2: use wasmparser Validator on every payload (this will give us a list of functions to validate)
    let mut validator = Validator::new();
    let mut functions_to_validate = Vec::new();

    for payload in Parser::new(0).parse_all(wasm_bytes.as_bytes()) {
        if let ValidPayload::Func(validator, body) = validator.payload(&payload?)? {
            functions_to_validate.push((validator, body))
        }
    }

    // Step 3: go operator-by-operator and validate each function (avoiding the convenience function)

    Ok(())
}
