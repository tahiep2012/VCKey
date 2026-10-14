fn main() {
    let mut build = cc::Build::new();
    build.cpp(true)
        .file("ukengine/byteio.cpp")
        .file("ukengine/charset.cpp")
        .file("ukengine/convert.cpp")
        .file("ukengine/data.cpp")
        .file("ukengine/error.cpp")
        .file("ukengine/inputproc.cpp")
        .file("ukengine/mactab.cpp")
        .file("ukengine/pattern.cpp")
        .file("ukengine/ukengine.cpp")
        .file("ukengine/usrkeymap.cpp")
        .file("ukengine/unikey.cpp")
        .include("ukengine");

    if build.get_compiler().is_like_gnu() || build.get_compiler().is_like_clang() {
        build.flag("-funsigned-char");
        build.flag("-Wno-unused-parameter");
        build.flag("-Wno-sign-compare");
        build.flag("-Wno-invalid-source-encoding");
    }

    build.cpp_link_stdlib(Some("c++"));
    build.compile("ukengine");

    tauri_build::build();
}
