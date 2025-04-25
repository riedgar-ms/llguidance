#ifndef CBISON_API_H
#define CBISON_API_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define CBISON_MAGIC 0x1bb53ed3
#define CBISON_VERSION_MAJOR 1
#define CBISON_VERSION_MINOR 0

typedef struct cbison_matcher *cbison_matcher_t;
typedef struct cbison_api *cbison_api_t;

/**
 * This represents the API for a constraint engine that is specialized
 * for a given tokenizer.
 *
 * We currently do not cover creation APIs for these here.
 */
struct cbison_api {
  /**
   * Always CBISON_MAGIC (0x1bb53ed3)
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
   * Check if given grammar is valid.
   * This is about twice as fast as creating a matcher (which also validates).
   * See matcher_new() for the grammar format.
   * Returns 0 on success and -1 on error and 1 on warning.
   * The error message or warning is written to message, which is message_len
   * bytes long. It's always NUL-terminated.
   */
  int32_t (*validate_grammar)(cbison_api_t api, const char *grammar_type,
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
  cbison_matcher_t (*new_matcher)(cbison_api_t api, const char *grammar_type,
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
  cbison_matcher_t (*clone_matcher)(cbison_matcher_t matcher);
};

#endif // CBISON_API_H