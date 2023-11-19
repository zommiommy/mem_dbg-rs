# mem_dbg

Traits and associated procedural macros to recursively compute the memory usage of a data structure or print its layout.

The standard library function [`std::mem::size_of`] returns the stack size of a type in bytes, but
it does not take into consideration heap memory. This crate provides a trait [`MemSize`] and an associated procedural macro
to that purpose. Moreover, the trait [`MemDbg`] can be used to display the recursive layout of a data structure.
