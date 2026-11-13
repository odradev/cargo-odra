//! Reads what a built wasm file exports.

/// The entry point the Casper VM runs when a contract is installed. Odra exports it from every
/// contract's wasm.
pub const INSTALLER_ENTRY_POINT: &str = "call";

const EXPORT_SECTION: u8 = 7;
const FUNCTION_EXPORT: u8 = 0;

/// Checks if the wasm module in `wasm` exports a function called `name`.
///
/// A contract whose crate was never linked into the binary still builds: into a wasm with the
/// allocator and nothing to call. Reading the export section tells the two apart, which the
/// sources cannot be trusted to.
pub fn exports_function(wasm: &[u8], name: &str) -> Result<bool, String> {
    let mut reader = Reader(wasm);
    if reader.bytes(4)? != b"\0asm" {
        return Err("not a wasm module".to_string());
    }
    reader.bytes(4)?;
    while !reader.0.is_empty() {
        let id = reader.byte()?;
        let size = reader.u32()? as usize;
        let mut section = Reader(reader.bytes(size)?);
        if id != EXPORT_SECTION {
            continue;
        }
        for _ in 0..section.u32()? {
            let name_len = section.u32()? as usize;
            let export = section.bytes(name_len)?;
            let kind = section.byte()?;
            section.u32()?;
            if kind == FUNCTION_EXPORT && export == name.as_bytes() {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    Ok(false)
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn bytes(&mut self, count: usize) -> Result<&'a [u8], String> {
        if count > self.0.len() {
            return Err("the wasm module is truncated".to_string());
        }
        let (head, tail) = self.0.split_at(count);
        self.0 = tail;
        Ok(head)
    }

    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.bytes(1)?[0])
    }

    /// An unsigned LEB128 number of up to 32 bits.
    fn u32(&mut self) -> Result<u32, String> {
        let mut value = 0u32;
        for shift in (0..35).step_by(7) {
            let byte = self.byte()?;
            value |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err("a number in the wasm module is too long".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::exports_function;

    /// A module with one custom section and the given exports as `(name, kind)`.
    fn module(exports: &[(&str, u8)]) -> Vec<u8> {
        let mut wasm = b"\0asm\x01\0\0\0".to_vec();
        // A custom section to skip, 200 bytes long: a two byte LEB128 size.
        let mut custom = vec![1, b'x'];
        custom.resize(200, 0);
        wasm.extend([0, 0xc8, 0x01]);
        wasm.extend(custom);

        let mut section = vec![exports.len() as u8];
        for (index, (name, kind)) in exports.iter().enumerate() {
            section.push(name.len() as u8);
            section.extend(name.as_bytes());
            section.push(*kind);
            section.push(index as u8);
        }
        wasm.push(7);
        wasm.push(section.len() as u8);
        wasm.extend(section);
        wasm
    }

    #[test]
    fn finds_an_exported_function() {
        let wasm = module(&[("memory", 2), ("call", 0), ("__heap_base", 3)]);
        assert_eq!(exports_function(&wasm, "call"), Ok(true));
        assert_eq!(exports_function(&wasm, "cal"), Ok(false));
        assert_eq!(exports_function(&wasm, "memory"), Ok(false));
    }

    #[test]
    fn a_module_without_the_function_or_without_exports_has_none() {
        // What a contract built without its crate exports.
        let wasm = module(&[("memory", 2), ("migrate_events", 0), ("__data_end", 3)]);
        assert_eq!(exports_function(&wasm, "call"), Ok(false));
        // An export of another kind does not count.
        assert_eq!(exports_function(&module(&[("call", 3)]), "call"), Ok(false));
        assert_eq!(exports_function(b"\0asm\x01\0\0\0", "call"), Ok(false));
    }

    #[test]
    fn rejects_what_is_not_a_whole_wasm_module() {
        assert!(exports_function(b"not wasm at all", "call").is_err());
        let wasm = module(&[("call", 0)]);
        assert!(exports_function(&wasm[..wasm.len() - 2], "call").is_err());
    }
}
