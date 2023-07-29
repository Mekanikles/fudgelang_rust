use crate::asg;
use crate::vm;

fn create_constdata_utf8_static_string(
    builder: &mut vm::ProgramBuilder,
    str: &str,
) -> vm::ConstDataHandle {
    let block = builder.alloc_constdata(str.len() + 8);
    let data = builder.edit_constdata(&block);

    let str = str.as_bytes();

    //  String is [bytelen: u64, data: &[u8]]
    data[0..8].copy_from_slice(&str.len().to_be_bytes());
    data[8..].copy_from_slice(str);

    block
}

pub fn generate_program(_asg: &asg::Asg) -> vm::Program {
    // TODO: Hard-code generation for now!
    let mut builder = vm::ProgramBuilder::new();

    // Create const data
    let fmtstr = create_constdata_utf8_static_string(&mut builder, "Value of x: {}\n");

    // Store typed values on stack
    {
        //  Typed value is [typeid: u64, data: u64]

        // u32 literal "5"
        builder.load_stack_address(0, 0);
        builder.store_u64(0, crate::typesystem::PrimitiveType::U32 as u64);
        builder.load_stack_address(0, 8);
        builder.store_u64(0, 5u32 as u64);
    }

    // Prepare call #print_format(static_string, argcount, typed_value...)
    {
        // Load format string address into register 0
        builder.load_const_address(0, fmtstr.0);

        // Load arg count, located at stack address 0, into register 1
        builder.load_u8(1, 1);

        // Load typed value 5 address, located at stack address 0, into register 2
        builder.load_stack_address(2, 0);
    }

    // Issue call
    builder.call_builtin(crate::typesystem::BuiltInFunction::PrintFormat);

    builder.finish()
}
