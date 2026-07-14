use anyhow::{Result, bail};
use buffer_redux::Buffer;
use std::io::stdin;
use wasmparser::{
    Chunk, FuncToValidate, FuncValidatorAllocations, FunctionBody, Parser, ValidPayload, Validator,
    WasmModuleResources,
};

fn main() -> Result<()> {
    // Step 1: parse input, and use Validator on every payload
    let mut parser = Parser::new(0);
    let mut validator = Validator::new();
    let mut buf = Buffer::new_ringbuf();
    let mut eof = false;
    let mut allocs = Default::default();
    let mut stdin = stdin().lock();

    loop {
        match parser.parse(buf.buf(), eof)? {
            Chunk::Parsed { consumed, payload } => {
                match validator.payload(&payload)? {
                    ValidPayload::Func(f, body) => allocs = handle(f, body, allocs)?,
                    ValidPayload::Parser(_) => unimplemented!("component model"),
                    ValidPayload::End(_) => return Ok(()),
                    _ => (),
                }
                buf.consume(consumed);
            }
            Chunk::NeedMoreData(hint) => {
                if eof {
                    bail!("unexpected end");
                }
                buf.reserve(hint as usize);
                if buf.read_from(&mut stdin)? == 0 {
                    eof = true;
                }
            }
        }
    }
}

fn handle<T: WasmModuleResources>(
    f: FuncToValidate<T>,
    body: FunctionBody<'_>,
    allocs: FuncValidatorAllocations,
) -> Result<FuncValidatorAllocations> {
    // Step 2: go operator-by-operator and validate each function
    let mut func_validator = f.into_validator(allocs);
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
    Ok(func_validator.into_allocations())
}
