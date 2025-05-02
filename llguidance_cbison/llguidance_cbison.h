#ifndef LLGUIDANCE_H
#define LLGUIDANCE_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Construct a new cbison factory for a given tokenizer.
 * # Safety
 * This function should only be called from C code.
 */
const LlgCbisonFactory *llg_new_cbison_factory(const LlgFactoryInit *init,
                                               char *error_string,
                                               size_t error_string_len);

/**
 * Construct a new cbison factory for a given tokenizer and options.
 * # Safety
 * This function should only be called from C code.
 */
const LlgCbisonFactory *llg_new_cbison_factory_json(CbisonTokenizer *tokenizer,
                                                    const char *options_json,
                                                    char *error_string,
                                                    size_t error_string_len);

/**
 * This for testing purposes only.
 */
const LlgCbisonTokenizer *llg_new_cbison_byte_tokenizer(void);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* LLGUIDANCE_H */
