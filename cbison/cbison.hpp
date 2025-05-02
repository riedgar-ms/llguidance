#pragma once
#include <cstdint>
#include <cstddef>
#include <vector>
#include <string>
#include <optional>
#include "cbison_api.h"

namespace cbison {

/// C++ wrapper for a CBISON matcher instance.
class Matcher {
  cbison_factory_t api_;
  cbison_matcher_t m_;

public:
  /// Wrap existing matcher pointer.
  /// @param api  Factory pointer used to free/clone.
  /// @param m    Raw matcher pointer.
  Matcher(cbison_factory_t api, cbison_matcher_t m) noexcept
      : api_(api), m_(m) {}

  /// Frees the matcher.
  ~Matcher() noexcept {
    if (m_)
      api_->free_matcher(m_);
  }

  Matcher(const Matcher &) = delete;
  Matcher &operator=(const Matcher &) = delete;

  Matcher(Matcher &&o) noexcept : api_(o.api_), m_(o.m_) { o.m_ = nullptr; }

  Matcher &operator=(Matcher &&o) noexcept {
    if (m_)
      api_->free_matcher(m_);
    api_ = o.api_;
    m_ = o.m_;
    o.m_ = nullptr;
    return *this;
  }

  cbison_matcher_t get() const noexcept { return m_; }

  /// Clone the matcher.
  /// @return New Matcher.
  Matcher clone() const noexcept {
    auto c = api_->clone_matcher(m_);
    return Matcher(api_, c);
  }

  /// Compute token mask for current state.
  /// @return Vector of representing bitmask for the entire tokenizer
  std::vector<uint32_t> computeMask() const noexcept {
    size_t bytes = api_->mask_byte_len;
    size_t words = bytes / 4;
    std::vector<uint32_t> mask(words);
    api_->compute_mask(m_, mask.data(), bytes);
    return mask;
  }

  /// Compute fast-forward (forced) tokens.
  /// @param max_tokens  Maximum buffer size.
  /// @return Vector of token IDs, can be empty.
  std::vector<uint32_t>
  computeFFTokens(size_t max_tokens = 100) const noexcept {
    std::vector<uint32_t> buf(max_tokens);
    int32_t n = api_->compute_ff_tokens(m_, buf.data(), max_tokens);
    if (n < 0)
      return {};
    buf.resize(static_cast<size_t>(n));
    return buf;
  }

  /// Get last error message from matcher.
  /// @return Optional string; std::nullopt if no error.
  std::optional<std::string> getError() const noexcept {
    auto e = api_->get_error(m_);
    if (!e)
      return std::nullopt;
    return std::string(e);
  }

  /// Check if EOS token is allowed now.
  bool isAccepting() const noexcept { return api_->is_accepting(m_); }

  /// Check if matcher is forced-stopped (error or stop).
  bool isStopped() const noexcept { return api_->is_stopped(m_); }

  /// Validate how many tokens can be consumed.
  /// @param tokens  List of token IDs.
  /// @return Number of tokens consumable, or -1 on error.
  int validateTokens(const std::vector<uint32_t> &tokens) const noexcept {
    return api_->validate_tokens(m_, tokens.data(), tokens.size());
  }

  /// Consume tokens.
  /// @param tokens  List of token IDs.
  /// @return 0 on success, -1 on error.
  int consumeTokens(const std::vector<uint32_t> &tokens) const noexcept {
    return api_->consume_tokens(m_, tokens.data(), tokens.size());
  }

  /// Reset matcher to initial state.
  /// @return 0 on success, -1 on error.
  int reset() const noexcept { return api_->reset ? api_->reset(m_) : -1; }

  /// Backtrack matcher by n tokens.
  /// @param n  Number of tokens to rollback.
  /// @return 0 on success, -1 on error.
  int rollback(size_t n) const noexcept {
    return api_->rollback ? api_->rollback(m_, n) : -1;
  }
};

/// C++ wrapper for a CBISON factory.
class Factory {
  cbison_factory_t f_;

public:
  /// Wrap existing factory address.
  /// @param addr  Pointer value returned from loader.
  Factory(const void *addr) noexcept
      : f_(reinterpret_cast<cbison_factory_t>((void *)addr)) {}

  /// Frees the factory.
  ~Factory() noexcept {
    if (f_)
      f_->free_factory(f_);
  }

  /// Vocabulary size.
  size_t nVocab() const noexcept { return f_->n_vocab; }

