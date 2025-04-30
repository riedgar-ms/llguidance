import llguidance
from .matcher import CbisonMatcher, CbisonFactory
from typing import TYPE_CHECKING


def main():
    t = llguidance.LLTokenizer("byte")
    f_addr = t.copy_as_cbison_factory()

    f = CbisonFactory(f_addr)

    ok, msg = f.validate_grammar("json", '{}')
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
    mask = m2.compute_mask()
    print(mask)

    l = m2.compute_ff_tokens()
    assert len(l) == 0

    m.rollback(1)

    mask = f.alloc_bitmasks_numpy(3)
    f.compute_masks_numpy([(m, 0), (m2, 1)], mask)
    print(mask)


main()
