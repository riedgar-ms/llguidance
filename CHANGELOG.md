### Changelog

All notable changes to this project will be documented in this file. Dates are displayed in UTC.

If a release doesn't introduce any interesting changes (build fixes etc.), it's skipped.


#### Unreleased


#### [v0.7.10](https://github.com/microsoft/llguidance/compare/v0.7.9...v0.7.10)

> 25 March 2025

- add `llg_macher_*()` functions to C interface [`#145`](https://github.com/microsoft/llguidance/pull/145)
- always pass validation of grammars with special tokens without tokenizer [`0892a2a`](https://github.com/microsoft/llguidance/commit/0892a2adb5c8d818c025fe554bd67f05a5770aa7)

#### [v0.7.9](https://github.com/microsoft/llguidance/compare/v0.7.8...v0.7.9)

> 24 March 2025

- improve Python `LLMatcher.validate_grammar()` [`6b5f5ed`](https://github.com/microsoft/llguidance/commit/6b5f5eda7ca85ae2ca9a76c3813a0162a8b99b45)

#### [v0.7.8](https://github.com/microsoft/llguidance/compare/v0.7.5...v0.7.8)

> 21 March 2025

- Stabilize JSON schema property order [`#134`](https://github.com/microsoft/llguidance/pull/134)

#### [v0.7.5](https://github.com/microsoft/llguidance/compare/v0.7.4...v0.7.5)

> 21 March 2025

- add toktrie_hf_downloader crate [`11bea00`](https://github.com/microsoft/llguidance/commit/11bea00ecd1ef3c4a8970c1748db829e0c8a14de)
- use hf tokenizers library in python ext [`61728be`](https://github.com/microsoft/llguidance/commit/61728be47828525e959f6db226a0f17a783442bc)
- add LLTokenizer.tokenize_partial [`893cedf`](https://github.com/microsoft/llguidance/commit/893cedf614e234bd86bf01a99772d846b6ea884b)

#### [v0.7.4](https://github.com/microsoft/llguidance/compare/v0.7.3...v0.7.4)

> 19 March 2025

- fix gbnf parsing [`e5828b8`](https://github.com/microsoft/llguidance/commit/e5828b8a7a2fffaa9cf1aa2619c603a3d4ec7e17)

#### [v0.7.3](https://github.com/microsoft/llguidance/compare/v0.7.2...v0.7.3)

> 20 March 2025

- add LLMatcher.validate_grammar(); make it never raise [`0f8ec60`](https://github.com/microsoft/llguidance/commit/0f8ec6088a28eda13c2dd3d537733c0648e00cb3)
- add LLMatcher.reset() [`6a70aa7`](https://github.com/microsoft/llguidance/commit/6a70aa7efa8121fcd1865cefa9998926852eee25)

#### [v0.7.2](https://github.com/microsoft/llguidance/compare/v0.7.1...v0.7.2)

> 18 March 2025

- don't go into error state on final EOS [`1f0f21d`](https://github.com/microsoft/llguidance/commit/1f0f21d41fe88427d065b09414047d76b8b32041)

#### [v0.7.1](https://github.com/microsoft/llguidance/compare/v0.7.0...v0.7.1)

> 18 March 2025

- enable mypy in CI [`#140`](https://github.com/microsoft/llguidance/pull/140)
- add py.typed for annotations information [`#139`](https://github.com/microsoft/llguidance/pull/139)
- fix clippy warnings

#### [v0.7.0](https://github.com/microsoft/llguidance/compare/v0.6.31...v0.7.0)

> 7 March 2025

- remove JSON-based grammar serialization [`#133`](https://github.com/microsoft/llguidance/pull/133)

#### [v0.6.31](https://github.com/microsoft/llguidance/compare/v0.6.29...v0.6.31)

> 5 March 2025

- fix https://github.com/guidance-ai/guidance/issues/1131 - backtracking+prompt healing [`#1131`](https://github.com/guidance-ai/guidance/issues/1131)
- optimize substring [`9950600`](https://github.com/microsoft/llguidance/commit/9950600f46e433b4c42506f8816f61cee331774f)


#### [v0.6.29](https://github.com/microsoft/llguidance/compare/v0.6.28...v0.6.29)

> 25 February 2025

- [JSON] "x-guidance" JsonCompileOptions [`#130`](https://github.com/microsoft/llguidance/pull/130)

#### [v0.6.28](https://github.com/microsoft/llguidance/compare/v0.6.27...v0.6.28)

> 21 February 2025

- support for rollback() [`#126`](https://github.com/microsoft/llguidance/pull/126)
- allow lexer to produce alternative lexemes [`#124`](https://github.com/microsoft/llguidance/pull/124)

#### [v0.6.27](https://github.com/microsoft/llguidance/compare/v0.6.26...v0.6.27)

> 18 February 2025

- fix #122: captures with nullable symbols [`#122`](https://github.com/microsoft/llguidance/issues/122)

#### [v0.6.26](https://github.com/microsoft/llguidance/compare/v0.6.25...v0.6.26)

> 14 February 2025

- Extend Token::Number to match floats in scientific notation [`#121`](https://github.com/microsoft/llguidance/pull/121)
- native [suffix=...] support [`6d648c7`](https://github.com/microsoft/llguidance/commit/6d648c748bed4d83db28ed96ea87ad40ea51bc7e)

#### [v0.6.25](https://github.com/microsoft/llguidance/compare/v0.6.16...v0.6.25)

> 12 February 2025

- update referencing to 0.29.0 [`#118`](https://github.com/microsoft/llguidance/pull/118)
- Allow passing string for `capture_name` in lark syntax [`#119`](https://github.com/microsoft/llguidance/pull/119)

Plus a few releases messing with, deps, unsafe code cleanup.

#### [v0.6.16](https://github.com/microsoft/llguidance/compare/v0.6.15...v0.6.16)

> 6 February 2025

- Port over guidance's 'substring' [`#116`](https://github.com/microsoft/llguidance/pull/116)
- add %regex {...} syntax in lark for substrings [`b5ab086`](https://github.com/microsoft/llguidance/commit/b5ab0861e819b6e9221ef0aed3fcc827d6bad316)

#### [v0.6.15](https://github.com/microsoft/llguidance/compare/v0.6.14...v0.6.15)

> 4 February 2025

- move gbnf_to_lark to llguidance python pkg [`53134f1`](https://github.com/microsoft/llguidance/commit/53134f1befc6b6019bc88406e21b51c901943b51)
- add LLExecutor and fill_next_token_bitmask_par() [`ba4b917`](https://github.com/microsoft/llguidance/commit/ba4b9175b8d6c5445e1c0bcc8d5ef8e62b6cf73c)

#### [v0.6.14](https://github.com/microsoft/llguidance/compare/v0.6.13...v0.6.14)

> 3 February 2025

- add llguidance.numpy and llguidance.mlx submodules [`c627a39`](https://github.com/microsoft/llguidance/commit/c627a39689c9147fe7b072e5075960d16d43fc73)

#### [v0.6.13](https://github.com/microsoft/llguidance/compare/v0.6.12...v0.6.13)

> 3 February 2025

- add llguidance.torch and llguidance.hf submodules [`3fcdb1d`](https://github.com/microsoft/llguidance/commit/3fcdb1d93af076bbc8f1b3bef6fa9ead22b3e959)

#### [v0.6.12](https://github.com/microsoft/llguidance/compare/v0.6.11...v0.6.12)

> 31 January 2025

- fixes for numeric tokens [`b7c9970`](https://github.com/microsoft/llguidance/commit/b7c99709a9cb7f7a8a3c4716092e4d94fae2ff2c)
- make capture explicit in lark syntax [`2a57678`](https://github.com/microsoft/llguidance/commit/2a57678d9397e8be54cb0c9f14c4270604f8e1a5)

