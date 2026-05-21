use std::marker::PhantomData;

use crate::{
    context::{Context, Ptr},
    irbuild::{inserter::OpInsertionPoint, listener::RewriteListener, rewriter::Rewriter},
    operation::Operation,
    value::Value,
};

/// Object-safe rewriter facade used by `ConstFoldInterface::fold_in_place`.
/// A blanket impl forwards to any `Rewriter<L>`, so callers can pass an
/// `IRRewriter<DummyListener>`, `IRRewriter<MyPassListener>`, etc.
pub trait FoldRewriter {
    /// Appends an [Operation] at the current insertion point.
    /// The insertion point is updated to be after this newly inserted [Operation].
    fn append_operation(&mut self, ctx: &Context, operation: Ptr<Operation>);

    /// Inserts an [Operation] at the current insertion point.
    /// To insert a sequence in-order, use [append_operation](Self::append_operation).
    fn insert_operation(&mut self, ctx: &Context, operation: Ptr<Operation>);

    /// Set the insertion point.
    fn set_insertion_point(&mut self, point: OpInsertionPoint);

    /// Gets the current insertion point.
    fn get_insertion_point(&self) -> OpInsertionPoint;

    /// Replace an [Operation] (and delete it) with another operation.
    /// Results of the new operation must match the results of the old operation.
    fn replace_operation(&mut self, ctx: &mut Context, op: Ptr<Operation>, new_op: Ptr<Operation>);

    /// Replace an [Operation] (and delete it) with a list of values.
    /// Results of the new operation must match the list of values.
    fn replace_operation_with_values(
        &mut self,
        ctx: &mut Context,
        op: Ptr<Operation>,
        new_values: Vec<Value>,
    );

    /// Replace all uses of a [Value] with another value.
    fn replace_value_uses_with(&mut self, ctx: &Context, old_value: Value, new_value: Value);

    /// Erase an [Operation]. The operation must have no uses.
    fn erase_operation(&mut self, ctx: &mut Context, op: Ptr<Operation>);
}

/// Adapter that exposes any [`Rewriter<L>`] as a [`FoldRewriter`].
pub struct FoldRewriterAdapter<'a, L: RewriteListener, R: Rewriter<L>> {
    rewriter: &'a mut R,
    _phantom: PhantomData<L>,
}

impl<'a, L: RewriteListener, R: Rewriter<L>> FoldRewriterAdapter<'a, L, R> {
    pub fn new(rewriter: &'a mut R) -> Self {
        Self {
            rewriter,
            _phantom: PhantomData,
        }
    }
}

impl<'a, L: RewriteListener, R: Rewriter<L>> FoldRewriter for FoldRewriterAdapter<'a, L, R> {
    fn append_operation(&mut self, ctx: &Context, operation: Ptr<Operation>) {
        self.rewriter.append_operation(ctx, operation)
    }

    fn insert_operation(&mut self, ctx: &Context, operation: Ptr<Operation>) {
        self.rewriter.insert_operation(ctx, operation)
    }

    fn set_insertion_point(&mut self, point: OpInsertionPoint) {
        self.rewriter.set_insertion_point(point)
    }

    fn get_insertion_point(&self) -> OpInsertionPoint {
        self.rewriter.get_insertion_point()
    }

    fn replace_operation(&mut self, ctx: &mut Context, op: Ptr<Operation>, new_op: Ptr<Operation>) {
        self.rewriter.replace_operation(ctx, op, new_op)
    }

    fn replace_operation_with_values(
        &mut self,
        ctx: &mut Context,
        op: Ptr<Operation>,
        new_values: Vec<Value>,
    ) {
        self.rewriter
            .replace_operation_with_values(ctx, op, new_values)
    }

    fn replace_value_uses_with(&mut self, ctx: &Context, old_value: Value, new_value: Value) {
        self.rewriter
            .replace_value_uses_with(ctx, old_value, new_value)
    }

    fn erase_operation(&mut self, ctx: &mut Context, op: Ptr<Operation>) {
        self.rewriter.erase_operation(ctx, op)
    }
}
