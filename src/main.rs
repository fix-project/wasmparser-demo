use anyhow::{Result, bail};
use buffer_redux::{BufReader, BufWriter, Buffer, policy::ReaderPolicy};
use std::{fmt::write, io::{BufRead, Read, Write, stdin, stdout}, fs::File};
use wasmparser::{
    Chunk, FuncToValidate, FuncType, FuncValidatorAllocations, FunctionBody, Parser, Payload,
    ValidPayload, Validator, WasmModuleResources,
};

struct ModuleState {
    func_types: Vec<FuncType>,
    // imports: Vec<Import>,
    // func_type_ids: Vec<u32>,
    // TODO: add more
}
 
fn get_input_stream() -> BufReader<impl Read> {
    BufReader::new_ringbuf(stdin())
}

fn get_output_stream() -> BufWriter<impl Write> {
    BufWriter::new_ringbuf(stdout())
}

fn compile_from_stream(
    input_stream: &mut BufReader<impl Read>,
    output_stream: &mut BufWriter<impl Write>
) -> Result<()> {
    let mut parser = Parser::new(0);
    let mut eof = false;

    // VALIDATOR stuff
    let mut validator = Validator::new();
    let mut allocs = FuncValidatorAllocations::default();

    loop {
        // PER PAYLOAD
        match parser.parse(input_stream.buffer(), eof)? {
            Chunk::Parsed { consumed, payload } => {
                match validator.payload(&payload)? {
                    ValidPayload::Func(f, body) => allocs = handle(f, body, allocs)?,
                    ValidPayload::Parser(_) => unimplemented!("component model"),
                    ValidPayload::End(_) => break,
                    _ => { dbg!("-------------"); },
                }

                // information to produce as stream comes in
                match payload {
                    // Payload::Version
                    Payload::TypeSection(reader) => {
                        dbg!("TYPE");
                        for ft in reader.into_iter_err_on_gc_types().flatten() {
                            ft.codegen(output_stream)?;
                        }
                    }
                    Payload::ImportSection(_) => { 
                        dbg!("IMPORT");
                    }
                    Payload::FunctionSection(_) => { dbg!("FUNCTION"); }
                    Payload::TableSection(_) => { dbg!("TABLE"); }
                    Payload::MemorySection(_) => { dbg!("MEMORY"); }
                    Payload::TagSection(_) => { dbg!("TAG"); }
                    Payload::GlobalSection(_) => { dbg!("GLOBAL"); }
                    Payload::ExportSection(_) => { dbg!("EXPORT"); }
                    Payload::StartSection { .. } => { dbg!("START"); }
                    Payload::ElementSection(_) => { dbg!("ELEMENT"); }
                    Payload::DataCountSection { .. } => { dbg!("DATA COUNT"); }
                    Payload::DataSection(_) => { dbg!("DATA"); }
                    Payload::CodeSectionStart { .. } => { dbg!("CODE SECTION START"); }
                    Payload::CodeSectionEntry(body) => {
                        dbg!("CODE SECTION ENTRY");
                    }
                    _ => { dbg!("OTHER"); } 
                }
                input_stream.consume(consumed);
            }
            Chunk::NeedMoreData(hint) => {
                if eof {
                    bail!("unexpected end");
                }
                input_stream.reserve(hint as usize);
                if input_stream.read_into_buf()? == 0 {
                    eof = true;
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // Step 1: parse input, and use Validator on every payload
    
    // PARSER stuff
    let mut input_stream = get_input_stream();
    let mut output_stream = get_output_stream();

    compile_from_stream(&mut input_stream, &mut output_stream)?;
    output_stream.flush()?;

    Ok(())
}

// function to validate(?)
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

// code generation
trait CodeGen {
    fn codegen(&self, out: &mut impl Write) -> Result<()>;
}

impl CodeGen for FuncType {
    fn codegen(&self, out: &mut impl Write) -> Result<()>{
        let params = self.params();
        let results = self.results();
        writeln!(out, "{:?} -> {:?}", params, results)?;
        Ok(())
    }
}

