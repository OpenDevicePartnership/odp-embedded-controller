use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Put the linker script somewhere the linker can find it.
    fs::write(out_dir.join("memory.x"), include_bytes!("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");

    println!("cargo:rustc-env=RISCV_RT_BASE_ISA=rv32i");
    println!("cargo:rerun-if-env-changed=RISCV_RT_BASE_ISA");

    // qemu-ec.sh looks for this symbol to initialize GPIO2, which this feature's firmware requires.
    if env::var_os("CARGO_FEATURE_TIME_ALARM_POWER_INPUT").is_some() {
        println!("cargo:rustc-link-arg-bins=--defsym=__odp_qemu_ec_power_input=1");
    }
}
