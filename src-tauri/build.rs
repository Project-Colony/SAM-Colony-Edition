use std::env;
use std::fmt::Write as _;
use std::path::Path;

fn main() {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match os.as_str() {
        // Windows: link steam_api64.dll via delay-load so the import is resolved
        // on first call, not at process start. bootstrap::bootstrap() runs
        // before that first call and writes the embedded DLL to a writable dir,
        // then calls SetDllDirectoryW so the loader finds it.
        "windows" => {
            println!("cargo:rustc-link-arg-bin=samira=/DELAYLOAD:steam_api64.dll");
            println!("cargo:rustc-link-arg-bin=samira=delayimp.lib");
        }
        "linux" | "macos" => {
            write_steam_stubs(&os, &arch);
            // steamworks-sys links libsteam_api, and dyld refuses to start a
            // program whose library is missing unless the link is weak. The
            // stubs leave no symbol bound to it, so nothing else changes.
            if os == "macos" {
                println!("cargo:rustc-link-arg=-Wl,-weak-lsteam_api");
            }
        }
        _ => {}
    }

    tauri_build::build()
}

/// Writes `steam_stubs.rs` for src/bootstrap.rs: `NAMES`, every function
/// libsteam_api exports, and for each a stub of the same name that jumps to
/// the address at its index in `TABLE`. Defined in SAM itself, the stubs take
/// every reference steamworks-sys makes, so the library is never needed at
/// startup (the linker drops it with `--as-needed` on Linux).
///
/// The names come from the Linux build for macOS too: both export the same
/// flat API, except SteamRealPath, which is Linux only and unused.
fn write_steam_stubs(os: &str, arch: &str) {
    let lib = "vendor/libsteam_api.so";
    println!("cargo:rerun-if-changed={lib}");
    let names = steam_functions(&std::fs::read(lib).unwrap());

    let (prefix, hidden) = match os {
        "macos" => ("_", ".private_extern"),
        _ => ("", ".hidden"),
    };
    // x16 is the scratch register the arm64 ABI reserves for such veneers.
    let jump = match (os, arch) {
        (_, "x86_64") => "jmp qword ptr [rip + {table} + OFFSET]",
        ("macos", "aarch64") => {
            "adrp x16, {table}@PAGE\nadd x16, x16, {table}@PAGEOFF\nldr x16, [x16, #OFFSET]\nbr x16"
        }
        _ => panic!(
            "SAM embeds the Steamworks library for x86_64 Linux and macOS only, not {arch} {os}"
        ),
    };

    let mut asm = String::from(".text\n.p2align 2\n");
    for (i, name) in names.iter().enumerate() {
        let jump = jump.replace("OFFSET", &(8 * i).to_string());
        writeln!(
            asm,
            ".globl {prefix}{name}\n{hidden} {prefix}{name}\n{prefix}{name}:\n{jump}"
        )
        .unwrap();
    }
    let list: String = names.iter().map(|n| format!("    c\"{n}\",\n")).collect();
    let code = format!(
        "const NAMES: [&std::ffi::CStr; {}] = [\n{list}];\n\nstd::arch::global_asm!(r#\"\n{asm}\"#, table = sym TABLE);\n",
        names.len()
    );
    std::fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("steam_stubs.rs"),
        code,
    )
    .unwrap();
}

/// The Steam* functions an x86_64 ELF shared library exports: the ones
/// steamworks-sys has bindings for.
fn steam_functions(elf: &[u8]) -> Vec<String> {
    assert_eq!(&elf[..5], b"\x7fELF\x02", "not a 64-bit ELF file");
    // Little-endian fields at their offsets in the ELF64 file header, section
    // headers and symbols.
    let int = |at: usize, len: usize| {
        let mut bytes = [0; 8];
        bytes[..len].copy_from_slice(&elf[at..at + len]);
        u64::from_le_bytes(bytes) as usize
    };
    let section = |i: usize| int(0x28, 8) + i * int(0x3a, 2);
    let dynsym = (0..int(0x3c, 2))
        .map(section)
        .find(|&s| int(s + 4, 4) == 11) // SHT_DYNSYM
        .expect("no .dynsym section");
    let strings = int(section(int(dynsym + 0x28, 4)) + 0x18, 8);
    let (start, size, entry) = (
        int(dynsym + 0x18, 8),
        int(dynsym + 0x20, 8),
        int(dynsym + 0x38, 8),
    );

    (start..start + size)
        .step_by(entry)
        // STB_GLOBAL STT_FUNC, defined in the library.
        .filter(|&sym| elf[sym + 4] == 0x12 && int(sym + 6, 2) != 0)
        .map(|sym| {
            let name = &elf[strings + int(sym, 4)..];
            String::from_utf8(name[..name.iter().position(|&b| b == 0).unwrap()].to_vec()).unwrap()
        })
        .filter(|name| name.starts_with("Steam"))
        .collect()
}
