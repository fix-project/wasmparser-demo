use anyhow::Result;
use std::io::{Read, stdin};
use wasmparser::{
    Chunk, FuncToValidate, FunctionBody, Parser, ValidPayload, Validator, WasmModuleResources,
};

fn main() -> Result<()> {
    // Step 1: parse input, and use Validator on every payload (this will give us a list of functions to validate)
    let mut parser = Parser::new(0);
    let mut validator = Validator::new();
    let mut buf = Vec::new();
    let mut eof = false;

    loop {
        match parser.parse(&buf, eof)? {
            Chunk::NeedMoreData(hint) => {
                let current_len = buf.len();
                buf.resize(current_len + hint as usize, 0);
                let bytes_read = stdin().read(&mut buf[current_len..])?;
                buf.truncate(current_len + bytes_read);
                if bytes_read == 0 {
                    eof = true;
                }
                continue;
            }
            Chunk::Parsed { consumed, payload } => {
                match validator.payload(&payload)? {
                    ValidPayload::Func(func_to_validate, body) => handle(func_to_validate, body)?,
                    ValidPayload::End(_) => return Ok(()),
                    _ => (),
                }

                buf.drain(..consumed); // XXX would be better to have some sort of ring buffer
            }
        }
    }
}

fn handle<'a, T: WasmModuleResources>(f: FuncToValidate<T>, body: FunctionBody<'a>) -> Result<()> {
    // Step 2: go operator-by-operator and validate each function (avoiding the convenience function)
    let mut func_validator = f.into_validator(Default::default());
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
    Ok(())
}
