use crate::patch_builder::create_components_patch;
use brdb::pending::BrPendingFs;
use brdb::schema::{BrdbValue, WireVariant};
use brdb::{AsBrdbValue, Brz, IntoReader};
use serde_yaml;
use std::path::PathBuf;

pub fn write_brz() -> Result<(), Box<dyn std::error::Error>> {
    let src = PathBuf::from("rom_template.brz");
    let dst = PathBuf::from("rom.brz");
    let db = Brz::open(src)?.into_reader();
    let component_schema = db.components_schema()?;

    let mut chunk_files = vec![];
    let brick_grids_folder = vec![];
    let program = parse_program()?;
    for chunk in db.brick_chunk_index(1)? {

        let (mut soa, components) = db.component_chunk(1, chunk.index)?;
        let mut component_index_map = vec![];
        for component in components.clone() {
            let component_name = String::from(component.get_name());
            if component_name == "BrickComponentData_WireGraph_Expr_Compare" {
                let segment_index = format!("{}", component.prop("InputB")?.as_brdb_wire_variant()?).parse::<i64>().unwrap();
                component_index_map.push(segment_index);
            }
        }
        let mut component_count = 0;
        for mut component in components {
            let component_name = String::from(component.get_name());
            if component_name == "BrickComponentData_WireGraphPseudo_Var" {
                let rom_segment = component_index_map[component_count];
                if rom_segment < (program.len() as i64) {
                    component.set_prop("Value", BrdbValue::WireVar(WireVariant::Int(program[rom_segment as usize])))?;
                }
                component_count += 1;
            }
            soa.unwritten_struct_data.push(Box::new(component));
        }
        chunk_files.push((
            format!("{}.mps", *chunk),
            BrPendingFs::File(Some(soa.to_bytes(&component_schema)?)),
        ));

    }

    let components_patch = create_components_patch(chunk_files, brick_grids_folder);

    let pending = db
        .to_pending()?
        .with_patch(components_patch)?;
    Brz::write_pending(dst, pending)?;
    Ok(())
}

fn parse_program() -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let f = std::fs::File::open("program.yaml")?;
    let data: serde_yaml::Value = serde_yaml::from_reader(f)?;
    let mut labels = vec![];
    let mut label_sizes = vec![];
    let mut total_opcodes = 0;
    for (key, value ) in data.as_mapping().unwrap(){
        labels.push(key.as_str().unwrap().clone());
        label_sizes.push(total_opcodes);
        total_opcodes += value.as_sequence().unwrap().len() as i64;
    }
    let mut program = vec![0; total_opcodes as usize];
    let mut opcode_index = 0;
    for label in labels.clone() {
        let program_start = data.get(label).unwrap();
        for opcode in program_start.as_sequence().unwrap().iter()
        {
            let mut opcode_split = opcode.as_str().unwrap().split(" ");
            let operation = opcode_split.next().unwrap();
            let read_register_1 = opcode_split.next().unwrap();
            let read_register_2 = opcode_split.next().unwrap();
            let write_register = opcode_split.next().unwrap();
            let constant = opcode_split.next().unwrap();
            let mut operation_code: i64 = 0;
            print!("{opcode_index}: ");
            operation_code = get_operation_code(operation, operation_code);
            operation_code |= get_register(read_register_1) << 8;
            operation_code |= get_register(read_register_2) << 12;
            operation_code |= get_register(write_register) << 16;
            operation_code |= get_constant(constant, &labels, &label_sizes) << 20;
            program[opcode_index] = operation_code;
            opcode_index += 1;
        }
    }
    Ok(program)
}

fn get_constant(constant: &str, labels: &Vec<&str>, label_sizes: &Vec<i64>) -> i64 {
    if constant.parse::<i64>().is_ok() {
        let const_int = constant.parse().unwrap();
        print!("{const_int}\n");
        const_int
    }
    else {
        let index = 0;
        for i in 0..labels.len() {
            if labels[i] == constant {
                let const_int = label_sizes[i];
                print!("{const_int}\n");
                return const_int;
            }
        }
        panic!("Label not found for constant {}", constant);
    }
}

fn get_register(register: &str) -> i64 {
    print!("{register} ");
    if register == "Zero" {
        0
    }
    else if register == "SP" {
        1
    }
    else if register == "P1" {
        2
    }
    else if register == "P2" {
        3
    }
    else if register == "Ret1" {
        4
    }
    else if register == "Ret2" {
        5
    }
    else {
        for i in 0..10 {
            if register == format!("R{}", i).as_str() {
                return 6 + i;
            }
        }
        panic!("Unknown register: {}", register);
    }
}

fn get_operation_code(operation: &str, operation_code: i64) -> i64 {
    let mut operation_code = operation_code;
    if operation == "add" {
        operation_code |= 0;
    } else if operation == "sub" {
        operation_code |= 1;
    } else if operation == "bor" {
        operation_code |= 2;
    } else if operation == "band" {
        operation_code |= 3;
    } else if operation == "shiftl" {
        operation_code |= 4;
    } else if operation == "shiftr" {
        operation_code |= 5;
    } else if operation == "mul" {
        operation_code |= 6;
    } else if operation == "div" {
        operation_code |= 7;
    } else if operation == "move" {
        operation_code |= 8;
    } else if operation == "const" {
        operation_code |= 9;
    } else if operation == "save" {
        operation_code |= 10;
    } else if operation == "load" {
        operation_code |= 11;
    } else if operation == "jump" {
        operation_code |= 12;
    } else if operation == "bnz" {
        operation_code |= 13;
    } else if operation == "cmpl" {
        operation_code |= 14;
    } else if operation == "bnot" {
        operation_code |= 15;
    } else if operation == "bxor" {
        operation_code |= 16;
    } else if operation == "time" {
        operation_code |= 17;
    } else if operation == "mod" {
        operation_code |= 18;
    } else if operation == "modf" {
        operation_code |= 19;
    } else if operation == "or" {
        operation_code |= 20;
    } else if operation == "and" {
        operation_code |= 21;
    } else if operation == "not" {
        operation_code |= 22;
    } else if operation == "xor" {
        operation_code |= 23;
    } else {
        panic!("Unknown operation: {}", operation);
    }
    print!("{operation} ");

    operation_code
}