  /// Mask byte length: ceil(n_vocab/32)*4.
  size_t maskByteLen() const noexcept { return f_->mask_byte_len; }

  /// Create new matcher.
  /// @param type     Grammar type ("regex", "json", etc.).
  /// @param grammar  Grammar string.
  /// @return Matcher; m_.getError() yields error if any.
  Matcher newMatcher(const std::string &type,
                     const std::string &grammar) const noexcept {
    auto m = f_->new_matcher(f_, type.c_str(), grammar.c_str());
    return Matcher(f_, m);
  }

  /// Validate grammar without creating matcher.
  /// @param type     Grammar type.
  /// @param grammar  Grammar string.
  /// @return pair(ok, message): ok==true on success or warning.
  std::pair<bool, std::string>
  validateGrammar(const std::string &type,
                  const std::string &grammar) const noexcept {
    char buf[16 * 1024];
    int32_t r = f_->validate_grammar(f_, type.c_str(), grammar.c_str(), buf,
                                     sizeof(buf));
    if (r == 0)
      return {true, ""};
    return {r >= 0, std::string(buf)};
  }

  /// Batch compute masks.
  /// @param reqs     Vector of (Matcher*, dest_pointer) pairs.
  /// @return 0 on success, -1 on error.
  int computeMasks(const std::vector<std::pair<Matcher *, uint32_t *>> &reqs)
      const noexcept {
    size_t n = reqs.size();
    std::vector<cbison_mask_req_t> c(n);
    for (size_t i = 0; i < n; ++i) {
      c[i].matcher = reqs[i].first->get();
      c[i].mask_dest = reqs[i].second;
    }
    return f_->compute_masks ? f_->compute_masks(f_, c.data(), n) : -1;
  }
};

class Tokenizer {
  cbison_tokenizer_t t_;

public:
  /// Wrap existing tokenizer.
  Tokenizer(cbison_tokenizer_t t) noexcept : t_(t) {
    if (t_)
      t_->incr_ref_count(t_);
  }

  /// Frees the tokenizer.
  ~Tokenizer() noexcept {
    if (t_)
      t_->decr_ref_count(t_);
  }

  Tokenizer(const Tokenizer &) = delete;
  Tokenizer &operator=(const Tokenizer &) = delete;

  Tokenizer(Tokenizer &&o) noexcept : t_(o.t_) { o.t_ = nullptr; }
  Tokenizer &operator=(Tokenizer &&o) noexcept {
    if (t_)
      t_->decr_ref_count(t_);
    t_ = o.t_;
    o.t_ = nullptr;
    return *this;
  }

  cbison_tokenizer_t get() const noexcept { return t_; }

  /// Return vector of bytes for given token.
  std::vector<uint8_t> getToken(uint32_t token_id) const noexcept {
    size_t est_len = 32; // initial guess
    std::vector<uint8_t> buf(est_len);
    int n = t_->get_token(t_, token_id, buf.data(), buf.size());
    if (n < 0)
      return {};
    if (static_cast<size_t>(n) > buf.size()) {
      buf.resize(n);
      n = t_->get_token(t_, token_id, buf.data(), buf.size());
      if (n < 0)
        return {};
    }
    buf.resize(n);
    return buf;
  }

  /// Tokenize bytes, return token ids.
  std::vector<uint32_t>
  tokenizeBytes(const std::vector<uint8_t> &bytes) const noexcept {
    if (!t_->tokenize_bytes)
      return {};
    size_t est_tokens = bytes.size() + 1; // worst case
    std::vector<uint32_t> out(est_tokens);
    size_t n = t_->tokenize_bytes(t_, bytes.data(), bytes.size(), out.data(),
                                  out.size());
    out.resize(n);
    return out;
  }

  /// Tokenize string (UTF-8), returns token ids.
  std::vector<uint32_t> tokenizeString(const std::string &s) const noexcept {
    return tokenizeBytes(std::vector<uint8_t>(s.begin(), s.end()));
  }

  /// Vocabulary size.
  size_t vocabSize() const noexcept { return t_->n_vocab; }

  /// EOS token ID.
  uint32_t eosTokenId() const noexcept { return t_->eos_token_id; }

  /// Whether input to tokenize_bytes must be UTF-8.
  bool requiresUtf8() const noexcept {
    return t_->tokenize_bytes_requires_utf8;
  }
};

} // namespace cbison