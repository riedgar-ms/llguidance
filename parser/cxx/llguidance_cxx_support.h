#pragma once

#include "llguidance_cxx.h"
#include <cstdint>

namespace llguidance {

class FactoryInit {
protected:
  size_t m_vocab_size;
  uint32_t m_tok_eos;
  uint32_t m_stderr_log_level;
  bool m_allow_ff_tokens;
  bool m_allow_backtracking;
  rust::Vec<rust::String> m_slices;

public:
  FactoryInit(size_t vocab_size, uint32_t tok_eos,
              uint32_t stderr_log_level = 1, bool allow_ff_tokens = false,
              bool allow_backtracking = false);

  virtual ~FactoryInit() = default;

  // Non-virtual configuration getters
  size_t vocab_size() const { return m_vocab_size; }
  uint32_t tok_eos() const { return m_tok_eos; }
  uint32_t stderr_log_level() const { return m_stderr_log_level; }
  bool allow_ff_tokens() const { return m_allow_ff_tokens; }
  bool allow_backtracking() const { return m_allow_backtracking; }
  rust::Vec<rust::String> slices() const { return m_slices; }

  // Virtual methods that need implementation
  virtual rust::Vec<uint8_t> token_bytes(size_t token) const = 0;
  virtual rust::Vec<uint32_t> tokenize(rust::Str text) const = 0;

protected:
  FactoryInit(const FactoryInit &) = delete;
  FactoryInit &operator=(const FactoryInit &) = delete;
  FactoryInit(FactoryInit &&) = delete;
  FactoryInit &operator=(FactoryInit &&) = delete;
};

} // namespace llguidance