#ifndef LLGUIDANCE_CBISON_H
#define LLGUIDANCE_CBISON_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include "llguidance.h"
#include "cbison_api.h"

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Construct a new cbison factory for a given tokenizer.
 */
cbison_factory_t llg_cbison_new_factory_init(const LlgFactoryInit *init,
                                                    char *error_string,
                                                    size_t error_string_len);

/**
 * Construct a new CBISON factory for a given tokenizer and options.
 * The reference count of the tokenizer is incremented (until the factory is freed).
 */
cbison_factory_t llg_cbison_new_factory(cbison_tokenizer_t tokenizer,
                                               const char *options_json,
                                               char *error_string,
                                               size_t error_string_len);

/**
 * This for testing purposes only.
 */
cbison_tokenizer_t llg_cbison_new_byte_tokenizer(void);

/**
 * Construct a new cbison tokenizer from a JSON string representing a HuggingFace
 * fast tokenizer (tokenizer.json file).
 * You can override the vocab size and the end of sentence token id.
 * Keep them at 0 and -1 respectively to use the default values from the tokenizer.
 */
cbison_tokenizer_t llg_cbison_new_hf_tokenizer(const char *tokenizer_json,
                                                      const char *options_json,
                                                      char *error_string,
                                                      size_t error_string_len);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* LLGUIDANCE_CBISON_H */
