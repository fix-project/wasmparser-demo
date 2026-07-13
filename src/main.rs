use anyhow::Result;
use wasmparser::{Parser, ValidPayload, Validator};

fn main() -> Result<()> {
    // Step 1: read file. (XXX: in the future, stream input to Parser instead of assembling whole module in memory)
    let wasm_bytes = std::io::read_to_string(std::io::stdin())?;

    // Step 2: use wasmparser Validator on every payload (this will give us a list of functions to validate)
    let mut validator = Validator::new();
    let mut functions_to_validate = Vec::new();

    for payload in Parser::new(0).parse_all(wasm_bytes.as_bytes()) {
        if let ValidPayload::Func(func_to_validate, body) = validator.payload(&payload?)? {
            functions_to_validate.push((func_to_validate, body))
        }
    }

    // Step 3: go operator-by-operator and validate each function (avoiding the convenience function)
    for (func_to_validate, body) in functions_to_validate {
        let mut func_validator = func_to_validate.into_validator(Default::default());
        let mut reader = body.get_binary_reader();
        let mut operator_count = 0;
        func_validator.read_locals(&mut reader)?;
        while !reader.eof() {
            print!("Before operator {operator_count}, here are the operands on the stack:");
            for depth in 0..func_validator.operand_stack_height() {
                print!(" {:?}", func_validator.get_operand_type(depth as usize));
            }
            println!();

            reader.visit_operator(&mut func_validator.visitor(reader.original_position()))??;
            operator_count += 1;
        }
        reader.finish_expression(&func_validator.visitor(reader.original_position()))?;
    }

    Ok(())
}
