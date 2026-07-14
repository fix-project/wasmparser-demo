use core::alloc;
use std::io;
use anyhow::Result;
use wasmparser::{Chunk, FuncValidatorAllocations, Parser, Payload::*, Validator};

fn main() -> Result<()> {
    parse(io::stdin())?;
    Ok(())
}

fn parse(mut reader: impl io::Read) -> Result<()> {
    let mut buf = Vec::new();
    let mut cur = Parser::new(0);
    let mut eof = false;
    let mut stack = Vec::new();

    let mut validator = Validator::new();

    loop {
        let (payload, consumed) = match cur.parse(&buf, eof)? {
            Chunk::NeedMoreData(hint) => {
                assert!(!eof); // otherwise an error would be returned

                // Use the hint to preallocate more space, then read
                // some more data into our buffer.
                //
                // Note that the buffer management here is not ideal,
                // but it's compact enough to fit in an example!
                let len = buf.len();
                buf.extend((0..hint).map(|_| 0u8));
                let n = reader.read(&mut buf[len..])?;
                buf.truncate(len + n);
                eof = n == 0;
                continue;
            }

            Chunk::Parsed { consumed, payload } => (payload, consumed),
        };
        
        match payload {
            // Sections for WebAssembly modules
            Version { num, encoding, range } => {
                validator.version(num, encoding, &range)?;
            }
            TypeSection(body) => {
                validator.type_section(&body)?;
            }
            ImportSection(body) => {
                validator.import_section(&body)?;
            }
            FunctionSection(body) => {
                validator.function_section(&body)?;
            }
            TableSection(_) => { /* ... */ }
            MemorySection(_) => { /* ... */ }
            TagSection(_) => { /* ... */ }
            GlobalSection(_) => { /* ... */ }
            ExportSection(body) => {
                validator.export_section(&body)?;
            }
            StartSection { func, range } => {
                validator.start_section(func, &range)?;
            }
            ElementSection(_) => { /* ... */ }
            DataCountSection { .. } => { /* ... */ }
            DataSection(_) => { /* ... */ }

            // Here we know how many functions we'll be receiving as
            // `CodeSectionEntry`, so we can prepare for that, and
            // afterwards we can parse and handle each function
            // individually.
            CodeSectionStart { count, range, size } => {
                validator.code_section_start(&range)?;
            }
            CodeSectionEntry(body) => {
                dbg!(&body);
                let to_validate = dbg!(validator.code_section_entry(&body)?);
                let allocations = FuncValidatorAllocations::default();
                let mut validator = to_validate.into_validator(allocations);
                
                // TODO: FuncValidator, FuncToValidator(?), difference
                // let mut locals = Vec::new();
                for entry in body.get_locals_reader()?.into_iter() {
                    println!("local: {:?}", entry?);
                }

                // here we can iterate over `body` to parse the function
                // and its locals
                for op in body.get_operators_reader()?.into_iter() {
                    println!("op: {:?}", op?);
                }
            }

            // // Sections for WebAssembly components
            // InstanceSection(_) => { /* ... */ }
            // CoreTypeSection(_) => { /* ... */ }
            // ComponentInstanceSection(_) => { /* ... */ }
            // ComponentAliasSection(_) => { /* ... */ }
            // ComponentTypeSection(_) => { /* ... */ }
            // ComponentCanonicalSection(_) => { /* ... */ }
            // ComponentStartSection { .. } => { /* ... */ }
            // ComponentImportSection(_) => { /* ... */ }
            // ComponentExportSection(_) => { /* ... */ }
            //
            // ModuleSection { parser, .. }
            // | ComponentSection { parser, .. } => {
            //     stack.push(cur.clone());
            //     cur = parser.clone();
            // }
            //
            // CustomSection(_) => { /* ... */ }

            // Once we've reached the end of a parser we either resume
            // at the parent parser or we break out of the loop because
            // we're done.
            End(offset) => {
                validator.end(offset)?;
                if let Some(parent_parser) = stack.pop() {
                    cur = parent_parser;
                } else {
                    break;
                }
            }

            // most likely you'd return an error here
            unhandled => { dbg!(unhandled); }
        }

        // once we're done processing the payload we can forget the
        // original.
        buf.drain(..consumed);
    }

    Ok(())
}
