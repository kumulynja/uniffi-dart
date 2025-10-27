use crate::gen::{
    quote,
    render::{Renderable, TypeHelperRenderer},
};

use crate::gen::CodeType;
use genco::lang::dart;

#[derive(Debug)]
pub struct StringCodeType;
impl CodeType for StringCodeType {
    fn type_label(&self) -> String {
        "String".to_owned()
    }
}

impl Renderable for StringCodeType {
    fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        quote! {
            class FfiConverterString {
                static final Map<String, Uint8List> _utf8Cache = {};
                static final List<String> _cacheKeys = [];
                static const int _maxCacheSize = 128;

                static final Uint8List _emptyUtf8 = Uint8List(0);

                static Uint8List _getCachedUtf8(String value) {
                    if (value.isEmpty) return _emptyUtf8;
                    if (value.length > 256) {
                        UniffiMemoryProfiler.incrementStringCacheMisses();
                        return utf8.encoder.convert(value);
                    }

                    if (_utf8Cache.containsKey(value)) {
                        UniffiMemoryProfiler.incrementStringCacheHits();
                        return _utf8Cache[value]!;
                    }

                    UniffiMemoryProfiler.incrementStringCacheMisses();
                    final encoded = utf8.encoder.convert(value);

                    if (_utf8Cache.length >= _maxCacheSize) {
                        final oldestKey = _cacheKeys.removeAt(0);
                        _utf8Cache.remove(oldestKey);
                    }

                    _utf8Cache[value] = encoded;
                    _cacheKeys.add(value);
                    return encoded;
                }

                static String lift( RustBuffer buf) {
                    // reading the entire buffer, the len is where the string finishes
                    return utf8.decoder.convert(buf.asUint8List());
                }

                static RustBuffer lower( String value) {
                    return toRustBuffer(_getCachedUtf8(value));
                }

                static LiftRetVal<String> read( Uint8List buf) {
                    final end = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0) + 4;
                    return LiftRetVal(utf8.decoder.convert(buf, 4, end), end);
                }

                static int allocationSize([String value = ""]) {
                    // Optimized: reuse cached UTF-8 conversion instead of double encoding
                    return _getCachedUtf8(value).length + 4;
                }

                static int write( String value, Uint8List buf) {
                    // Optimized: use cached UTF-8 conversion to eliminate duplicate encoding
                    final list = _getCachedUtf8(value);
                    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, list.length);
                    buf.setAll(4, list);
                    return list.length + 4;
                }
            }
        }
    }
}
