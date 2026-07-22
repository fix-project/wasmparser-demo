// CURRENTLY SUPPORTS:
// TYPES: i32, i64, f32, f64 -> uint32_t, uint64_t, float, double
// SINGLE RETURN VALUES

use anyhow::{Result, bail, anyhow};
use buffer_redux::{BufReader, BufWriter};
use std::{io::{BufRead, Read, Write, stdin, stdout}, fs::File};
use wasmparser::{
    Chunk, FuncToValidate, FuncType, FuncValidatorAllocations, FunctionBody, ModuleArity, Parser, Payload, TypeRef, ValType, ValidPayload, Validator, WasmModuleResources
};

fn get_input_stream() -> BufReader<impl Read> {
    BufReader::new_ringbuf(stdin())
}

fn get_output_stream() -> Result<BufWriter<impl Write>> {
    let f = File::create("out.h")?;
    Ok(BufWriter::new_ringbuf(f))
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
                    ValidPayload::Func(f, body) => { 
                        allocs = codegen(output_stream, f, body, allocs)?;
                        // allocs = handle(f, body, allocs)?,
                    }, 
                    ValidPayload::Parser(_) => unimplemented!("component model"),
                    ValidPayload::End(_) => break,
                    ValidPayload::Ok => { 
                        // match payload {
                        //     Payload::ImportSection(reader) => {
                        //     }
                        // }
                    },
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

fn codegen<T: WasmModuleResources>(
    out: &mut impl Write,
    f: FuncToValidate<T>,
    body: FunctionBody<'_>,
    allocs: FuncValidatorAllocations,
) -> Result<FuncValidatorAllocations> {
    let func_validator = f.into_validator(allocs);

    // get type, function id
    let func_id = func_validator.index(); // function id

    let func_type_id = func_validator.type_index_of_function(func_id).unwrap();
    let func_type = func_validator.sub_type_at(func_type_id).unwrap().unwrap_func(); // function type
    let results = func_type.results();
    let params = func_type.params();

    // type -> string
    let cc_return = 
        if func_type.results().is_empty() {
            "void"
        } else {
            cc_type(&results[0])
        };

    let mut cc_params = String::new();
    for (i, ty) in params.iter().enumerate() {
        if i > 0 {
            cc_params += ", ";
        }
        cc_params += cc_type(ty);
    }

    // <return> f0(<param>, <param>);
    writeln!(out, "{cc_return} f{func_id}({cc_params});")?;

    Ok(func_validator.into_allocations())
}

fn print_includes(out: &mut impl Write) -> Result<()> {
    let includes = ["<stdint.h>"];

    for inc in includes {
        writeln!(out, "#include {}", inc)?;
    }
    writeln!(out)?;

    Ok(())
}

fn print_typedefs(out: &mut impl Write) -> Result<()> {
    let typedefs = [
        ("uint32_t", cc_type(&ValType::I32)),
        ("uint64_t", cc_type(&ValType::I64)),
        ("float", cc_type(&ValType::F32)),
        ("double", cc_type(&ValType::F64)),
    ];

    for td in typedefs {
        writeln!(out, "typedef {} {};", td.0, td.1)?;
    }
    writeln!(out)?;

    Ok(())
}

fn print_program(
    input_stream: &mut BufReader<impl Read>,
    output_stream: &mut BufWriter<impl Write>
) -> Result<()> {
    writeln!(output_stream, "class Module {{")?;
    writeln!(output_stream, "public:")?;
    compile_from_stream(input_stream, output_stream)?;
    writeln!(output_stream, "}};")?;

    output_stream.flush()?;

    Ok(())
}

fn main() -> Result<()> {
    // Step 1: parse input, and use Validator on every payload
    
    // PARSER stuff
    let mut input_stream = get_input_stream();
    let mut output_stream = get_output_stream()?;

    // TYPEDEF stuff
    print_includes(&mut output_stream)?;
    print_typedefs(&mut output_stream)?;
    
    print_program(&mut input_stream, &mut output_stream)?;

    Ok(())
}

// function to validate(?)
// fn handle<T: WasmModuleResources>(
//     f: FuncToValidate<T>,
//     body: FunctionBody<'_>,
//     allocs: FuncValidatorAllocations,
// ) -> Result<FuncValidatorAllocations> {
//     // Step 2: go operator-by-operator and validate each function
//     let mut func_validator = f.into_validator(allocs);
//     let mut reader = body.get_binary_reader();
//     let mut operator_count = 0;
//     func_validator.read_locals(&mut reader)?;
//     while !reader.eof() {
//         print!("Before operator {operator_count}, here are the operands on the stack:");
//         for depth in 0..func_validator.operand_stack_height() {
//             print!(" {:?}", func_validator.get_operand_type(depth as usize));
//         }
//         println!();
//
//         reader.visit_operator(&mut func_validator.visitor(reader.original_position()))??;
//         operator_count += 1;
//     }
//     reader.finish_expression(&func_validator.visitor(reader.original_position()))?;
//     Ok(func_validator.into_allocations())
// }
//
// code generation

fn cc_type(ty: &ValType) -> &'static str {
    match ty {
        ValType::I32 => "i32",
        ValType::I64 => "i64",
        ValType::F32 => "f32",
        ValType::F64 => "f64",
        _ => unimplemented!(),
    }
}

