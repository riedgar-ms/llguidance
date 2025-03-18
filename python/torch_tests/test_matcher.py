from typing import Any, Dict, List, Tuple
import llguidance
from llguidance.numpy import fill_next_token_bitmask_par, allocate_token_bitmask
import pytest
from numpy.typing import NDArray
import numpy as np

_tokenizer = None


def tokenizer() -> llguidance.LLTokenizer:
    global _tokenizer
    if _tokenizer is None:
        _tokenizer = llguidance.LLTokenizer("byte")
    return _tokenizer


def matcher(grm: str) -> llguidance.LLMatcher:
    return llguidance.LLMatcher(tokenizer(), grm, log_level=1)


def check_one_grammar(grm: str, s: str, passing: bool) -> None:
    # print("Checking", repr(s))
    interp = matcher(grm)
    final_reject = False
    if s.startswith("FINAL_REJECT:"):
        final_reject = True
        s = s[len("FINAL_REJECT:"):]
    tokens = tokenizer().tokenize_str(s)
    for i, t in enumerate(tokens):
        next_tokens = tokens[i:]
        if passing or final_reject:
            assert interp.validate_tokens(next_tokens) == len(next_tokens)
        else:
            assert interp.validate_tokens(next_tokens) < len(next_tokens)
        mask = interp.compute_logit_bias()
        if mask[t] == 0:
            if passing or final_reject:
                print("Token not in mask",
                      tokenizer().dbg_tokens(tokens[:i + 1]), repr(s))
                assert False
            return
        else:
            assert mask[t] == 200
        interp.consume_token(t)
    if final_reject:
        if interp.is_accepting():
            print("Expected to fail at the end", s)
            assert False
        else:
            return
    if not passing:
        print("Expected to fail", s)
        assert False
    assert interp.is_accepting()


def check_grammar(grm: str, passing: List[str], failing: List[str]) -> None:
    for s in passing:
        check_one_grammar(grm, s, True)
    for s in failing:
        check_one_grammar(grm, s, False)


def test_json() -> None:
    grm = llguidance.LLMatcher.grammar_from_json_schema(
        {"type": "object"}, {"whitespace_flexible": False})
    check_grammar(grm, ["{}", '{"foo":1}'], ["FINAL_REJECT:{", " {}", "{ }"])

    grm = llguidance.LLMatcher.grammar_from_json_schema({
        "type": "object",
        "properties": {
            "foo": {
                "type": "integer"
            }
        },
        "required": ["foo"]
    })
    check_grammar(grm, ['{"foo":1}', '{"foo":1,"bar":2}', '{ "foo" : 1 }'],
                  ["{}", "FINAL_REJECT:{", ' {"foo":1}', '{"bar":1}'])


def test_lark() -> None:
    check_grammar(
        'start: /.../ "abc" /.../',
        [
            "abcabcabc",
            "aaaabcccc",
            # NOTE: Also ensures that multi-byte characters still count as a single character
            "🔵🟠✅abc❌🟠🔵",
        ],
        [
            "aaabcccc",
            "aaaaabcccc",
            "FINAL_REJECT:aaaabccc",
            "aaaabccccc",
            "🔵🟠✅❌abc❌✅🟠🔵",
            "🔵🟠abc🟠🔵",
        ],
    )


def test_lark_syntax() -> None:
    with pytest.raises(ValueError, match="no_such_rule"):
        matcher('start: /.../ no_such_rule')


def test_slices() -> None:
    t = tokenizer()
    gen_slices = llguidance.LLTokenizer.general_slices()
    assert len(gen_slices) > 0
    json_slices = llguidance.LLTokenizer.json_slices()
    assert len(json_slices) > 0
    t2 = t.with_slices(json_slices)
    assert t.tokenize_str("Hello, world!") == t2.tokenize_str("Hello, world!")


def mask_has(mask: NDArray[np.int32], t: int) -> bool:
    return mask[t // 32] & (1 << (t % 32)) != 0


def test_par_errors() -> None:
    t = tokenizer()
    exec = llguidance.LLExecutor()
    g0 = matcher(r"start: /[a-zA-Z ]*/")
    g1 = matcher(r"start: /[0-9 ]*/")
    mask = allocate_token_bitmask(3, t.vocab_size)

    with pytest.raises(AssertionError, match="count mismatch"):
        fill_next_token_bitmask_par(exec, [g0, g1], mask)

    with pytest.raises(RuntimeError, match="Already borrowed"):
        fill_next_token_bitmask_par(exec, [g0, g1, g1], mask)

    # should be OK
    fill_next_token_bitmask_par(exec, [g0, g1], mask[0:2, :])
    t_a = t.tokenize_str("a")[0]
    t_1 = t.tokenize_str("1")[0]
    assert mask_has(mask[0, :], t_a)
    assert not mask_has(mask[0, :], t_1)
    assert not mask_has(mask[1, :], t_a)
    assert mask_has(mask[1, :], t_1)
