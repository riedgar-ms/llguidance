import llguidance
from .matcher import CbisonMatcher, CbisonFactory, CbisonTokenizer
from typing import TYPE_CHECKING
import numpy as np


def test_factory():
    t = llguidance.LLTokenizer("byte")
    f_addr = t.copy_as_cbison_factory()

    f = CbisonFactory(f_addr)

    ok, msg = f.validate_grammar("json", '{}')
    assert ok and not msg

    ok, msg = f.validate_grammar("json", b'{}')
    assert ok and not msg

    ok, msg = f.validate_grammar("json", 'foobar')
    assert not ok
    assert "expected ident" in msg

    m = f.new_matcher("json", 'foobar')
    assert "expected ident" in m.get_error()

    m = f.new_matcher("json", '{}')
    assert not m.get_error()
    assert not m.is_accepting()

    tokens = t.tokenize_str('{"a":abc}')
    n_valid = m.validate_tokens(tokens)
    assert n_valid < len(tokens)

    tokens = t.tokenize_str('{"a":12}')
    n_valid = m.validate_tokens(tokens)
    assert n_valid == len(tokens)
    assert not m.is_accepting()
    m.consume_tokens(tokens)
    assert m.is_accepting()
    assert m.is_stopped()

    m.rollback(3)
    m2 = m.copy()
    assert not m.is_accepting()
    assert not m.is_stopped()
    m.consume_tokens(tokens[-3:])
    assert m.is_accepting()
    assert m.is_stopped()
    m.reset()
    assert not m.is_accepting()
    assert not m.is_stopped()
    m.consume_tokens(tokens)
    assert m.is_accepting()
    assert m.is_stopped()

    assert not m2.is_accepting()
    assert not m2.is_stopped()
    m2.consume_tokens(tokens[-3:])
    assert m2.is_accepting()
    assert m2.is_stopped()

    m2.rollback(1)
    mask2 = m2.compute_mask()
    #print(mask2)

    l = m2.compute_ff_tokens()
    assert len(l) == 0

    m.rollback(1)

    mask = f.alloc_bitmasks_numpy(3)
    f.compute_masks_numpy([(m, 0), (m2, 2)], mask)
    #print(mask)

    mask2_np = np.frombuffer(mask2, dtype=np.int32)
    assert (mask2_np == mask[0, :]).all()
    assert (mask2_np == mask[2, :]).all()
    assert mask[1, :].all() == 0


def test_tokenizer():
    ll_t = llguidance.LLTokenizer("byte")
    addr = ll_t.copy_as_cbison_tokenizer()
    t = CbisonTokenizer(addr)
    assert 200 < t.n_vocab < 300
    assert t.is_special_token(t.eos_token_id)
    assert t.get_token(t.eos_token_id) == b"<|end|>"
    tokens = t.tokenize_bytes(b"abc")
    assert len(tokens) == 3


def main():
    test_factory()
    test_tokenizer()
    print("All tests passed!")


if __name__ == "__main__":
    main()
