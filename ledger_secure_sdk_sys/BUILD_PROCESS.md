# `build.rs` Process

Flow executed by `build.rs`. `main()` first emits a fixed set of
`cargo:rerun-if-*` directives that tell Cargo which inputs invalidate the
build cache, then runs 8 sequential phases on an `SDKBuilder`. Each phase
reads particular inputs and produces specific outputs — env vars exported
to rustc, files in `$OUT_DIR`, or compiled C objects.

```
 ┌─────────────────────────────────────────────────────────────────────────┐
 │ main() prelude: emit cargo:rerun-if-* directives for every tracked      │
 │ input — LEDGER_SDK_PATH, {DEV}_SDK fallbacks, HEAP_SIZE,                 │
 │ LEDGER_SDK_EXTRA_{DEFINES,CFLAGS}, CC, plus devices/, link.ld, src/c/*. │
 │ Emitted up-front so cache-invalidation fires even if a later phase      │
 │ panics. cc::Build and bindgen::CargoCallbacks add their own             │
 │ rerun-if-changed entries for the C/header files they consume.           │
 └────────────────────────────┬────────────────────────────────────────────┘
                              ▼
                            ┌─────────────────────────────┐
 INPUTS (env + filesystem)  │       SDKBuilder::new()     │   OUTPUTS
                            └──────────────┬──────────────┘
                                           │
 ┌─────────────────────────┐  ┌────────────▼────────────┐
 │ `arm-none-eabi-gcc      │─▶│ 1. gcc_toolchain()      │  self.gcc_toolchain
 │   -print-sysroot`       │  │    (fallback:           │   = <sysroot PathBuf>
 │ (or /usr/lib/arm-       │  │     /usr/lib/arm-…)     │
 │   none-eabi)            │  └────────────┬────────────┘
 └─────────────────────────┘               │
                                           │
 ┌─────────────────────────┐  ┌────────────▼────────────┐
 │ CARGO_CFG_TARGET_OS     │  │ 2. device()             │  cargo:rustc-env=
 │   ∈ {nanox, nanosplus,  │  │  ─ find DeviceSpec in   │     TARGET=<name>
 │       stax, flex,       │  │    static SPECS table   │     C_SDK_GRAPHICS=
 │       apex_p}           │  │  ─ read .defines file   │       nbgl | bagl
 │ LEDGER_SDK_PATH (or     │─▶│    (header2define)      │
 │   {DEV}_SDK fallback)   │  │  ─ read .cflags file    │  self.device fully
 │ CARGO_FEATURE_NANO_NBGL │  │    (read_lines)         │     populated:
 │ devices/<dev>/*.defines │  │  ─ derive glyph folders │     name, c_sdk path,
 │ devices/<dev>/*.cflags  │  │    from spec            │     target triple,
 │ devices/<dev>/*.ld      │  │  ─ derive arm_libs path │     defines, cflags,
 │ (all reached via the    │  │    (st33 vs st33k1)     │     glyph folders,
 │  DeviceSpec helpers)    │  │  ─ pick linker_script   │     arm_libs,
 │                         │  │    devices/<dev>/*.ld   │     linker_script
 └─────────────────────────┘  └────────────┬────────────┘
                                           │
 ┌─────────────────────────┐  ┌────────────▼────────────┐
 │ <c_sdk>/Makefile.defines│  │ 3. get_info()           │  cargo:rustc-env=
 │ <c_sdk>/Makefile.target │  │  retrieve_csdk_info():  │   API_LEVEL
 │ <c_sdk>/target/<dev>/   │─▶│   ├─ Makefile.* parse   │   TARGET_ID
 │   include/bolos_target.h│  │   ├─ bolos_target.h     │   TARGET_NAME
 │ git describe (in c_sdk) │  │   └─ git describe       │   C_SDK_NAME
 │                         │  │                         │   C_SDK_HASH
 │                         │  │                         │   C_SDK_VERSION
 └─────────────────────────┘  └────────────┬────────────┘
                                           │
 ┌─────────────────────────┐  ┌────────────▼────────────┐
 │ <c_sdk>/Makefile.conf.cx│─▶│ 4. cxdefines()          │  self.cxdefines
 │   (HAVE_* tokens)       │  │  + NATIVE_LITTLE_ENDIAN │   (Vec<String>)
 └─────────────────────────┘  └────────────┬────────────┘
                                           │
 ┌─────────────────────────┐  ┌────────────▼─────────────┐
 │ AUX_C_FILES (2 files)   │  │ 5. build_c_sdk()         │
 │ SDK_C_FILES (12 files)  │  │   cc::Build (CC=clang)   │
 │ <c_sdk>/include/…       │  │     compiler, target,    │
 │ <c_sdk>/io/…            │  │     includes, defines,   │
 │ <c_sdk>/lib_cxng/…      │  │     cflags               │
 │ <c_sdk>/lib_ux/…        │  │                          │
 │ <c_sdk>/lib_bagl/…      │  │   ┌────────────────────┐ │  libledger-secure-
 │ <c_sdk>/lib_nbgl/…      │─▶│   │ generate_glyphs(): │ │   sdk.a built into
 │ <c_sdk>/lib_stusb/…     │  │   │  if device.is_nbgl │ │   $OUT_DIR
 │ <c_sdk>/lib_blewbxx/…   │  │   │   → icon2glyph.py  │ │
 │ <c_sdk>/lib_u2f/…       │  │   │  else (BAGL)       │ │  $OUT_DIR/glyphs/
 │ ./src/c/src.c, sjlj.s   │  │   │   → icon3.py       │ │   {glyphs.c,glyphs.h}
 │ LEDGER_SDK_EXTRA_DEFINES│  │   │  → glyphs.{c,h}    │ │
 │ LEDGER_SDK_EXTRA_CFLAGS │  │   └─────────┬──────────┘ │  cargo:rustc-link-
 │ CARGO_FEATURE_DEBUG_CSDK│  │             │            │   lib=c
 │                         │  │   if HAVE_IO_USB   →     │  cargo:rustc-link-
 │                         │  │     configure_lib_usb    │   search=<arm_libs>
 │                         │  │   if HAVE_BLE      →     │
 │                         │  │     configure_lib_ble    │
 │                         │  │   if HAVE_NBGL     →     │
 │                         │  │     configure_lib_nbgl   │
 │                         │  │   if HAVE_BAGL     →     │
 │                         │  │     add glyphs.c         │
 │                         │  │   if HAVE_IO_U2F   →     │
 │                         │  │     configure_lib_u2f    │
 │                         │  └────────────┬─────────────┘
 │                         │               │
 │                         │  ┌────────────▼─────────────┐
 │                         │  │ 6. generate_bindings()   │
 │                         │  │   bindgen::builder()     │
 │                         │  │     clang_args + headers │
 │                         │  │     (libcxng.h, os.h,    │  $OUT_DIR/bindings.rs
 │                         │─▶│      syscalls.h, os_ux,  │  (CargoCallbacks adds
 │                         │  │      swap_lib_calls,     │   rerun-if-changed)
 │                         │  │      os_pki, os_hdkey,   │
 │                         │  │      + nbgl_use_case OR  │
 │                         │  │        bagl headers      │
 │                         │  │      — driven by         │
 │                         │  │      device.is_nbgl())   │
 └─────────────────────────┘  │     + cxdefines as -D    │
                              └────────────┬─────────────┘
                                           │
 ┌─────────────────────────┐  ┌────────────▼────────────┐
 │ HEAP_SIZE env var       │  │ 7. generate_heap_size() │  $OUT_DIR/heap_size.rs
 │   "8192" or             │─▶│  parse & clamp to       │   (pub const
 │   "nanosplus:8192,…"    │  │  [2048, max_per_device] │    HEAP_SIZE: usize)
 │ default = 8192          │  │                         │
 └─────────────────────────┘  └────────────┬────────────┘
                                           │
 ┌─────────────────────────┐  ┌────────────▼────────────┐
 │ devices/<dev>/<dev>_    │  │ 8. copy_linker_script() │  $OUT_DIR/<dev>_layout.ld
 │   layout.ld             │─▶│                         │  $OUT_DIR/link.ld
 │ ./link.ld               │  │                         │  cargo:rustc-link-
 └─────────────────────────┘  └────────────┬────────────┘   search=$OUT_DIR
                                           │
                                           ▼
                              if LEDGER_SDK_BUILD_TIMING is set:
                              total build.rs time printed via
                              `cargo:warning=…` (off by default)
```

