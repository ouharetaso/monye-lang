use monye_syntax::parser::{
    Program,
    Declaration,
};
use crate::instruction::{
    Instruction,
    OpCode::*
};
use crate::translate::error::{
    TranslateError,
};
use crate::translate::lir::{
    Mochi,
    Function,
    Signature,
    Reg,
};
use crate::translate::env::{
    GlobalEnv,
    LocalEnv
};
use crate::translate::typing::{
    TypeNameExt
};
use crate::translate::compiler::block::translate_block;



pub fn translate(ast: Program) -> Result<Mochi, TranslateError> {
    let mut global_env = GlobalEnv::new();

    // register functions to global environment
    for decl in &ast.0 {
        match decl {
            Declaration::FnDecl {
                name: spanned_name,
                params: spanned_params,
                ret_ty: spanned_ret_ty,
                body: _ 
            } => {
                let name = spanned_name.node().clone();
                let signature = Signature::new(
                    &spanned_params.iter().map(|(_, ty)|{
                        ty.node().clone()
                    })
                    .collect(),
                    spanned_ret_ty.node()
                );
                global_env.add_func(&name, &signature);
            },
        }
    }

    let mut functions = Vec::new();

    for decl in &ast.0 {
        match decl {
            Declaration::FnDecl{
                name: spanned_name,
                params: spanned_params,
                ret_ty: spanned_ret_ty,
                body: spanned_body 
            } => {
                let mut local_env = LocalEnv::new();
                let mut constants = Vec::new();
                let target_reg = Reg(0);
                for (param, ty) in spanned_params {
                    local_env.add_variable(param.node(), ty.node());
                }

                let (mut insn_seq, ty) = translate_block(
                    &mut global_env,
                    Some(local_env),
                    &mut constants,
                    target_reg,
                    spanned_body,
                    Some(spanned_ret_ty.node())
                )?;

                insn_seq.extend(vec![
                    Instruction(Ret, target_reg.0, 0, 0)
                ]);

                match spanned_ret_ty.node().try_cast(&ty) {
                    Ok(_) => (),
                    Err(e) => return Err(TranslateError(
                        e,
                        spanned_ret_ty.span()
                    )),
                }

                let signature = Signature::new(
                    &spanned_params.iter().map(|(_, ty)|{
                        ty.node().clone()
                    })
                    .collect(),
                    spanned_ret_ty.node()
                );
                // 無いわけない
                let (_, func_id) = global_env.get_func(spanned_name.node()).unwrap();

                let function = Function::new(
                    spanned_name.node(),
                    func_id,
                    &signature,
                    insn_seq,
                    constants
                );
                functions.push(function);
            },
        }
    }
    
    Ok(Mochi::new(functions))
}
