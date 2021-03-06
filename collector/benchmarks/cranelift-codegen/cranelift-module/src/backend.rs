//! Defines the `Backend` trait.

use crate::DataContext;
use crate::Linkage;
use crate::ModuleNamespace;
use crate::ModuleResult;
use core::marker;
use cranelift_codegen::isa::TargetIsa;
use cranelift_codegen::Context;
use cranelift_codegen::{binemit, ir};

/// A `Backend` implements the functionality needed to support a `Module`.
///
/// Two notable implementations of this trait are:
///  - `SimpleJITBackend`, defined in [cranelift-simplejit], which JITs
///    the contents of a `Module` to memory which can be directly executed.
///  - `FaerieBackend`, defined in [cranelift-faerie], which writes the
///    contents of a `Module` out as a native object file.
///
/// [cranelift-simplejit]: https://docs.rs/cranelift-simplejit/
/// [cranelift-faerie]: https://docs.rs/cranelift-faerie/
pub trait Backend
where
    Self: marker::Sized,
{
    /// A builder for constructing `Backend` instances.
    type Builder;

    /// The results of compiling a function.
    type CompiledFunction;

    /// The results of "compiling" a data object.
    type CompiledData;

    /// The completed output artifact for a function, if this is meaningful for
    /// the `Backend`.
    type FinalizedFunction;

    /// The completed output artifact for a data object, if this is meaningful for
    /// the `Backend`.
    type FinalizedData;

    /// This is an object returned by `Module`'s
    /// [`finish`](struct.Module.html#method.finish) function,
    /// if the `Backend` has a purpose for this.
    type Product;

    /// Create a new `Backend` instance.
    fn new(_: Self::Builder) -> Self;

    /// Return the `TargetIsa` to compile for.
    fn isa(&self) -> &TargetIsa;

    /// Declare a function.
    fn declare_function(&mut self, name: &str, linkage: Linkage);

    /// Declare a data object.
    fn declare_data(&mut self, name: &str, linkage: Linkage, writable: bool);

    /// Define a function, producing the function body from the given `Context`.
    ///
    /// Functions must be declared before being defined.
    fn define_function(
        &mut self,
        name: &str,
        ctx: &Context,
        namespace: &ModuleNamespace<Self>,
        code_size: u32,
    ) -> ModuleResult<Self::CompiledFunction>;

    /// Define a zero-initialized data object of the given size.
    ///
    /// Data objects must be declared before being defined.
    fn define_data(
        &mut self,
        name: &str,
        writable: bool,
        data_ctx: &DataContext,
        namespace: &ModuleNamespace<Self>,
    ) -> ModuleResult<Self::CompiledData>;

    /// Write the address of `what` into the data for `data` at `offset`. `data` must refer to a
    /// defined data object.
    fn write_data_funcaddr(
        &mut self,
        data: &mut Self::CompiledData,
        offset: usize,
        what: ir::FuncRef,
    );

    /// Write the address of `what` plus `addend` into the data for `data` at `offset`. `data` must
    /// refer to a defined data object.
    fn write_data_dataaddr(
        &mut self,
        data: &mut Self::CompiledData,
        offset: usize,
        what: ir::GlobalValue,
        addend: binemit::Addend,
    );

    /// Perform all outstanding relocations on the given function. This requires all `Local`
    /// and `Export` entities referenced to be defined.
    fn finalize_function(
        &mut self,
        func: &Self::CompiledFunction,
        namespace: &ModuleNamespace<Self>,
    ) -> Self::FinalizedFunction;

    /// Return the finalized artifact from the backend, if relevant.
    fn get_finalized_function(&self, func: &Self::CompiledFunction) -> Self::FinalizedFunction;

    /// Perform all outstanding relocations on the given data object. This requires all
    /// `Local` and `Export` entities referenced to be defined.
    fn finalize_data(
        &mut self,
        data: &Self::CompiledData,
        namespace: &ModuleNamespace<Self>,
    ) -> Self::FinalizedData;

    /// Return the finalized artifact from the backend, if relevant.
    fn get_finalized_data(&self, data: &Self::CompiledData) -> Self::FinalizedData;

    /// "Publish" all finalized functions and data objects to their ultimate destinations.
    fn publish(&mut self);

    /// Consume this `Backend` and return a result. Some implementations may
    /// provide additional functionality through this result.
    fn finish(self) -> Self::Product;
}