## Notes

- **C compiler**: `cc::Build` is told to use `clang` (cross-compiling via the
  `--target` flag) unless `CC` is set in the environment — it is **not**
  invoking `arm-none-eabi-gcc` directly for compilation. The ARM GCC toolchain
  is used only for its sysroot headers and the prebuilt `libc.a` linked from
  `arm_libs`.
- **`#define main _start`**: silently renames the C SDK's `main` so Rust's
  `_start` (defined in `ledger_device_sdk/src/lib.rs`) becomes the real entry
  point.
- **Single source of truth for per-device config**: everything that differs
  between the five devices (target triple, env-var fallback, ARM lib subdir,
  NBGL glyph folders, `.defines`/`.cflags`/`_layout.ld` filenames) lives in
  the static `SPECS: &[DeviceSpec]` table. `device()` looks up the spec for
  the current `CARGO_CFG_TARGET_OS` and derives everything else; `Device::spec()`
  lets later phases reach the spec from a `Device` value.
- **NBGL vs BAGL is a property of `Device`**: `Device::is_nbgl()` returns
  true for Stax/Flex/ApexP, and for Nano devices when the `nano_nbgl` feature
  is on. It is the single source of truth used by `device()` (which `HAVE_*`
  defines to inject, which glyph folders to use), `generate_glyphs()` (which
  Python helper to invoke), and `generate_bindings()` (which lib_nbgl vs
  lib_bagl headers to bind against). `C_SDK_GRAPHICS=nbgl|bagl` is emitted
  once at the end of `device()`.
- **Per-feature branching in `build_c_sdk`**: the `for s in
  self.device.defines.iter()` loop conditionally pulls in USB/BLE/NBGL/U2F
  sources based on which `HAVE_*` defines the device's `.defines` file
  declared (`cxdefines` from `Makefile.conf.cx` is shared across all devices).
- **`infos.rs`** (referenced by the `cargo:rustc-env=…` lines in steps 2 and
  3) consumes `TARGET`, `API_LEVEL`, `TARGET_ID`, `TARGET_NAME`, `C_SDK_NAME`,
  `C_SDK_HASH`, `C_SDK_VERSION`, `C_SDK_GRAPHICS` at compile time and emits
  the ELF metadata sections (`ledger.target`, `ledger.api_level`, …) that
  `cargo-ledger` later reads.
- **Error handling**: `build.rs` doesn't carry a typed error type — each
  phase either succeeds, returns its payload directly, or panics with a
  message describing what went wrong (build scripts fail the build on
  panic, which is the desired behavior for misconfiguration).
