use super::{Cpu,
            CPU_ENDIAN_BIG,
            CpuReg,
            CpuOpt,
            Error,
            ExitReason,
            System};

#[derive(PartialEq)]
pub enum Arch {
	R2000,
}

pub const REG_AT: CpuReg = CpuReg::CpuSpecific(1);

#[allow(dead_code)]
enum HookRange {
	All,
}

#[allow(dead_code)]
struct BlockHook {
	range: HookRange,
	hook: Box<Fn(u64, u64)>,
}

#[allow(dead_code)]
struct CodeHook {
	base: u64,
	hook: Box<Fn(u64, u64)>,
}

#[derive(Default)]
#[allow(dead_code)]
struct SimpleMips32InterpreterCore {
	gprs: [u32; 32],
	pc: u32,
	be: bool,
	block_hooks: Vec<BlockHook>,
	code_hooks: Vec<CodeHook>,
}

impl SimpleMips32InterpreterCore {
	fn new() -> SimpleMips32InterpreterCore {
		Default::default()
	}
}

impl Cpu for SimpleMips32InterpreterCore {
	fn execute_range(&mut self, base: u64, end: u64) -> Result<ExitReason, Error> {
		while ((self.pc as u64) >= base) && ((self.pc as u64) <= end) {
			for block_hook in self.block_hooks.iter() {
				(&block_hook.hook)(self.pc as u64, 4);
			}
			for code_hook in self.code_hooks.iter() {
				(&code_hook.hook)(self.pc as u64, 4);
			}
			self.gprs[1] |= 0x3456;
			self.pc += 4;
		}

		Ok(ExitReason::PcOutOfRange(self.pc as u64))
	}

	fn get_reg(&self, reg: CpuReg) -> Result<u64, Error> {
		match reg {
			CpuReg::CpuSpecific(r) if r <= 31 => {
				Ok(self.gprs[r as usize] as u64)
			},
			CpuReg::Pc => {
				Ok(self.pc as u64)
			},
			_ => {
				Err(Error::GetRegUnknownReg(reg))
			},
		}
	}

	fn set_reg(&mut self, reg: CpuReg, value: u64) -> Result<(), Error> {
		if value > 0x00000000FFFFFFFF {
			return Err(Error::SetRegValueOutOfRange(reg, value));
		}

		let value32 = value as u32;

		match reg {
			CpuReg::CpuSpecific(r) if r <= 31 => {
				self.gprs[r as usize] = value32;
			},
			CpuReg::Pc => {
				self.pc = value32;
			},
			_ => {
				return Err(Error::SetRegUnknownReg(reg, value));
			},
		}

		Ok(())
	}

	fn add_block_hook_all(&mut self, hook: Box<Fn(u64, u64)>) -> Result<(), Error> {
		self.block_hooks.push( BlockHook {
			range: HookRange::All,
			hook: hook,
		});

		Ok(())
	}

	fn add_code_hook_single(&mut self, base: u64, hook: Box<Fn(u64, u64)>) -> Result<(), Error> {
		self.code_hooks.push( CodeHook {
			base: base,
			hook: hook,
		});

		Ok(())
	}
}

#[allow(unused_variables)]
pub fn build_cpu(system: &mut System, opts: CpuOpt, arch: Arch) -> Result<Box<Cpu>, Error> {
	let is_big: bool = (opts & CPU_ENDIAN_BIG) == CPU_ENDIAN_BIG;

	Ok(match arch {
		Arch::R2000 => {
			Box::new(
				SimpleMips32InterpreterCore {
					be: is_big,
					.. SimpleMips32InterpreterCore::new()
				}
			)
		},
	})
}
