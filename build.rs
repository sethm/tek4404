use std::path::Path;

fn main() {
    cc::Build::new()
        .file("Musashi/softfloat/softfloat.c")
        .file("Musashi/m68kops.c")
        .file("Musashi/m68kcpu.c")
        .file("Musashi/m68kfpu.c")
        .file("Musashi/m68kdasm.c")
        .include(Path::new("Musashi"))
        .include(Path::new("Musashi/softfloat"))
        .compile("musashi");
}
