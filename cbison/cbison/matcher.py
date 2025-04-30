import ctypes
from .bindings import struct_cbison_factory, struct_cbison_matcher, cbison_mask_req_t, string_cast
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import numpy as np
    from numpy.typing import NDArray


class CbisonMatcher:

    def __init__(self, api: struct_cbison_factory,
                 addr: struct_cbison_matcher) -> None:
        self.api = api
        self.matcher = addr

    def __del__(self) -> None:
        if self.matcher:
            self.api.free_matcher(self.matcher)
            self.matcher = None

    def copy(self) -> 'CbisonMatcher':
        m = self.api.clone_matcher(self.matcher)
        if m is None:
            raise RuntimeError("Failed to clone matcher")
        return CbisonMatcher(self.api, m)

    def compute_mask(self) -> bytearray:
        mem = bytearray(self.api.mask_byte_len)
        self.compute_mask_into(
            (ctypes.c_uint32 * (len(mem) // 4)).from_buffer(mem))
        return mem

    def compute_mask_into(self, trg: ctypes.Array[ctypes.c_uint32]) -> int:
        return self.api.compute_mask(self.matcher, trg, len(trg) * 4)

    def compute_mask_numpy(self, bitmask: 'NDArray[np.int32]') -> int:
        assert bitmask.dtype == np.int32, "Mask must be int32"
        assert bitmask.ndim == 1, "Mask must be 1D"
        assert bitmask.flags["C_CONTIGUOUS"], "Mask must be contiguous"
        return self.unsafe_compute_mask_ptr(bitmask.ctypes.data,
                                            bitmask.size * bitmask.itemsize)

    def unsafe_compute_mask_ptr(self, trg_pointer: int,
                                trg_byte_size: int) -> int:
        ptr = ctypes.cast(trg_pointer, ctypes.POINTER(ctypes.c_uint32))
        return self.api.compute_mask(self.matcher, ptr, trg_byte_size)

    def compute_ff_tokens(self) -> list[int]:
        max_tokens = 100
        c_tokens = (ctypes.c_uint32 * max_tokens)()
        n_forced = self.api.compute_ff_tokens(self.matcher, c_tokens,
                                              max_tokens)
        if n_forced <= 0:
            return []
        if n_forced > max_tokens:
            # should not happen
            raise RuntimeError("Too many forced tokens")
        return list(c_tokens[:n_forced])

    def get_error(self) -> str:
        err = self.api.get_error(self.matcher)
        if err is None:
            return ""
        return err.decode("utf-8")

    def is_accepting(self) -> bool:
        return self.api.is_accepting(self.matcher)

    def is_stopped(self) -> bool:
        return self.api.is_stopped(self.matcher)

    def validate_tokens(self, tokens: list[int]) -> int:
        c_tokens = (ctypes.c_uint32 * len(tokens))(*tokens)
        return self.api.validate_tokens(self.matcher, c_tokens, len(tokens))

    def consume_tokens(self, tokens: list[int]) -> int:
        c_tokens = (ctypes.c_uint32 * len(tokens))(*tokens)
        return self.api.consume_tokens(self.matcher, c_tokens, len(tokens))

    def reset(self) -> int:
        return self.api.reset(self.matcher)

    def rollback(self, n: int) -> int:
        return self.api.rollback(self.matcher, n)


def _check_addr(addr: int) -> None:
    if not isinstance(addr, int) or not addr or (addr & 0x3) != 0:
        raise ValueError("Invalid address")


class CbisonFactory:

    def __init__(self, addr: int) -> None:
        _check_addr(addr)
        handle = struct_cbison_factory.from_address(addr)
        if handle.magic != 0x1bb53ed3:
            raise ValueError("Invalid magic")
        if handle.version_major != 1 or handle.version_minor < 0:
            raise ValueError("Invalid version")
        self.handle: struct_cbison_factory = handle

    def __del__(self) -> None:
        if self.handle:
            self.handle.free_factory(self.handle)
            self.handle = None  # type: ignore

    @property
    def n_vocab(self) -> int:
        return self.handle.n_vocab

    @property
    def mask_byte_len(self) -> int:
        return self.handle.mask_byte_len

    def new_matcher(self, grammar_type: str,
                    grammar: str | bytes) -> CbisonMatcher:
        if isinstance(grammar, str):
            grammar = grammar.encode("utf-8")
        elif not isinstance(grammar, bytes):
            raise TypeError("grammar must be str or bytes")

        m = self.handle.new_matcher(self.handle, grammar_type.encode("utf-8"),
                                    grammar)

        return CbisonMatcher(self.handle, m)

    def validate_grammar(self, grammar_type: str,
                         grammar: str | bytes) -> tuple[bool, str]:
        msg_buf = ctypes.create_string_buffer(16 * 1024)
        if isinstance(grammar, str):
            grammar = grammar.encode("utf-8")
        elif not isinstance(grammar, bytes):
            raise TypeError("grammar must be str or bytes")
        r = self.handle.validate_grammar(self.handle,
                                         grammar_type.encode("utf-8"), grammar,
                                         msg_buf, len(msg_buf))
        if r == 0:
            return True, ""
        message = string_cast(msg_buf)
        if message is None:
            return False, "Unknown error"
        assert isinstance(message, str)
        return r >= 0, message

    def alloc_bitmasks_numpy(self, batch: int) -> 'NDArray[np.int32]':
        import numpy as np
        n_elts = self.mask_byte_len // 4
        return np.zeros((batch, n_elts), dtype=np.int32)

    def compute_masks_numpy(self, matchers: list[tuple[CbisonMatcher, int]],
                            bitmask: 'NDArray[np.int32]') -> int:
        import numpy as np
        assert bitmask.dtype == np.int32, "Mask must be int32"
        assert bitmask.ndim == 2, "Mask must be 2D"
        batch, vocab = bitmask.shape
        n_elts = self.mask_byte_len // 4
        assert vocab == n_elts, "Mask must be of size mask_byte_len"
        assert bitmask.flags["C_CONTIGUOUS"], "Mask must be contiguous"
        trg = (cbison_mask_req_t * len(matchers))()
        ptr = bitmask.ctypes.data
        mask_len = self.mask_byte_len
        p_type = ctypes.POINTER(ctypes.c_uint32)
        for i, (m, idx) in enumerate(matchers):
            assert idx < batch, "Invalid index"
            trg[i].matcher = m.matcher
            trg[i].mask_dest = ctypes.cast(ptr + idx * mask_len, p_type)
        return self.handle.compute_masks(self.handle, trg, len(matchers))
