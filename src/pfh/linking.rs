//! Linking an external crate and sharing data.
//! 
//! When running a repl you might want to link an external crate. The specific use case is a developer wants to link the crate they are working on into the repl for the user to be able to use. A developer might also want to make data available to the repl. Papyrus has this functionality but makes some assumptions that the developer will need to be aware of, detailed below. When linking is desired, there are two main aspects to consider, the crate name to link and the data transferrence scheme (`Borrow` or `BorrowMut`).
//! 
//! The most simple linking is to make available an external crate (not through [`crates.io`](https://www.crates.io/)). There is no data transferrence, and as such simple chained functions on the `ReplData` structure can achieve this. There are two macros for the more complex data transferrence, `repl_data_brw!` and `repl_data_brw_mut!`.
//! 
//! There are four variations when using each macro, each applies to `repl_data_brw_mut!` as well.
//! 
//! ```ignore
//! repl_data_brw!(crate_name, type);
//! repl_data_brw!((crate_name, rlib_path), type);
//! repl_data_brw!(compilation_dir, crate_name, type);
//! repl_data_brw!(compilation_dir, (crate_name, rlib_path), type);
//! ```
//! 
//! The `crate_name` is the string of the external crate to link, the optional addition of `rlib_path` forces Papyrus to copy the rlib from that location instead of searching for it. The `type` parameter is the type ascription, ie `String`, `MyStruct`, etc. Finally the `compilation_dir` defines a directory to use rather than the default.
//! 
//! # Worked Example - No Data
//! 
//! Let's work on a crate called `some-lib`.
//! 
//! ## File Setup
//! 
//! ***main.rs***:
//! 
//! ```ignore
//! use papyrus::{Repl, ReplData};
//! 
//! fn main() {
//!   let mut data = ReplData::default()
//!     .with_extern_crate("some_lib", None)
//!     .expect("failed linking crate")
//! 	.no_extern_data();
//!   let repl = Repl::default_terminal(&mut data);
//! 
//!   repl.run();
//! }
//! ```
//! 
//! ***lib.rs***:
//! 
//! ```rust
//! pub struct MyStruct {
//!   a: i32,
//!   b: i32,
//! }
//! 
//! impl MyStruct {
//!   pub fn new(a: i32, b: i32) -> Self {
//!   MyStruct { a, b }
//!   }
//! 
//!   pub fn add_contents(&self) -> i32 {
//!     self.a + self.b
//!   }
//! }
//! ```
//! 
//! ***Cargo.toml***:
//! 
//! ```toml
//! [package]
//! name = "some-lib"
//! 
//! ...
//! 
//! [lib]
//! name = "some_lib"
//! crate-type = ["rlib", "staticlib"]
//! path = "src/lib.rs" # you may need path to the library
//! 
//! [dependencies]
//! papyrus = "*"
//! ...
//! ```
//! 
//! Notice that you will have to specify the library with a certain `crate-type`. Papyrus links using an `rlib` file, but I have shown that you can also build multiple library files.
//! 
//! If you build this project you should find a `libsome_lib.rlib` sitting in your build directory. Papyrus uses this to link when compiling.
//! 
//! ### Repl
//! 
//! Run this project (`cargo run`). It should spool up fine and prompt you with `papyrus=>`. Now you can try to use the linked crate.
//! 
//! ```sh
//! papyrus=> some_lib::MyStruct::new(20, 30).add_contents()
//! papyrus [out0]: 50
//! ```
//! 
//! # What's going on
//! 
//! - Papyrus takes the crate name you specify and will add this as `extern crate CRATE_NAME;` to the source file.
//! - When setting the external crate name, the `rlib` library is found and copied into the compilation directory.
//!   - Papyrus uses `std::env::current_exe()` to find the executing folder, and searches for the `rlib` file in that folder (`libCRATE_NAME.rlib`)
//!   - Specify the path to the `rlib` library if it is located in a different folder
//! - When compiling the repl code, a rust flag is set, linking the `rlib` such that `extern crate CRATE_NAME;` works.
//! 
//! # Worked Example - Borrowed Data
//! 
//! Keep the example before, but alter the `main.rs` file.
//! 
//! ***main.rs***:
//! 
//! ```ignore
//! extern crate somelib;
//! 
//! use somelib::MyStruct;
//! use papyrus::{Repl, ReplData};
//! 
//! fn main() {
//!   let my_struct = MyStruct::new(20,30);
//! 
//!   let mut data = repl_data_brw!("somelib", MyStruct).expect("failed creating data");
//!   let repl = Repl::default_terminal(&mut data);
//! 
//!   repl.run();
//! }
//! ```
//! 
//! Run this project (`cargo run`). It should spool up fine and prompt you with `papyrus=>`. Now you can try to use the linked data. The linked data is in a variable `app_dat` and depending of the variant it will be `&` or `&mut`.
//! 
//! ```sh
//! papyrus=> app_data.add_contents()
//! papyrus [out0]: 50
//! ```
//! 
//! This will work with `BorrowMut` variants as well.
//! 
//! # Notes
//! 
//! ## Panics
//! 
//! To avoid crashing the application on a panic, `catch_unwind` is employed. This function requires data that crosses the boundary be `UnwindSafe`, making `&` and `&mut` not valid data types. Papyrus uses `AssertUnwindSafe` wrappers to make this work, however it makes `app_data` vunerable to breaking invariant states if a panic is triggered. In practice the repl is designed to be low imapct and such should not have many cases where broken invariants are caused, however there is no garauntee.

