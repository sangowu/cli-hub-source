# Dependency license inventory / 依赖许可清单

This inventory records the upstream license expressions for the locked Windows Rust dependency graph (including build dependencies) and npm production packages. It excludes npm development tooling and does not assert that every listed package is embedded in the application. It was generated from `cargo metadata --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc` and `package-lock.json` for CLI Hub 0.1.17.

本清单记录锁定的 Windows Rust 依赖图（含构建依赖）及 npm 生产依赖所声明的许可证；不代表每项都嵌入二进制，也不是完整法律兼容性认证。版本与依赖变化后应重新生成和核对。

The collected package license and notice texts are bundled as [dependency notices](../public/licenses/dependency-notices.txt). Font licenses are listed in [third-party notices](THIRD_PARTY_NOTICES.md). Some dependencies may also impose source-availability obligations; use the package links and exact versions when obtaining their source.

| Ecosystem | Package | Version | Declared license expression |
| --- | --- | --- | --- |
| Rust | [adler2](https://crates.io/crates/adler2/2.0.1) | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| Rust | [aho-corasick](https://crates.io/crates/aho-corasick/1.1.5) | 1.1.5 | Unlicense OR MIT |
| Rust | [alloc-no-stdlib](https://crates.io/crates/alloc-no-stdlib/3.0.0) | 3.0.0 | BSD-3-Clause |
| Rust | [alloc-stdlib](https://crates.io/crates/alloc-stdlib/0.3.0) | 0.3.0 | BSD-3-Clause |
| Rust | [anyhow](https://crates.io/crates/anyhow/1.0.104) | 1.0.104 | MIT OR Apache-2.0 |
| Rust | [arboard](https://crates.io/crates/arboard/3.6.1) | 3.6.1 | MIT OR Apache-2.0 |
| Rust | [atomic-waker](https://crates.io/crates/atomic-waker/1.1.2) | 1.1.2 | Apache-2.0 OR MIT |
| Rust | [auto-launch](https://crates.io/crates/auto-launch/0.5.0) | 0.5.0 | MIT |
| Rust | [autocfg](https://crates.io/crates/autocfg/1.5.1) | 1.5.1 | Apache-2.0 OR MIT |
| Rust | [base64](https://crates.io/crates/base64/0.22.1) | 0.22.1 | MIT OR Apache-2.0 |
| Rust | [base64](https://crates.io/crates/base64/0.23.1) | 0.23.1 | MIT OR Apache-2.0 |
| Rust | [bit-set](https://crates.io/crates/bit-set/0.8.0) | 0.8.0 | Apache-2.0 OR MIT |
| Rust | [bit-vec](https://crates.io/crates/bit-vec/0.8.0) | 0.8.0 | Apache-2.0 OR MIT |
| Rust | [bitflags](https://crates.io/crates/bitflags/1.3.2) | 1.3.2 | MIT/Apache-2.0 |
| Rust | [bitflags](https://crates.io/crates/bitflags/2.13.2) | 2.13.2 | MIT OR Apache-2.0 |
| Rust | [block-buffer](https://crates.io/crates/block-buffer/0.10.4) | 0.10.4 | MIT OR Apache-2.0 |
| Rust | [brotli](https://crates.io/crates/brotli/9.0.0) | 9.0.0 | BSD-3-Clause AND MIT |
| Rust | [brotli-decompressor](https://crates.io/crates/brotli-decompressor/6.0.1) | 6.0.1 | BSD-3-Clause/MIT |
| Rust | [bs58](https://crates.io/crates/bs58/0.5.1) | 0.5.1 | MIT/Apache-2.0 |
| Rust | [bytemuck](https://crates.io/crates/bytemuck/1.25.2) | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | [byteorder](https://crates.io/crates/byteorder/1.5.0) | 1.5.0 | Unlicense OR MIT |
| Rust | [byteorder-lite](https://crates.io/crates/byteorder-lite/0.1.0) | 0.1.0 | Unlicense OR MIT |
| Rust | [bytes](https://crates.io/crates/bytes/1.12.1) | 1.12.1 | MIT |
| Rust | [camino](https://crates.io/crates/camino/1.2.6) | 1.2.6 | MIT OR Apache-2.0 |
| Rust | [cargo-platform](https://crates.io/crates/cargo-platform/0.1.9) | 0.1.9 | MIT OR Apache-2.0 |
| Rust | [cargo_metadata](https://crates.io/crates/cargo_metadata/0.19.2) | 0.19.2 | MIT |
| Rust | [cargo_toml](https://crates.io/crates/cargo_toml/1.0.1) | 1.0.1 | Apache-2.0 OR MIT |
| Rust | [cc](https://crates.io/crates/cc/1.5.1) | 1.5.1 | MIT OR Apache-2.0 |
| Rust | [cfb](https://crates.io/crates/cfb/0.14.0) | 0.14.0 | MIT |
| Rust | [cfb](https://crates.io/crates/cfb/0.7.3) | 0.7.3 | MIT |
| Rust | [cfg-if](https://crates.io/crates/cfg-if/1.0.5) | 1.0.5 | MIT OR Apache-2.0 |
| Rust | [cfg_aliases](https://crates.io/crates/cfg_aliases/0.1.1) | 0.1.1 | MIT |
| Rust | [chrono](https://crates.io/crates/chrono/0.4.45) | 0.4.45 | MIT OR Apache-2.0 |
| Rust | [clipboard-win](https://crates.io/crates/clipboard-win/5.4.1) | 5.4.1 | BSL-1.0 |
| Rust | [cookie](https://crates.io/crates/cookie/0.18.2) | 0.18.2 | MIT OR Apache-2.0 |
| Rust | [cpufeatures](https://crates.io/crates/cpufeatures/0.2.17) | 0.2.17 | MIT OR Apache-2.0 |
| Rust | [crc32fast](https://crates.io/crates/crc32fast/1.5.2) | 1.5.2 | MIT OR Apache-2.0 |
| Rust | [crossbeam-channel](https://crates.io/crates/crossbeam-channel/0.5.17) | 0.5.17 | MIT OR Apache-2.0 |
| Rust | [crossbeam-utils](https://crates.io/crates/crossbeam-utils/0.8.23) | 0.8.23 | MIT OR Apache-2.0 |
| Rust | [crypto-common](https://crates.io/crates/crypto-common/0.1.7) | 0.1.7 | MIT OR Apache-2.0 |
| Rust | [cssparser](https://crates.io/crates/cssparser/0.37.0) | 0.37.0 | MPL-2.0 |
| Rust | [cssparser-macros](https://crates.io/crates/cssparser-macros/0.7.1) | 0.7.1 | MPL-2.0 |
| Rust | [ctor](https://crates.io/crates/ctor/1.0.13) | 1.0.13 | Apache-2.0 OR MIT |
| Rust | [darling](https://crates.io/crates/darling/0.24.1) | 0.24.1 | MIT |
| Rust | [darling_core](https://crates.io/crates/darling_core/0.24.1) | 0.24.1 | MIT |
| Rust | [darling_macro](https://crates.io/crates/darling_macro/0.24.1) | 0.24.1 | MIT |
| Rust | [defmt](https://crates.io/crates/defmt/1.1.1) | 1.1.1 | MIT OR Apache-2.0 |
| Rust | [defmt-macros](https://crates.io/crates/defmt-macros/1.1.1) | 1.1.1 | MIT OR Apache-2.0 |
| Rust | [defmt-parser](https://crates.io/crates/defmt-parser/1.0.0) | 1.0.0 | MIT OR Apache-2.0 |
| Rust | [deranged](https://crates.io/crates/deranged/0.5.8) | 0.5.8 | MIT OR Apache-2.0 |
| Rust | [derive_more](https://crates.io/crates/derive_more/2.1.1) | 2.1.1 | MIT |
| Rust | [derive_more-impl](https://crates.io/crates/derive_more-impl/2.1.1) | 2.1.1 | MIT |
| Rust | [digest](https://crates.io/crates/digest/0.10.7) | 0.10.7 | MIT OR Apache-2.0 |
| Rust | [dirs](https://crates.io/crates/dirs/7.0.0) | 7.0.0 | MIT OR Apache-2.0 |
| Rust | [dirs-sys](https://crates.io/crates/dirs-sys/0.5.0) | 0.5.0 | MIT OR Apache-2.0 |
| Rust | [displaydoc](https://crates.io/crates/displaydoc/0.2.7) | 0.2.7 | MIT OR Apache-2.0 |
| Rust | [dom_query](https://crates.io/crates/dom_query/0.28.0) | 0.28.0 | MIT |
| Rust | [downcast-rs](https://crates.io/crates/downcast-rs/1.2.1) | 1.2.1 | MIT/Apache-2.0 |
| Rust | [dpi](https://crates.io/crates/dpi/0.1.2) | 0.1.2 | Apache-2.0 AND MIT |
| Rust | [dtoa](https://crates.io/crates/dtoa/1.0.11) | 1.0.11 | MIT OR Apache-2.0 |
| Rust | [dtoa-short](https://crates.io/crates/dtoa-short/0.3.5) | 0.3.5 | MPL-2.0 |
| Rust | [dunce](https://crates.io/crates/dunce/1.0.5) | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 |
| Rust | [dyn-clone](https://crates.io/crates/dyn-clone/1.0.20) | 1.0.20 | MIT OR Apache-2.0 |
| Rust | [embed-resource](https://crates.io/crates/embed-resource/3.0.11) | 3.0.11 | MIT |
| Rust | [equivalent](https://crates.io/crates/equivalent/1.0.2) | 1.0.2 | Apache-2.0 OR MIT |
| Rust | [erased-serde](https://crates.io/crates/erased-serde/0.4.10) | 0.4.10 | MIT OR Apache-2.0 |
| Rust | [error-code](https://crates.io/crates/error-code/3.4.0) | 3.4.0 | BSL-1.0 |
| Rust | [fastrand](https://crates.io/crates/fastrand/2.5.0) | 2.5.0 | Apache-2.0 OR MIT |
| Rust | [fax](https://crates.io/crates/fax/0.2.7) | 0.2.7 | MIT |
| Rust | [fdeflate](https://crates.io/crates/fdeflate/0.3.7) | 0.3.7 | MIT OR Apache-2.0 |
| Rust | [filedescriptor](https://crates.io/crates/filedescriptor/0.8.3) | 0.8.3 | MIT |
| Rust | [find-msvc-tools](https://crates.io/crates/find-msvc-tools/0.1.14) | 0.1.14 | MIT OR Apache-2.0 |
| Rust | [flate2](https://crates.io/crates/flate2/1.1.10) | 1.1.10 | MIT OR Apache-2.0 |
| Rust | [fnv](https://crates.io/crates/fnv/1.0.7) | 1.0.7 | Apache-2.0 / MIT |
| Rust | [foldhash](https://crates.io/crates/foldhash/0.2.0) | 0.2.0 | Zlib |
| Rust | [form_urlencoded](https://crates.io/crates/form_urlencoded/1.2.2) | 1.2.2 | MIT OR Apache-2.0 |
| Rust | [futures-channel](https://crates.io/crates/futures-channel/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| Rust | [futures-core](https://crates.io/crates/futures-core/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| Rust | [futures-io](https://crates.io/crates/futures-io/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| Rust | [futures-macro](https://crates.io/crates/futures-macro/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| Rust | [futures-sink](https://crates.io/crates/futures-sink/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| Rust | [futures-task](https://crates.io/crates/futures-task/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| Rust | [futures-util](https://crates.io/crates/futures-util/0.3.34) | 0.3.34 | MIT OR Apache-2.0 |
| Rust | [generic-array](https://crates.io/crates/generic-array/0.14.7) | 0.14.7 | MIT |
| Rust | [getrandom](https://crates.io/crates/getrandom/0.2.17) | 0.2.17 | MIT OR Apache-2.0 |
| Rust | [getrandom](https://crates.io/crates/getrandom/0.3.4) | 0.3.4 | MIT OR Apache-2.0 |
| Rust | [getrandom](https://crates.io/crates/getrandom/0.4.3) | 0.4.3 | MIT OR Apache-2.0 |
| Rust | [glob](https://crates.io/crates/glob/0.3.4) | 0.3.4 | MIT OR Apache-2.0 |
| Rust | [half](https://crates.io/crates/half/2.7.1) | 2.7.1 | MIT OR Apache-2.0 |
| Rust | [hashbrown](https://crates.io/crates/hashbrown/0.12.3) | 0.12.3 | MIT OR Apache-2.0 |
| Rust | [hashbrown](https://crates.io/crates/hashbrown/0.17.1) | 0.17.1 | MIT OR Apache-2.0 |
| Rust | [heck](https://crates.io/crates/heck/0.5.0) | 0.5.0 | MIT OR Apache-2.0 |
| Rust | [hex](https://crates.io/crates/hex/0.4.3) | 0.4.3 | MIT OR Apache-2.0 |
| Rust | [html5ever](https://crates.io/crates/html5ever/0.39.0) | 0.39.0 | MIT OR Apache-2.0 |
| Rust | [http](https://crates.io/crates/http/1.5.0) | 1.5.0 | MIT OR Apache-2.0 |
| Rust | [http-body](https://crates.io/crates/http-body/1.1.0) | 1.1.0 | MIT |
| Rust | [http-body-util](https://crates.io/crates/http-body-util/0.1.5) | 0.1.5 | MIT |
| Rust | [httparse](https://crates.io/crates/httparse/1.10.1) | 1.10.1 | MIT OR Apache-2.0 |
| Rust | [hyper](https://crates.io/crates/hyper/1.11.1) | 1.11.1 | MIT |
| Rust | [hyper-rustls](https://crates.io/crates/hyper-rustls/0.27.10) | 0.27.10 | Apache-2.0 OR ISC OR MIT |
| Rust | [hyper-util](https://crates.io/crates/hyper-util/0.1.21) | 0.1.21 | MIT |
| Rust | [ico](https://crates.io/crates/ico/0.5.0) | 0.5.0 | MIT |
| Rust | [icu_collections](https://crates.io/crates/icu_collections/2.3.0) | 2.3.0 | Unicode-3.0 |
| Rust | [icu_locale_core](https://crates.io/crates/icu_locale_core/2.3.0) | 2.3.0 | Unicode-3.0 |
| Rust | [icu_normalizer](https://crates.io/crates/icu_normalizer/2.3.0) | 2.3.0 | Unicode-3.0 |
| Rust | [icu_normalizer_data](https://crates.io/crates/icu_normalizer_data/2.3.0) | 2.3.0 | Unicode-3.0 |
| Rust | [icu_properties](https://crates.io/crates/icu_properties/2.3.0) | 2.3.0 | Unicode-3.0 |
| Rust | [icu_properties_data](https://crates.io/crates/icu_properties_data/2.3.0) | 2.3.0 | Unicode-3.0 |
| Rust | [icu_provider](https://crates.io/crates/icu_provider/2.3.1) | 2.3.1 | Unicode-3.0 |
| Rust | [ident_case](https://crates.io/crates/ident_case/1.0.1) | 1.0.1 | MIT/Apache-2.0 |
| Rust | [idna](https://crates.io/crates/idna/1.1.0) | 1.1.0 | MIT OR Apache-2.0 |
| Rust | [idna_adapter](https://crates.io/crates/idna_adapter/1.2.2) | 1.2.2 | Apache-2.0 OR MIT |
| Rust | [image](https://crates.io/crates/image/0.25.10) | 0.25.10 | MIT OR Apache-2.0 |
| Rust | [indexmap](https://crates.io/crates/indexmap/1.9.3) | 1.9.3 | Apache-2.0 OR MIT |
| Rust | [indexmap](https://crates.io/crates/indexmap/2.14.2) | 2.14.2 | Apache-2.0 OR MIT |
| Rust | [infer](https://crates.io/crates/infer/0.19.0) | 0.19.0 | MIT |
| Rust | [infer](https://crates.io/crates/infer/0.22.0) | 0.22.0 | MIT |
| Rust | [ipnet](https://crates.io/crates/ipnet/2.12.2) | 2.12.2 | MIT OR Apache-2.0 |
| Rust | [itoa](https://crates.io/crates/itoa/1.0.18) | 1.0.18 | MIT OR Apache-2.0 |
| Rust | [jiff](https://crates.io/crates/jiff/0.2.37) | 0.2.37 | Unlicense OR MIT |
| Rust | [jiff-core](https://crates.io/crates/jiff-core/0.1.1) | 0.1.1 | Unlicense OR MIT |
| Rust | [jiff-tzdb](https://crates.io/crates/jiff-tzdb/0.1.8) | 0.1.8 | Unlicense OR MIT |
| Rust | [jiff-tzdb-platform](https://crates.io/crates/jiff-tzdb-platform/0.1.3) | 0.1.3 | Unlicense OR MIT |
| Rust | [json-patch](https://crates.io/crates/json-patch/4.2.0) | 4.2.0 | MIT/Apache-2.0 |
| Rust | [jsonptr](https://crates.io/crates/jsonptr/0.7.1) | 0.7.1 | MIT OR Apache-2.0 |
| Rust | [keyboard-types](https://crates.io/crates/keyboard-types/0.8.3) | 0.8.3 | MIT OR Apache-2.0 |
| Rust | [lazy_static](https://crates.io/crates/lazy_static/1.5.0) | 1.5.0 | MIT OR Apache-2.0 |
| Rust | [libc](https://crates.io/crates/libc/0.2.189) | 0.2.189 | MIT OR Apache-2.0 |
| Rust | [litemap](https://crates.io/crates/litemap/0.8.3) | 0.8.3 | Unicode-3.0 |
| Rust | [lock_api](https://crates.io/crates/lock_api/0.4.14) | 0.4.14 | MIT OR Apache-2.0 |
| Rust | [log](https://crates.io/crates/log/0.4.34) | 0.4.34 | MIT OR Apache-2.0 |
| Rust | [markup5ever](https://crates.io/crates/markup5ever/0.39.0) | 0.39.0 | MIT OR Apache-2.0 |
| Rust | [memchr](https://crates.io/crates/memchr/2.8.3) | 2.8.3 | Unlicense OR MIT |
| Rust | [mime](https://crates.io/crates/mime/0.3.17) | 0.3.17 | MIT OR Apache-2.0 |
| Rust | [minisign-verify](https://crates.io/crates/minisign-verify/0.2.5) | 0.2.5 | MIT |
| Rust | [miniz_oxide](https://crates.io/crates/miniz_oxide/0.8.9) | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| Rust | [miniz_oxide](https://crates.io/crates/miniz_oxide/0.9.1) | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| Rust | [mio](https://crates.io/crates/mio/1.2.3) | 1.2.3 | MIT |
| Rust | [moxcms](https://crates.io/crates/moxcms/0.8.1) | 0.8.1 | BSD-3-Clause OR Apache-2.0 |
| Rust | [muda](https://crates.io/crates/muda/0.20.0) | 0.20.0 | Apache-2.0 OR MIT |
| Rust | [new_debug_unreachable](https://crates.io/crates/new_debug_unreachable/1.0.6) | 1.0.6 | MIT |
| Rust | [nix](https://crates.io/crates/nix/0.28.0) | 0.28.0 | MIT |
| Rust | [num-conv](https://crates.io/crates/num-conv/0.2.2) | 0.2.2 | MIT OR Apache-2.0 |
| Rust | [num-traits](https://crates.io/crates/num-traits/0.2.19) | 0.2.19 | MIT OR Apache-2.0 |
| Rust | [once_cell](https://crates.io/crates/once_cell/1.21.4) | 1.21.4 | MIT OR Apache-2.0 |
| Rust | [option-ext](https://crates.io/crates/option-ext/0.2.0) | 0.2.0 | MPL-2.0 |
| Rust | [parking_lot](https://crates.io/crates/parking_lot/0.12.5) | 0.12.5 | MIT OR Apache-2.0 |
| Rust | [parking_lot_core](https://crates.io/crates/parking_lot_core/0.9.12) | 0.9.12 | MIT OR Apache-2.0 |
| Rust | [percent-encoding](https://crates.io/crates/percent-encoding/2.3.2) | 2.3.2 | MIT OR Apache-2.0 |
| Rust | [phf](https://crates.io/crates/phf/0.13.1) | 0.13.1 | MIT |
| Rust | [phf_codegen](https://crates.io/crates/phf_codegen/0.13.1) | 0.13.1 | MIT |
| Rust | [phf_generator](https://crates.io/crates/phf_generator/0.13.1) | 0.13.1 | MIT |
| Rust | [phf_macros](https://crates.io/crates/phf_macros/0.13.1) | 0.13.1 | MIT |
| Rust | [phf_shared](https://crates.io/crates/phf_shared/0.13.1) | 0.13.1 | MIT |
| Rust | [pin-project-lite](https://crates.io/crates/pin-project-lite/0.2.17) | 0.2.17 | Apache-2.0 OR MIT |
| Rust | [plist](https://crates.io/crates/plist/1.10.1) | 1.10.1 | MIT |
| Rust | [png](https://crates.io/crates/png/0.17.16) | 0.17.16 | MIT OR Apache-2.0 |
| Rust | [png](https://crates.io/crates/png/0.18.1) | 0.18.1 | MIT OR Apache-2.0 |
| Rust | [portable-pty](https://crates.io/crates/portable-pty/0.9.0) | 0.9.0 | MIT |
| Rust | [potential_utf](https://crates.io/crates/potential_utf/0.1.6) | 0.1.6 | Unicode-3.0 |
| Rust | [powerfmt](https://crates.io/crates/powerfmt/0.2.0) | 0.2.0 | MIT OR Apache-2.0 |
| Rust | [precomputed-hash](https://crates.io/crates/precomputed-hash/0.1.1) | 0.1.1 | MIT |
| Rust | [proc-macro2](https://crates.io/crates/proc-macro2/1.0.107) | 1.0.107 | MIT OR Apache-2.0 |
| Rust | [pxfm](https://crates.io/crates/pxfm/0.1.30) | 0.1.30 | BSD-3-Clause OR Apache-2.0 |
| Rust | [quick-error](https://crates.io/crates/quick-error/2.0.1) | 2.0.1 | MIT/Apache-2.0 |
| Rust | [quick-xml](https://crates.io/crates/quick-xml/0.42.0) | 0.42.0 | MIT |
| Rust | [quote](https://crates.io/crates/quote/1.0.47) | 1.0.47 | MIT OR Apache-2.0 |
| Rust | [raw-window-handle](https://crates.io/crates/raw-window-handle/0.6.2) | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| Rust | [ref-cast](https://crates.io/crates/ref-cast/1.0.27) | 1.0.27 | MIT OR Apache-2.0 |
| Rust | [ref-cast-impl](https://crates.io/crates/ref-cast-impl/1.0.27) | 1.0.27 | MIT OR Apache-2.0 |
| Rust | [regex](https://crates.io/crates/regex/1.13.1) | 1.13.1 | MIT OR Apache-2.0 |
| Rust | [regex-automata](https://crates.io/crates/regex-automata/0.4.18) | 0.4.18 | MIT OR Apache-2.0 |
| Rust | [regex-syntax](https://crates.io/crates/regex-syntax/0.8.11) | 0.8.11 | MIT OR Apache-2.0 |
| Rust | [reqwest](https://crates.io/crates/reqwest/0.13.5) | 0.13.5 | MIT OR Apache-2.0 |
| Rust | [rfd](https://crates.io/crates/rfd/0.16.0) | 0.16.0 | MIT |
| Rust | [ring](https://crates.io/crates/ring/0.17.14) | 0.17.14 | Apache-2.0 AND ISC |
| Rust | [rustc-hash](https://crates.io/crates/rustc-hash/2.1.3) | 2.1.3 | Apache-2.0 OR MIT |
| Rust | [rustc_version](https://crates.io/crates/rustc_version/0.4.1) | 0.4.1 | MIT OR Apache-2.0 |
| Rust | [rustls](https://crates.io/crates/rustls/0.23.45) | 0.23.45 | Apache-2.0 OR ISC OR MIT |
| Rust | [rustls-pki-types](https://crates.io/crates/rustls-pki-types/1.15.1) | 1.15.1 | MIT OR Apache-2.0 |
| Rust | [rustls-platform-verifier](https://crates.io/crates/rustls-platform-verifier/0.7.1) | 0.7.1 | MIT OR Apache-2.0 |
| Rust | [rustls-webpki](https://crates.io/crates/rustls-webpki/0.103.15) | 0.103.15 | ISC |
| Rust | [same-file](https://crates.io/crates/same-file/1.0.6) | 1.0.6 | Unlicense/MIT |
| Rust | [schemars](https://crates.io/crates/schemars/0.8.22) | 0.8.22 | MIT |
| Rust | [schemars](https://crates.io/crates/schemars/0.9.0) | 0.9.0 | MIT |
| Rust | [schemars](https://crates.io/crates/schemars/1.2.2) | 1.2.2 | MIT |
| Rust | [schemars_derive](https://crates.io/crates/schemars_derive/0.8.22) | 0.8.22 | MIT |
| Rust | [scopeguard](https://crates.io/crates/scopeguard/1.2.0) | 1.2.0 | MIT OR Apache-2.0 |
| Rust | [selectors](https://crates.io/crates/selectors/0.38.0) | 0.38.0 | MPL-2.0 |
| Rust | [semver](https://crates.io/crates/semver/1.0.28) | 1.0.28 | MIT OR Apache-2.0 |
| Rust | [serde](https://crates.io/crates/serde/1.0.229) | 1.0.229 | MIT OR Apache-2.0 |
| Rust | [serde-untagged](https://crates.io/crates/serde-untagged/0.1.9) | 0.1.9 | MIT OR Apache-2.0 |
| Rust | [serde_core](https://crates.io/crates/serde_core/1.0.229) | 1.0.229 | MIT OR Apache-2.0 |
| Rust | [serde_derive](https://crates.io/crates/serde_derive/1.0.229) | 1.0.229 | MIT OR Apache-2.0 |
| Rust | [serde_derive_internals](https://crates.io/crates/serde_derive_internals/0.29.1) | 0.29.1 | MIT OR Apache-2.0 |
| Rust | [serde_json](https://crates.io/crates/serde_json/1.0.151) | 1.0.151 | MIT OR Apache-2.0 |
| Rust | [serde_repr](https://crates.io/crates/serde_repr/0.1.21) | 0.1.21 | MIT OR Apache-2.0 |
| Rust | [serde_spanned](https://crates.io/crates/serde_spanned/1.1.1) | 1.1.1 | MIT OR Apache-2.0 |
| Rust | [serde_with](https://crates.io/crates/serde_with/3.23.0) | 3.23.0 | MIT OR Apache-2.0 |
| Rust | [serde_with_macros](https://crates.io/crates/serde_with_macros/3.23.0) | 3.23.0 | MIT OR Apache-2.0 |
| Rust | [serial2](https://crates.io/crates/serial2/0.2.38) | 0.2.38 | BSD-2-Clause OR Apache-2.0 |
| Rust | [serialize-to-javascript](https://crates.io/crates/serialize-to-javascript/0.1.2) | 0.1.2 | MIT OR Apache-2.0 |
| Rust | [serialize-to-javascript-impl](https://crates.io/crates/serialize-to-javascript-impl/0.1.2) | 0.1.2 | MIT OR Apache-2.0 |
| Rust | [servo_arc](https://crates.io/crates/servo_arc/0.4.3) | 0.4.3 | MIT OR Apache-2.0 |
| Rust | [sha2](https://crates.io/crates/sha2/0.10.9) | 0.10.9 | MIT OR Apache-2.0 |
| Rust | [shared_library](https://crates.io/crates/shared_library/0.1.9) | 0.1.9 | Apache-2.0/MIT |
| Rust | [shell-words](https://crates.io/crates/shell-words/1.1.1) | 1.1.1 | MIT/Apache-2.0 |
| Rust | [shlex](https://crates.io/crates/shlex/2.0.1) | 2.0.1 | MIT OR Apache-2.0 |
| Rust | [simd-adler32](https://crates.io/crates/simd-adler32/0.3.10) | 0.3.10 | MIT |
| Rust | [siphasher](https://crates.io/crates/siphasher/1.0.4) | 1.0.4 | MIT OR Apache-2.0 |
| Rust | [slab](https://crates.io/crates/slab/0.4.12) | 0.4.12 | MIT |
| Rust | [smallvec](https://crates.io/crates/smallvec/1.16.2) | 1.16.2 | MIT OR Apache-2.0 |
| Rust | [socket2](https://crates.io/crates/socket2/0.6.5) | 0.6.5 | MIT OR Apache-2.0 |
| Rust | [softbuffer](https://crates.io/crates/softbuffer/0.4.8) | 0.4.8 | MIT OR Apache-2.0 |
| Rust | [stable_deref_trait](https://crates.io/crates/stable_deref_trait/1.2.1) | 1.2.1 | MIT OR Apache-2.0 |
| Rust | [string_cache](https://crates.io/crates/string_cache/0.9.0) | 0.9.0 | MIT OR Apache-2.0 |
| Rust | [string_cache_codegen](https://crates.io/crates/string_cache_codegen/0.6.1) | 0.6.1 | MIT OR Apache-2.0 |
| Rust | [strsim](https://crates.io/crates/strsim/0.11.1) | 0.11.1 | MIT |
| Rust | [subtle](https://crates.io/crates/subtle/2.6.1) | 2.6.1 | BSD-3-Clause |
| Rust | [syn](https://crates.io/crates/syn/2.0.119) | 2.0.119 | MIT OR Apache-2.0 |
| Rust | [syn](https://crates.io/crates/syn/3.0.6) | 3.0.6 | MIT OR Apache-2.0 |
| Rust | [sync_wrapper](https://crates.io/crates/sync_wrapper/1.0.2) | 1.0.2 | Apache-2.0 |
| Rust | [synstructure](https://crates.io/crates/synstructure/0.14.0) | 0.14.0 | MIT |
| Rust | [tao](https://crates.io/crates/tao/0.37.1) | 0.37.1 | Apache-2.0 |
| Rust | [tauri](https://crates.io/crates/tauri/2.12.0) | 2.12.0 | Apache-2.0 OR MIT |
| Rust | [tauri-build](https://crates.io/crates/tauri-build/2.7.0) | 2.7.0 | Apache-2.0 OR MIT |
| Rust | [tauri-codegen](https://crates.io/crates/tauri-codegen/2.7.0) | 2.7.0 | Apache-2.0 OR MIT |
| Rust | [tauri-macros](https://crates.io/crates/tauri-macros/2.7.0) | 2.7.0 | Apache-2.0 OR MIT |
| Rust | [tauri-plugin](https://crates.io/crates/tauri-plugin/2.7.0) | 2.7.0 | Apache-2.0 OR MIT |
| Rust | [tauri-plugin-autostart](https://crates.io/crates/tauri-plugin-autostart/2.6.0) | 2.6.0 | Apache-2.0 OR MIT |
| Rust | [tauri-plugin-clipboard-manager](https://crates.io/crates/tauri-plugin-clipboard-manager/2.4.0) | 2.4.0 | Apache-2.0 OR MIT |
| Rust | [tauri-plugin-dialog](https://crates.io/crates/tauri-plugin-dialog/2.8.0) | 2.8.0 | Apache-2.0 OR MIT |
| Rust | [tauri-plugin-fs](https://crates.io/crates/tauri-plugin-fs/2.6.0) | 2.6.0 | Apache-2.0 OR MIT |
| Rust | [tauri-plugin-process](https://crates.io/crates/tauri-plugin-process/2.4.0) | 2.4.0 | Apache-2.0 OR MIT |
| Rust | [tauri-plugin-single-instance](https://crates.io/crates/tauri-plugin-single-instance/2.5.0) | 2.5.0 | Apache-2.0 OR MIT |
| Rust | [tauri-plugin-updater](https://crates.io/crates/tauri-plugin-updater/2.13.0) | 2.13.0 | Apache-2.0 OR MIT |
| Rust | [tauri-runtime](https://crates.io/crates/tauri-runtime/2.12.0) | 2.12.0 | Apache-2.0 OR MIT |
| Rust | [tauri-runtime-wry](https://crates.io/crates/tauri-runtime-wry/2.12.0) | 2.12.0 | Apache-2.0 OR MIT |
| Rust | [tauri-utils](https://crates.io/crates/tauri-utils/2.10.0) | 2.10.0 | Apache-2.0 OR MIT |
| Rust | [tauri-winres](https://crates.io/crates/tauri-winres/0.3.6) | 0.3.6 | MIT |
| Rust | [tempfile](https://crates.io/crates/tempfile/3.27.0) | 3.27.0 | MIT OR Apache-2.0 |
| Rust | [tendril](https://crates.io/crates/tendril/0.5.1) | 0.5.1 | MIT OR Apache-2.0 |
| Rust | [thiserror](https://crates.io/crates/thiserror/1.0.69) | 1.0.69 | MIT OR Apache-2.0 |
| Rust | [thiserror](https://crates.io/crates/thiserror/2.0.21) | 2.0.21 | MIT OR Apache-2.0 |
| Rust | [thiserror-impl](https://crates.io/crates/thiserror-impl/1.0.69) | 1.0.69 | MIT OR Apache-2.0 |
| Rust | [thiserror-impl](https://crates.io/crates/thiserror-impl/2.0.21) | 2.0.21 | MIT OR Apache-2.0 |
| Rust | [tiff](https://crates.io/crates/tiff/0.11.3) | 0.11.3 | MIT |
| Rust | [time](https://crates.io/crates/time/0.3.55) | 0.3.55 | MIT OR Apache-2.0 |
| Rust | [time-core](https://crates.io/crates/time-core/0.1.9) | 0.1.9 | MIT OR Apache-2.0 |
| Rust | [time-macros](https://crates.io/crates/time-macros/0.2.32) | 0.2.32 | MIT OR Apache-2.0 |
| Rust | [tinystr](https://crates.io/crates/tinystr/0.8.4) | 0.8.4 | Unicode-3.0 |
| Rust | [tinyvec](https://crates.io/crates/tinyvec/1.13.3) | 1.13.3 | Zlib OR Apache-2.0 OR MIT |
| Rust | [tokio](https://crates.io/crates/tokio/1.53.1) | 1.53.1 | MIT |
| Rust | [tokio-rustls](https://crates.io/crates/tokio-rustls/0.26.6) | 0.26.6 | MIT OR Apache-2.0 |
| Rust | [tokio-util](https://crates.io/crates/tokio-util/0.7.19) | 0.7.19 | MIT |
| Rust | [toml](https://crates.io/crates/toml/1.1.6+spec-1.1.0) | 1.1.6+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | [toml_datetime](https://crates.io/crates/toml_datetime/1.1.1+spec-1.1.0) | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | [toml_parser](https://crates.io/crates/toml_parser/1.1.3+spec-1.1.0) | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | [toml_writer](https://crates.io/crates/toml_writer/1.1.2+spec-1.1.0) | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | [tower](https://crates.io/crates/tower/0.5.3) | 0.5.3 | MIT |
| Rust | [tower-http](https://crates.io/crates/tower-http/0.6.11) | 0.6.11 | MIT |
| Rust | [tower-layer](https://crates.io/crates/tower-layer/0.3.3) | 0.3.3 | MIT |
| Rust | [tower-service](https://crates.io/crates/tower-service/0.3.3) | 0.3.3 | MIT |
| Rust | [tracing](https://crates.io/crates/tracing/0.1.44) | 0.1.44 | MIT |
| Rust | [tracing-attributes](https://crates.io/crates/tracing-attributes/0.1.31) | 0.1.31 | MIT |
| Rust | [tracing-core](https://crates.io/crates/tracing-core/0.1.36) | 0.1.36 | MIT |
| Rust | [tray-icon](https://crates.io/crates/tray-icon/0.25.1) | 0.25.1 | MIT OR Apache-2.0 |
| Rust | [try-lock](https://crates.io/crates/try-lock/0.2.5) | 0.2.5 | MIT |
| Rust | [typeid](https://crates.io/crates/typeid/1.0.3) | 1.0.3 | MIT OR Apache-2.0 |
| Rust | [typenum](https://crates.io/crates/typenum/1.20.1) | 1.20.1 | MIT OR Apache-2.0 |
| Rust | [unicode-ident](https://crates.io/crates/unicode-ident/1.0.26) | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| Rust | [unicode-segmentation](https://crates.io/crates/unicode-segmentation/1.13.3) | 1.13.3 | MIT OR Apache-2.0 |
| Rust | [untrusted](https://crates.io/crates/untrusted/0.9.0) | 0.9.0 | ISC |
| Rust | [url](https://crates.io/crates/url/2.5.8) | 2.5.8 | MIT OR Apache-2.0 |
| Rust | [urlpattern](https://crates.io/crates/urlpattern/0.6.0) | 0.6.0 | MIT |
| Rust | [utf8_iter](https://crates.io/crates/utf8_iter/1.0.4) | 1.0.4 | Apache-2.0 OR MIT |
| Rust | [uuid](https://crates.io/crates/uuid/1.26.1) | 1.26.1 | Apache-2.0 OR MIT |
| Rust | [version_check](https://crates.io/crates/version_check/0.9.5) | 0.9.5 | MIT/Apache-2.0 |
| Rust | [vswhom](https://crates.io/crates/vswhom/0.1.0) | 0.1.0 | MIT |
| Rust | [vswhom-sys](https://crates.io/crates/vswhom-sys/0.1.3) | 0.1.3 | MIT |
| Rust | [walkdir](https://crates.io/crates/walkdir/2.5.0) | 2.5.0 | Unlicense/MIT |
| Rust | [want](https://crates.io/crates/want/0.3.1) | 0.3.1 | MIT |
| Rust | [web-time](https://crates.io/crates/web-time/1.1.0) | 1.1.0 | MIT OR Apache-2.0 |
| Rust | [web_atoms](https://crates.io/crates/web_atoms/0.2.6) | 0.2.6 | MIT OR Apache-2.0 |
| Rust | [webview2-com](https://crates.io/crates/webview2-com/0.39.1) | 0.39.1 | MIT |
| Rust | [webview2-com-macros](https://crates.io/crates/webview2-com-macros/0.8.1) | 0.8.1 | MIT |
| Rust | [webview2-com-sys](https://crates.io/crates/webview2-com-sys/0.39.1) | 0.39.1 | MIT |
| Rust | [weezl](https://crates.io/crates/weezl/0.1.12) | 0.1.12 | MIT OR Apache-2.0 |
| Rust | [winapi](https://crates.io/crates/winapi/0.3.9) | 0.3.9 | MIT/Apache-2.0 |
| Rust | [winapi-util](https://crates.io/crates/winapi-util/0.1.11) | 0.1.11 | Unlicense OR MIT |
| Rust | [window-vibrancy](https://crates.io/crates/window-vibrancy/0.8.1) | 0.8.1 | Apache-2.0 OR MIT |
| Rust | [windows](https://crates.io/crates/windows/0.62.2) | 0.62.2 | MIT OR Apache-2.0 |
| Rust | [windows-collections](https://crates.io/crates/windows-collections/0.3.2) | 0.3.2 | MIT OR Apache-2.0 |
| Rust | [windows-core](https://crates.io/crates/windows-core/0.62.2) | 0.62.2 | MIT OR Apache-2.0 |
| Rust | [windows-future](https://crates.io/crates/windows-future/0.3.2) | 0.3.2 | MIT OR Apache-2.0 |
| Rust | [windows-implement](https://crates.io/crates/windows-implement/0.60.2) | 0.60.2 | MIT OR Apache-2.0 |
| Rust | [windows-interface](https://crates.io/crates/windows-interface/0.59.3) | 0.59.3 | MIT OR Apache-2.0 |
| Rust | [windows-link](https://crates.io/crates/windows-link/0.2.1) | 0.2.1 | MIT OR Apache-2.0 |
| Rust | [windows-numerics](https://crates.io/crates/windows-numerics/0.3.1) | 0.3.1 | MIT OR Apache-2.0 |
| Rust | [windows-registry](https://crates.io/crates/windows-registry/0.6.1) | 0.6.1 | MIT OR Apache-2.0 |
| Rust | [windows-result](https://crates.io/crates/windows-result/0.4.1) | 0.4.1 | MIT OR Apache-2.0 |
| Rust | [windows-strings](https://crates.io/crates/windows-strings/0.5.1) | 0.5.1 | MIT OR Apache-2.0 |
| Rust | [windows-sys](https://crates.io/crates/windows-sys/0.59.0) | 0.59.0 | MIT OR Apache-2.0 |
| Rust | [windows-sys](https://crates.io/crates/windows-sys/0.60.2) | 0.60.2 | MIT OR Apache-2.0 |
| Rust | [windows-sys](https://crates.io/crates/windows-sys/0.61.2) | 0.61.2 | MIT OR Apache-2.0 |
| Rust | [windows-targets](https://crates.io/crates/windows-targets/0.52.6) | 0.52.6 | MIT OR Apache-2.0 |
| Rust | [windows-targets](https://crates.io/crates/windows-targets/0.53.5) | 0.53.5 | MIT OR Apache-2.0 |
| Rust | [windows-threading](https://crates.io/crates/windows-threading/0.2.1) | 0.2.1 | MIT OR Apache-2.0 |
| Rust | [windows-version](https://crates.io/crates/windows-version/0.1.7) | 0.1.7 | MIT OR Apache-2.0 |
| Rust | [windows_x86_64_msvc](https://crates.io/crates/windows_x86_64_msvc/0.52.6) | 0.52.6 | MIT OR Apache-2.0 |
| Rust | [windows_x86_64_msvc](https://crates.io/crates/windows_x86_64_msvc/0.53.1) | 0.53.1 | MIT OR Apache-2.0 |
| Rust | [winnow](https://crates.io/crates/winnow/1.0.4) | 1.0.4 | MIT |
| Rust | [winreg](https://crates.io/crates/winreg/0.10.1) | 0.10.1 | MIT |
| Rust | [winreg](https://crates.io/crates/winreg/0.55.0) | 0.55.0 | MIT |
| Rust | [writeable](https://crates.io/crates/writeable/0.6.4) | 0.6.4 | Unicode-3.0 |
| Rust | [wry](https://crates.io/crates/wry/0.57.0) | 0.57.0 | Apache-2.0 OR MIT |
| Rust | [yoke](https://crates.io/crates/yoke/0.8.3) | 0.8.3 | Unicode-3.0 |
| Rust | [yoke-derive](https://crates.io/crates/yoke-derive/0.8.3) | 0.8.3 | Unicode-3.0 |
| Rust | [zerocopy](https://crates.io/crates/zerocopy/0.8.59) | 0.8.59 | BSD-2-Clause OR Apache-2.0 OR MIT |
| Rust | [zerocopy-derive](https://crates.io/crates/zerocopy-derive/0.8.59) | 0.8.59 | BSD-2-Clause OR Apache-2.0 OR MIT |
| Rust | [zerofrom](https://crates.io/crates/zerofrom/0.1.8) | 0.1.8 | Unicode-3.0 |
| Rust | [zerofrom-derive](https://crates.io/crates/zerofrom-derive/0.1.8) | 0.1.8 | Unicode-3.0 |
| Rust | [zeroize](https://crates.io/crates/zeroize/1.9.0) | 1.9.0 | Apache-2.0 OR MIT |
| Rust | [zerotrie](https://crates.io/crates/zerotrie/0.2.5) | 0.2.5 | Unicode-3.0 |
| Rust | [zerovec](https://crates.io/crates/zerovec/0.11.8) | 0.11.8 | Unicode-3.0 |
| Rust | [zerovec-derive](https://crates.io/crates/zerovec-derive/0.11.6) | 0.11.6 | Unicode-3.0 |
| Rust | [zip](https://crates.io/crates/zip/4.6.1) | 4.6.1 | MIT |
| Rust | [zlib-rs](https://crates.io/crates/zlib-rs/0.6.8) | 0.6.8 | Zlib |
| Rust | [zmij](https://crates.io/crates/zmij/1.0.23) | 1.0.23 | MIT |
| Rust | [zune-core](https://crates.io/crates/zune-core/0.5.3) | 0.5.3 | MIT OR Apache-2.0 OR Zlib |
| Rust | [zune-jpeg](https://crates.io/crates/zune-jpeg/0.5.15) | 0.5.15 | MIT OR Apache-2.0 OR Zlib |
| npm | [@tauri-apps/api](https://www.npmjs.com/package/@tauri-apps/api/v/2.12.0) | 2.12.0 | Apache-2.0 OR MIT |
| npm | [@tauri-apps/plugin-autostart](https://www.npmjs.com/package/@tauri-apps/plugin-autostart/v/2.6.0) | 2.6.0 | MIT OR Apache-2.0 |
| npm | [@tauri-apps/plugin-clipboard-manager](https://www.npmjs.com/package/@tauri-apps/plugin-clipboard-manager/v/2.4.1) | 2.4.1 | MIT OR Apache-2.0 |
| npm | [@tauri-apps/plugin-dialog](https://www.npmjs.com/package/@tauri-apps/plugin-dialog/v/2.8.0) | 2.8.0 | MIT OR Apache-2.0 |
| npm | [@tauri-apps/plugin-process](https://www.npmjs.com/package/@tauri-apps/plugin-process/v/2.4.0) | 2.4.0 | MIT OR Apache-2.0 |
| npm | [@tauri-apps/plugin-updater](https://www.npmjs.com/package/@tauri-apps/plugin-updater/v/2.13.0) | 2.13.0 | MIT OR Apache-2.0 |
| npm | [@xterm/addon-fit](https://www.npmjs.com/package/@xterm/addon-fit/v/0.11.0) | 0.11.0 | MIT |
| npm | [@xterm/xterm](https://www.npmjs.com/package/@xterm/xterm/v/6.0.0) | 6.0.0 | MIT |
| npm | [react](https://www.npmjs.com/package/react/v/19.3.0) | 19.3.0 | MIT |
| npm | [react-dom](https://www.npmjs.com/package/react-dom/v/19.3.0) | 19.3.0 | MIT |
| npm | [scheduler](https://www.npmjs.com/package/scheduler/v/0.28.0) | 0.28.0 | MIT |
