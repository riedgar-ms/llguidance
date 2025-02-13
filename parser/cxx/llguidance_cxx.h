#pragma once
#include "rust/cxx.h"
#include "llguidance_cxx_support.h"
#include <memory>

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

::rust::Box<::ParserFactory> parser_factory(::std::unique_ptr<::TokenizerInit> tok_init) noexcept;