use std::path::{Path, PathBuf};
use std::{fs, io};

mod macros {
	/// Create `ReplData` with expectation to borrow the data type `type`.
	#[macro_export]
	macro_rules! repl_data_brw {
		// (crate_name, type)
		($crate_name:expr, $type:ty) => {{
			use papyrus;
			let crate_name: &'static str = $crate_name;
			let repl_data_res: std::io::Result<
				papyrus::ReplData<_, papyrus::linking::BorrowData, $type>,
			> = papyrus::ReplData::default().with_extern_crate_and_data(
				crate_name,
				None,
				stringify!($type),
				);
			repl_data_res
			}};
		// ((crate_name, rlib_path), type)
		($crate_and_rlib:expr, $type:ty) => {{
			use papyrus;
			let (crate_name, rlib_path): (&'static str, &str) = $crate_and_rlib;
			let repl_data_res: std::io::Result<
				papyrus::ReplData<_, papyrus::linking::BorrowData, $type>,
			> = papyrus::ReplData::default().with_extern_crate_and_data(
				crate_name,
				Some(rlib_path),
				stringify!($type),
				);
			repl_data_res
			}};
		// (compilation_dir, crate_name, type)
		($comp_dir:expr, $crate_name:expr, $type:ty) => {{
			use papyrus;
			let compilation_dir: &str = $comp_dir;
			let crate_name: &'static str = $crate_name;
			let repl_data_res: std::io::Result<
				papyrus::ReplData<_, papyrus::linking::BorrowData, $type>,
			> = papyrus::ReplData::default().with_compilation_dir(compilation_dir);
			match repl_data_res {
				Ok(r) => r.with_extern_crate_and_data(crate_name, None, stringify!($type)),
				Err(e) => Err(e),
				}
			}};
		// (compilation_dir, (crate_name, rlb_path), type)
		($comp_dir:expr, $crate_and_rlib:expr, $type:ty) => {{
			use papyrus;
			let compilation_dir: &str = $comp_dir;
			let (crate_name, rlib_path): (&'static str, &str) = $crate_and_rlib;
			let repl_data_res: std::io::Result<
				papyrus::ReplData<_, papyrus::linking::BorrowData, $type>,
			> = papyrus::ReplData::default().with_compilation_dir(compilation_dir);
			match repl_data_res {
				Ok(r) => {
					r.with_extern_crate_and_data(crate_name, Some(rlib_path), stringify!($type))
					}
				Err(e) => Err(e),
				}
			}};
	}

	/// Create `ReplData` with expectation to mutably borrow the data type `type`.
	#[macro_export]
	macro_rules! repl_data_brw_mut {
		// (crate_name, type)
		($crate_name:expr, $type:ty) => {{
			use papyrus;
			let crate_name: &'static str = $crate_name;
			let repl_data_res: std::io::Result<
				papyrus::ReplData<_, papyrus::linking::BorrowMutData, $type>,
			> = papyrus::ReplData::default().with_extern_crate_and_data(
				crate_name,
				None,
				stringify!($type),
				);
			repl_data_res
			}};
		// ((crate_name, rlib_path), type)
		($crate_and_rlib:expr, $type:ty) => {{
			use papyrus;
			let (crate_name, rlib_path): (&'static str, &str) = $crate_and_rlib;
			let repl_data_res: std::io::Result<
				papyrus::ReplData<_, papyrus::linking::BorrowMutData, $type>,
			> = papyrus::ReplData::default().with_extern_crate_and_data(
				crate_name,
				Some(rlib_path),
				stringify!($type),
				);
			repl_data_res
			}};
		// (compilation_dir, crate_name, type)
		($comp_dir:expr, $crate_name:expr, $type:ty) => {{
			use papyrus;
			let compilation_dir: &str = $comp_dir;
			let crate_name: &'static str = $crate_name;
			let repl_data_res: std::io::Result<
				papyrus::ReplData<_, papyrus::linking::BorrowMutData, $type>,
			> = papyrus::ReplData::default().with_compilation_dir(compilation_dir);
			match repl_data_res {
				Ok(r) => r.with_extern_crate_and_data(crate_name, None, stringify!($type)),
				Err(e) => Err(e),
				}
			}};
		// (compilation_dir, (crate_name, rlb_path), type)
		($comp_dir:expr, $crate_and_rlib:expr, $type:ty) => {{
			use papyrus;
			let compilation_dir: &str = $comp_dir;
			let (crate_name, rlib_path): (&'static str, &str) = $crate_and_rlib;
			let repl_data_res: std::io::Result<
				papyrus::ReplData<_, papyrus::linking::BorrowMutData, $type>,
			> = papyrus::ReplData::default().with_compilation_dir(compilation_dir);
			match repl_data_res {
				Ok(r) => {
					r.with_extern_crate_and_data(crate_name, Some(rlib_path), stringify!($type))
					}
				Err(e) => Err(e),
				}
			}};
	}
}

pub struct LinkingConfiguration {
	/// The name of the external crate.
	/// Needs to match what is compiled.
	/// Example: `some_lib`
	/// - will search for `libsome_lib.rlib` in filesystem
	/// - will add `extern crate some_lib;` to source file
	/// - will compile with `--extern some_lib=libsome_lib.rlib` flag
	pub crate_name: &'static str,
	/// Linking data configuration.
	/// If the user wants to transfer data from the calling application then it can specify the type of data as a string.
	/// The string must include module path if not accesible from the root of the external crate.
	/// The `ArgumentType` parameter flags how to pass the data to the function.
	///
	/// Example: `MyStruct` under the module `some_mod` in crate `some_lib` with `ArgumentType::Borrow`
	/// - will add `some_lib::some_mod::MyStruct` to the function argument
	/// - function looks like `fn(app_data: &some_lib::some_mode::MyStruct)`
	data_type: Option<String>,
}

impl LinkingConfiguration {
	pub fn link_external_crate<P: AsRef<Path>>(
		compilation_dir: P,
		crate_name: &'static str,
		rlib_path: Option<&str>,
	) -> io::Result<Self> {
		let rlib_path = match rlib_path {
			Some(p) => PathBuf::from(p),
			None => get_rlib_path(crate_name)?,
		};

		dbg!(&rlib_path);

		fs::copy(
			rlib_path,
			compilation_dir
				.as_ref()
				.join(&format!("lib{}.rlib", crate_name)),
		)?;

		Ok(LinkingConfiguration {
			crate_name: crate_name,
			data_type: None,
		})
	}

	pub fn with_data(mut self, type_name: &str) -> Self {
		self.data_type = Some(type_name.to_string());
		self
	}

	pub fn construct_fn_args(&self, arg_type: &LinkingArgument) -> String {
		match self.data_type {
			Some(ref d) => match arg_type {
				LinkingArgument::BorrowData => format!("app_data: &{}::{}", self.crate_name, d),
				LinkingArgument::BorrowMutData => {
					format!("app_data: &mut {}::{}", self.crate_name, d)
				}
				LinkingArgument::NoData => String::new(),
			},
			None => String::new(),
		}
	}
}

pub struct NoData;
pub struct BorrowData;
pub struct BorrowMutData;

pub enum LinkingArgument {
	NoData,
	BorrowData,
	BorrowMutData,
}

fn get_rlib_path(crate_name: &str) -> io::Result<PathBuf> {
	let lib_name = format!("lib{}.rlib", crate_name);
	let exe = std::env::current_exe()?;
	dbg!(&exe);
	fs::read_dir(exe.parent().expect("files should always have a parent"))?
		.into_iter()
		.filter(|entry| entry.is_ok())
		.map(|entry| entry.expect("filtered some").path())
		.find(|path| path.ends_with(&lib_name))
		.ok_or(io::Error::new(
			io::ErrorKind::NotFound,
			format!("did not find file: '{}'", lib_name),
		))
}
