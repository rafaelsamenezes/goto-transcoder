// This module is responsible to convert between CBMC <=> ESBMC

use std::collections::HashMap;

use crate::cbmc::{CBMCFunction, CBMCParseResult};
use crate::esbmc::ESBMCParseResult;
use crate::irep::Irept;

fn fix_name(name: &str) -> String {
    match name {
        "__CPROVER__start" => String::from("__ESBMC_main"),
        _ => String::from(name),
    }
}

impl From<CBMCParseResult> for ESBMCParseResult {
    fn from(mut data: CBMCParseResult) -> Self {
        let mut adapted = ESBMCParseResult {
            reader: (data.reader),
            symbols_irep: Vec::with_capacity(data.symbols_irep.len()),
            functions_irep: Vec::with_capacity(data.functions_irep.len()),
        };

        // First, we need to walk through the symbols and map all the
        // ref-types into concrete types

        let mut type_cache: HashMap<Irept, Irept> = HashMap::new();

        for mut sym in data.symbols_irep {
            if sym.is_type && sym.stype.id == "struct" {
                let tagname = Irept::from(format!("tag-{}", &sym.base_name));
                sym.stype.fix_type(&type_cache);
                type_cache.insert(tagname, sym.stype.clone());
            }
            adapted.symbols_irep.push(Irept::from(sym));
        }

        // Lets double check for fixes
        for symbol in &mut adapted.symbols_irep {
            symbol.fix_type(&type_cache);
            assert_ne!(symbol.named_subt["type"].id, "struct_tag");
            assert_ne!(symbol.named_subt["type"].id, "c_bool");
        }

        // NOTE: ESBMC/CBMC uses the number offset of the function as the target
        //       which is fine for most cases. But CBMC for some reason likes to
        //       start from 1 and have a target number associated to the instruction.
        //       So we first parse everything and then fix the target numbers
        for mut foo in data.functions_irep {
            let mut target_revmap: HashMap<u32, u32> = HashMap::new();

            for (index, inst) in &mut foo.instructions.iter().enumerate() {
                target_revmap.insert(inst.target_number, (index) as u32);
            }

            // lets fix the targets
            for f in &mut foo.instructions {
                for t in &mut f.targets {
                    let unsigned_value: u32 = t.id.parse().unwrap();
                    let target_fixed = target_revmap.get(&unsigned_value).unwrap().to_string();
                    t.id = target_fixed;
                }
            }

            let function_name = fix_name(&foo.name);
            let mut function_irep = Irept::from(foo);
            function_irep.fix_type(&type_cache);
            adapted.functions_irep.push((function_name, function_irep));
        }

        adapted
    }
}
