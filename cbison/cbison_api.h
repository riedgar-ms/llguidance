#ifndef CBISON_API_H
#define CBISON_API_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// Factory and Tokenizer are versioned separately, as they are typically
// provided by different codebases.

#define CBISON_FACTORY_MAGIC 0x1bb53ed3
#define CBISON_FACTORY_VERSION_MAJOR 1
#define CBISON_FACTORY_VERSION_MINOR 0

#define CBISON_TOKENIZER_MAGIC 0xff79e338
#define CBISON_TOKENIZER_VERSION_MAJOR 1
#define CBISON_TOKENIZER_VERSION_MINOR 0

#ifndef CBISON_SKIP_STRUCTS
typedef struct cbison_matcher *cbison_matcher_t;
typedef struct cbison_factory *cbison_factory_t;
typedef struct cbison_tokenizer *cbison_tokenizer_t;
#endif

// This type is used when a value is returned or stored in a struct
// (think of cbison_matcher_t as cbison_matcher& and cbison_matcher_ptr_t as
// cbison_matcher* in C++ sense).
typedef cbison_matcher_t cbison_matcher_ptr_t;
typedef cbison_tokenizer_t cbison_tokenizer_ptr_t;

typedef struct cbison_mask_req cbison_mask_req_t;

/**
 * Typically provided by the inference engine to the structured output
 * engine.
 */
struct cbison_tokenizer {
  /**
   * Always CBISON_TOKENIZER_MAGIC (0xff79e338)
   */
  uint32_t magic;

  /**
   * The value is implementation-specific.
   */
  uint32_t impl_magic;

  /**
   * The major version of the API.
   * Major version is incremented when the API changes in a
   * backward-incompatible way.
   */
  uint32_t version_major;

  /**
   * The minor version of the API.
   * Minor version is incremented when the API changes in a backward-compatible
   * way.
   */
  uint32_t version_minor;

  /**
   * The number of tokens in the vocabulary.
   */
  size_t n_vocab;

  /**
   * The id for end-of-sequence token.
   */
  uint32_t eos_token_id;

  /**
   * Indicates of the tokenize_bytes() function requires the input bytes
   * to be valid UTF-8 (often the case).
   */
  bool tokenize_bytes_requires_utf8;

  uint32_t reserved_hd[6];

  /**
   * Get bytes for the given token.
   * Returns -1 on error (token_id >= n_vocab), and number of bytes in the token
   * on success (which can be larger than bytes_len). Writes at most bytes_len
   * bytes to bytes; they are *not* NUL-terminated.
   */
  int (*get_token)(cbison_tokenizer_t api, uint32_t token_id, uint8_t *bytes,
                   size_t bytes_len);

  /**
   * Returns 0 if the token is a normal text token (e.g. "hello", "\n"),
   * 1 if the token is a special token (e.g. <|endoftext|>, <|tool|>, etc.).
   * and -1 on error (token_id >= n_vocab).
   */
  int (*is_special_token)(cbison_tokenizer_t api, uint32_t token_id);

  /**
   * Tokenize the given bytes and return the tokens.
   * Always returns the number of tokens that would be written to output_tokens
   * if output_tokens_len was large enough.
   *
   * This can be omitted, resulting in compute_ff_tokens() always returning an
   * empty vector.
   *
   * If provided, this function must be thread-safe and reentrant.
   */
  size_t (*tokenize_bytes)(cbison_tokenizer_t api, const uint8_t *bytes,
                           size_t bytes_len, uint32_t *output_tokens,
                           size_t output_tokens_len);

  /**
   * Increment the reference count of the tokenizer.
   * All functions allocating tokenizers set the reference count to 1.
   * This can be no-op if the tokenizer is never freed.
   */
  void (*incr_ref_count)(cbison_tokenizer_ptr_t api);

  /**
   * Decrement the reference count of the tokenizer.
   * If the reference count reaches 0, the tokenizer is freed.
   * This can be no-op if the tokenizer is never freed.
   */
  void (*decr_ref_count)(cbison_tokenizer_ptr_t api);

  void *reserved_ptr[16];
};

/**
 * C Binary Interface for Structured Output Negotiation (CBISON)
 *
 * This represents a factory for matchers, that is specialized
 * for a given tokenizer.
 *
 * We currently do not cover creation APIs for these here.
 */
struct cbison_factory {
  /**
   * Always CBISON_FACTORY_MAGIC (0x1bb53ed3)
   */
  uint32_t magic;

  /**
   * The value is implementation-specific.
   */
  uint32_t impl_magic;

  /**
   * The major version of the API.
   * Major version is incremented when the API changes in a
   * backward-incompatible way.
   */
  uint32_t version_major;

  /**
   * The minor version of the API.
   * Minor version is incremented when the API changes in a backward-compatible
   * way.
   */
  uint32_t version_minor;

  /**
   * The number of tokens in the vocabulary.
   */
  size_t n_vocab;

