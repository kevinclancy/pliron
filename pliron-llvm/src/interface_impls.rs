//! Implementation of various op interfaces for LLVM IR instructions.

use pliron::{
    attribute::AttrObj,
    basic_block::BasicBlock,
    builtin::{attributes::IntegerAttr, op_interfaces::ConstFoldInterface},
    context::{Context, Ptr},
    derive::op_interface_impl,
    irbuild::{IRStatus, fold_rewriter::FoldRewriter, inserter::OpInsertionPoint},
    op::Op,
    opts::dce::{BlockArgRemoval, SideEffects},
};

use crate::ops::{
    AShrOp, AddOp, AddressOfOp, AllocaOp, AndOp, BitcastOp, ConstantOp, ExtractElementOp,
    ExtractValueOp, FAddOp, FCmpOp, FDivOp, FMulOp, FNegOp, FPExtOp, FPToSIOp, FPToUIOp, FPTruncOp,
    FRemOp, FSubOp, FreezeOp, FuncOp, GetElementPtrOp, ICmpOp, InsertElementOp, InsertValueOp,
    IntToPtrOp, LShrOp, MulOp, OrOp, PoisonOp, PtrToIntOp, SDivOp, SExtOp, SIToFPOp, SRemOp,
    SelectOp, ShlOp, ShuffleVectorOp, SubOp, TruncOp, UDivOp, UIToFPOp, URemOp, UndefOp, XorOp,
    ZExtOp, ZeroOp,
};

// Implement [SideEffects] with `has_side_effects` returning `false`
macro_rules! impl_side_effects_false {
  ($($op:ty),+ $(,)?) => {
    $(
      #[op_interface_impl]
      impl SideEffects for $op {
        fn has_side_effects(&self, _ctx: &Context) -> bool {
          false
        }
      }
    )+
  };
}

// Pure value-producing ops with no memory/control side effects.
// We don't need to implement [SideEffects] for the other ops,
// because the assumption is that the absense of the interface
// implies the presence of side effects, which is a safe default for DCE.
impl_side_effects_false!(
    AddOp,
    SubOp,
    MulOp,
    ShlOp,
    UDivOp,
    SDivOp,
    URemOp,
    SRemOp,
    AndOp,
    OrOp,
    XorOp,
    LShrOp,
    AShrOp,
    ICmpOp,
    AllocaOp,
    BitcastOp,
    IntToPtrOp,
    PtrToIntOp,
    UndefOp,
    PoisonOp,
    FreezeOp,
    ConstantOp,
    ZeroOp,
    AddressOfOp,
    SExtOp,
    ZExtOp,
    FPExtOp,
    TruncOp,
    FPTruncOp,
    FPToSIOp,
    FPToUIOp,
    SIToFPOp,
    UIToFPOp,
    InsertValueOp,
    ExtractValueOp,
    InsertElementOp,
    ExtractElementOp,
    ShuffleVectorOp,
    SelectOp,
    FNegOp,
    FAddOp,
    FSubOp,
    FMulOp,
    FDivOp,
    FRemOp,
    FCmpOp,
    GetElementPtrOp,
);

#[op_interface_impl]
impl BlockArgRemoval for FuncOp {
    fn can_remove_block_args(&self, ctx: &Context, block: Ptr<BasicBlock>) -> bool {
        !matches!(self.get_entry_block(ctx), Some(entry) if entry == block)
    }
}

#[op_interface_impl]
impl ConstFoldInterface for ConstantOp {
    fn check_fold(
        &self,
        ctx: &Context,
        _operand_attrs: &[Option<AttrObj>],
    ) -> Vec<Option<AttrObj>> {
        vec![Some(self.get_value(ctx))]
    }

    fn fold_in_place(
        &self,
        _ctx: &mut Context,
        _operand_attrs: &[Option<AttrObj>],
        _rewriter: &mut dyn FoldRewriter,
    ) -> IRStatus {
        IRStatus::Unchanged
    }
}

/// If all elements of `operand_attrs` are `Some(x)` where x is an IntegerAttr,
/// sum the operands and return the result. Otherwise return None.
fn add_op_fold_sum(operand_attrs: &[Option<AttrObj>]) -> Option<IntegerAttr> {
    let [Some(lhs), Some(rhs)] = operand_attrs else {
        return None;
    };
    let lhs = lhs.downcast_ref::<IntegerAttr>()?;
    let rhs = rhs.downcast_ref::<IntegerAttr>()?;
    Some(IntegerAttr::new(
        lhs.get_type(),
        lhs.value().add(&rhs.value()),
    ))
}

#[op_interface_impl]
impl ConstFoldInterface for AddOp {
    fn check_fold(
        &self,
        _ctx: &Context,
        operand_attrs: &[Option<AttrObj>],
    ) -> Vec<Option<AttrObj>> {
        vec![add_op_fold_sum(operand_attrs).map(|attr| Box::new(attr) as AttrObj)]
    }

    fn fold_in_place(
        &self,
        ctx: &mut Context,
        operand_attrs: &[Option<AttrObj>],
        rewriter: &mut dyn FoldRewriter,
    ) -> IRStatus {
        let Some(sum) = add_op_fold_sum(operand_attrs) else {
            return IRStatus::Unchanged;
        };
        let new_const = ConstantOp::new(ctx, Box::new(sum));
        let old_op = self.get_operation();
        let new_op = new_const.get_operation();
        rewriter.set_insertion_point(OpInsertionPoint::BeforeOperation(old_op));
        rewriter.insert_operation(ctx, new_op);
        rewriter.replace_operation(ctx, old_op, new_op);
        IRStatus::Changed
    }
}
