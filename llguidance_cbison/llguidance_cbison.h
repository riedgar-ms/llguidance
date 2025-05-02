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
cbison_factory_t llg_new_cbison_factory(const LlgFactoryInit *init,
                                               char *error_string,
                                               size_t error_string_len);

/**
 * Construct a new cbison factory for a given tokenizer and options.
 */
cbison_factory_t llg_new_cbison_factory_json(cbison_tokenizer_t tokenizer,
                                                    const char *options_json,
                                                    char *error_string,
                                                    size_t error_string_len);

/**
 * This for testing purposes only.
 */
cbison_tokenizer_t llg_new_cbison_byte_tokenizer(void);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* LLGUIDANCE_CBISON_H */
