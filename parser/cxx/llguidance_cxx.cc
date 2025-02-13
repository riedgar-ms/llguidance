#include "rust/cxx.h"
#include "llguidance_cxx_support.h"
#include <memory>

namespace rust {
inline namespace cxxbridge1 {
namespace repr {
using Fat = ::std::array<::std::uintptr_t, 2>;
} // namespace repr

namespace {
template <typename T>
class impl<Slice<T>> final {
public:
  static repr::Fat repr(Slice<T> slice) noexcept {
    return slice.repr;
  }
};

template <bool> struct deleter_if {
  template <typename T> void operator()(T *) {}
};

template <> struct deleter_if<true> {
  template <typename T> void operator()(T *ptr) { ptr->~T(); }
};
} // namespace
} // namespace cxxbridge1
} // namespace rust

using TokenizerInit = ::TokenizerInit;
struct ParserFactory;

#ifndef CXXBRIDGE1_STRUCT_ParserFactory
#define CXXBRIDGE1_STRUCT_ParserFactory
struct ParserFactory final : public ::rust::Opaque {
  ~ParserFactory() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_ParserFactory

extern "C" {
::std::size_t cxxbridge1$TokenizerInit$vocab_size(::TokenizerInit const &self) noexcept {
  ::std::size_t (::TokenizerInit::*vocab_size$)() const = &::TokenizerInit::vocab_size;
  return (self.*vocab_size$)();
}

::std::uint32_t cxxbridge1$TokenizerInit$tok_eos(::TokenizerInit const &self) noexcept {
  ::std::uint32_t (::TokenizerInit::*tok_eos$)() const = &::TokenizerInit::tok_eos;
  return (self.*tok_eos$)();
}

::rust::repr::Fat cxxbridge1$TokenizerInit$token_bytes(::TokenizerInit const &self, ::std::size_t token) noexcept {
  ::rust::Slice<::std::uint8_t const> (::TokenizerInit::*token_bytes$)(::std::size_t) const = &::TokenizerInit::token_bytes;
  return ::rust::impl<::rust::Slice<::std::uint8_t const>>::repr((self.*token_bytes$)(token));
}
::std::size_t cxxbridge1$ParserFactory$operator$sizeof() noexcept;
::std::size_t cxxbridge1$ParserFactory$operator$alignof() noexcept;

::ParserFactory *cxxbridge1$parser_factory(::TokenizerInit *tok_init) noexcept;
} // extern "C"

::std::size_t ParserFactory::layout::size() noexcept {
  return cxxbridge1$ParserFactory$operator$sizeof();
}

::std::size_t ParserFactory::layout::align() noexcept {
  return cxxbridge1$ParserFactory$operator$alignof();
}

::rust::Box<::ParserFactory> parser_factory(::std::unique_ptr<::TokenizerInit> tok_init) noexcept {
  return ::rust::Box<::ParserFactory>::from_raw(cxxbridge1$parser_factory(tok_init.release()));
}

extern "C" {
static_assert(::rust::detail::is_complete<::TokenizerInit>::value, "definition of TokenizerInit is required");
static_assert(sizeof(::std::unique_ptr<::TokenizerInit>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::TokenizerInit>) == alignof(void *), "");
void cxxbridge1$unique_ptr$TokenizerInit$null(::std::unique_ptr<::TokenizerInit> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::TokenizerInit>();
}
void cxxbridge1$unique_ptr$TokenizerInit$raw(::std::unique_ptr<::TokenizerInit> *ptr, ::TokenizerInit *raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::TokenizerInit>(raw);
}
::TokenizerInit const *cxxbridge1$unique_ptr$TokenizerInit$get(::std::unique_ptr<::TokenizerInit> const &ptr) noexcept {
  return ptr.get();
}
::TokenizerInit *cxxbridge1$unique_ptr$TokenizerInit$release(::std::unique_ptr<::TokenizerInit> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$TokenizerInit$drop(::std::unique_ptr<::TokenizerInit> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::TokenizerInit>::value>{}(ptr);
}

::ParserFactory *cxxbridge1$box$ParserFactory$alloc() noexcept;
void cxxbridge1$box$ParserFactory$dealloc(::ParserFactory *) noexcept;
void cxxbridge1$box$ParserFactory$drop(::rust::Box<::ParserFactory> *ptr) noexcept;
} // extern "C"

namespace rust {
inline namespace cxxbridge1 {
template <>
::ParserFactory *Box<::ParserFactory>::allocation::alloc() noexcept {
  return cxxbridge1$box$ParserFactory$alloc();
}
template <>
void Box<::ParserFactory>::allocation::dealloc(::ParserFactory *ptr) noexcept {
  cxxbridge1$box$ParserFactory$dealloc(ptr);
}
template <>
void Box<::ParserFactory>::drop() noexcept {
  cxxbridge1$box$ParserFactory$drop(this);
}
} // namespace cxxbridge1
} // namespace rust
