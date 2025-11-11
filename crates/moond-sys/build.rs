use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let m = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    match std::process::Command::new("sh")
        .arg("./sync.sh")
        .current_dir(&m)
        .spawn()
        .and_then(|mut s| s.wait())
    {
        Ok(a) => {
            if !a.success() {
                eprintln!("sync failed: {a}")
            }else{
                println!("cargo::rerun-if-changed={m}/../..")
            }
        }
        Err(e) => {
            eprintln!("sync failed: {e}")
        }
    }
    let includes = ["cc-copy/src", "cc-copy/include"].map(|a| format!("{m}/{a}"));
    cc::Build::new()
        .includes(&includes)
        .files(["core.c"].into_iter().map(|a| format!("cc-copy/src/{a}")))
        .compile("moond");
    let mut b = bindgen::Builder::default();
    for h in ["core.h"] {
        b = b.header(format!("{m}/cc-copy/include/{h}"));
    }
    b.layout_tests(false)
        .derive_default(true)
        .clang_args(includes.iter().flat_map(|path| ["-I", &path]))
        .generate()
        .unwrap()
        .write_to_file(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .unwrap();
    Ok(())
}