  /**
   * The size of token mask in bytes.
   * It equals (n_vocab + 31) / 32 * 4.
   */
  size_t mask_byte_len;

  /**
   * The id for end-of-sequence token.
   */
  uint32_t eos_token_id;

  uint32_t reserved_hd[7];

  /**
   * Free the factory.
   */
  void (*free_factory)(cbison_factory_t api);

  /**
   * Check if given grammar is valid.
   * This is about twice as fast as creating a matcher (which also validates).
   * See matcher_new() for the grammar format.
   * Returns 0 on success and -1 on error and 1 on warning.
   * The error message or warning is written to message, which is message_len
   * bytes long. It's always NUL-terminated.
   */
  int32_t (*validate_grammar)(cbison_factory_t api, const char *grammar_type,
                              const char *grammar, char *message,
                              size_t message_len);

  /**
   * Create a new matcher from the given grammar.
   * Always returns a non-null value. Call get_error() on the result
   * to check for errors.
   * The grammar is of different format, depending on grammar_type:
   * - "regex" - grammar is regular expression
   * - "json" or "json_schema" - grammar is (stringifed) JSON schema
   * - "json_object" - equivalent to JSON schema: {"type":"object"}; grammar is
   * ignored
   * - "lark" - grammar in (a variant of) Lark syntax
   * - "llguidance" or "guidance" - grammar is a list of Lark or JSON schemas in
   * JSON format
   */
  cbison_matcher_ptr_t (*new_matcher)(cbison_factory_t api,
                                      const char *grammar_type,
                                      const char *grammar);

  /**
   * Get the error message from the matcher.
   * The error message is always NUL-terminated.
   * Returns NULL if there is no error.
   */
  const char *(*get_error)(cbison_matcher_t matcher);

  /**
   * Compute the set of allowed tokens for the current state.
   * The result is written to mask_dest.
   * mask_byte_len must be equal to the one set in this struct.
   * Returns 0 on success and -1 on error.
   */
  int32_t (*compute_mask)(cbison_matcher_t matcher, uint32_t *mask_dest,
                          size_t mask_byte_len);

  /**
   * Advance the matcher by consuming the given tokens.
   * Returns 0 on success and -1 on error.
   */
  int32_t (*consume_tokens)(cbison_matcher_t matcher, const uint32_t *tokens,
                            size_t n_tokens);

  /**
   * Check if the grammar can fully accept the input now (ie., if it will allow
   * EOS token).
   */
  bool (*is_accepting)(cbison_matcher_t matcher);

  /**
   * Check if the matcher will force EOS token.
   * This returns true also in error state, as that is a forced stop.
   */
  bool (*is_stopped)(cbison_matcher_t matcher);

  /**
   * Check how many tokens can be consumed from the given tokens.
   * Returns the number of tokens that can be consumed, or -1 on error.
   */
  int32_t (*validate_tokens)(cbison_matcher_t matcher, const uint32_t *tokens,
                             size_t n_tokens);

  /**
   * Compute the fast-forward (forced) tokens for the current state.
   * The result is written to output.
   * Returns the number of tokens written to output (which can be 0) or -1 on
   * error.
   * This is optional (can be NULL).
   */
  int32_t (*compute_ff_tokens)(cbison_matcher_t matcher, uint32_t *output,
                               size_t output_len);

  /**
   * Free the matcher.
   */
  void (*free_matcher)(cbison_matcher_t matcher);

  /**
   * Backtracks the matcher states by num_tokens.
   * Returns 0 on success and -1 on error.
   * This is optional (can be NULL).
   */
  int32_t (*rollback)(cbison_matcher_t matcher, size_t num_tokens);

  /**
   * Resets the matcher to the initial state.
   * A matcher in error state cannot be reset.
   * Returns 0 on success and -1 on error.
   * This is optional (can be NULL).
   */
  int32_t (*reset)(cbison_matcher_t matcher);

  /**
   * Clone the matcher.
   * This is optional (can be NULL).
   */
  cbison_matcher_ptr_t (*clone_matcher)(cbison_matcher_t matcher);

  /**
   * Compute masks for a number of matchers.
   * The masks can be computed in parallel, and the function returns only
   * when all of them are computed.
   * The behavior is undefined if any matcher is specified more than once,
   * or if other operations are performed on the matchers while this function is
   * running.
   * This is optional (can be NULL).
   */
  int32_t (*compute_masks)(cbison_factory_t api, cbison_mask_req_t *reqs,
                           size_t n_reqs);

  void *reserved_ptr[16];
};

/**
 * Represents a single request for a mask.
 */
struct cbison_mask_req {
  /**
   * The matcher to compute the mask for.
   */
  cbison_matcher_ptr_t matcher;

  /**
   * Where to write the mask.
   * This must point to a buffer of size mask_byte_len bytes.
   */
  uint32_t *mask_dest;
};

#endif // CBISON_API_H