//! One module per tool. Each generator is a pure function: rice (and maybe the current
//! file contents) in, new file contents out. Nothing here touches the disk.

pub mod fastfetch;
pub mod komorebi;
pub mod oh_my_posh;
pub mod terminal;
pub mod yasb;
