import ctypes
from .bindings import struct_cbison_factory, struct_cbison_matcher, cbison_mask_req_t, string_cast
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import numpy as np
    from numpy.typing import NDArray


class CbisonMatcher:
    """
    Wrapper around a CBISON matcher instance. Provides methods to query and advance
    the internal grammar state and compute token masks for constrained decoding.
    """

    def __init__(self, api: struct_cbison_factory,
                 addr: struct_cbison_matcher) -> None:
        """
        Initializes a matcher with its factory and native pointer.
        """
        self.api = api
        self.matcher = addr

    def __del__(self) -> None:
        """
        Frees the matcher when garbage collected.
        """
        if self.matcher:
            self.api.free_matcher(self.matcher)
            self.matcher = None

    def copy(self) -> 'CbisonMatcher':
        """
        Clones the matcher into a new instance.

        Returns:
            A new CbisonMatcher with the same state.

        Raises:
            RuntimeError: if cloning fails.
        """
        m = self.api.clone_matcher(self.matcher)
        if m is None:
            raise RuntimeError("Failed to clone matcher")
        return CbisonMatcher(self.api, m)

    def compute_mask(self) -> bytearray:
        """
        Allocates a bytearray and computes the token mask into it.

        Returns:
            A bytearray containing the token mask.
        """
        mem = bytearray(self.api.mask_byte_len)
        self.compute_mask_into(
            (ctypes.c_uint32 * (len(mem) // 4)).from_buffer(mem))
        return mem

    def compute_mask_into(self, trg: ctypes.Array[ctypes.c_uint32]) -> int:
        """
        Computes the token mask into the given ctypes array.

        Args:
            trg: A ctypes array of uint32 with size mask_byte_len / 4.

        Returns:
            0 on success, -1 on error.
        """
        return self.api.compute_mask(self.matcher, trg, len(trg) * 4)

    def compute_mask_numpy(self, bitmask: 'NDArray[np.int32]') -> int:
        """
        Computes the token mask into a NumPy array.

        Args:
            bitmask: A 1D, int32, C-contiguous NumPy array.

        Returns:
            0 on success, -1 on error.
        """
        assert bitmask.dtype == np.int32, "Mask must be int32"
        assert bitmask.ndim == 1, "Mask must be 1D"
        assert bitmask.flags["C_CONTIGUOUS"], "Mask must be contiguous"
        return self.unsafe_compute_mask_ptr(bitmask.ctypes.data,
                                            bitmask.size * bitmask.itemsize)

    def unsafe_compute_mask_ptr(self, trg_pointer: int,
                                trg_byte_size: int) -> int:
        """
        Calls compute_mask using a raw memory pointer.

        Args:
            trg_pointer: Pointer to writable memory.
            trg_byte_size: Number of bytes to write.

        Returns:
            0 on success, -1 on error.
        """
        ptr = ctypes.cast(trg_pointer, ctypes.POINTER(ctypes.c_uint32))
        return self.api.compute_mask(self.matcher, ptr, trg_byte_size)

    def compute_ff_tokens(self) -> list[int]:
        """
        Computes the list of forced (fast-forward) tokens from the current state.

        Returns:
            A list of token IDs, or an empty list if none.
        """
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
        """
        Returns the last error message associated with the matcher.

        Returns:
            Error string or "" if no error.
        """
        err = self.api.get_error(self.matcher)
        if err is None:
            return ""
        return err.decode("utf-8")

    def is_accepting(self) -> bool:
        """
        Checks if the matcher would allow EOS now.

        Returns:
            True if matcher is in an accepting state.
        """
        return self.api.is_accepting(self.matcher)

    def is_stopped(self) -> bool:
        """
        Checks if the matcher is in a forced-stop state.

        Returns:
            True if matcher is stopped or in an error state.
        """
        return self.api.is_stopped(self.matcher)

    def validate_tokens(self, tokens: list[int]) -> int:
        """
        Validates how many of the provided tokens can be consumed.

        Args:
            tokens: List of token IDs.

        Returns:
            Number of valid tokens, or -1 on error.
        """
        c_tokens = (ctypes.c_uint32 * len(tokens))(*tokens)
        return self.api.validate_tokens(self.matcher, c_tokens, len(tokens))

    def consume_tokens(self, tokens: list[int]) -> int:
        """
        Consumes the provided tokens.

        Args:
            tokens: List of token IDs to consume.

        Returns:
            0 on success, -1 on error.
        """
        c_tokens = (ctypes.c_uint32 * len(tokens))(*tokens)
        return self.api.consume_tokens(self.matcher, c_tokens, len(tokens))

    def reset(self) -> int:
        """
        Resets the matcher to its initial state.

        Returns:
            0 on success, -1 on error.
        """
        return self.api.reset(self.matcher)

    def rollback(self, n: int) -> int:
        """
        Rolls back the matcher state by `n` tokens.

        Args:
            n: Number of tokens to undo.

        Returns:
            0 on success, -1 on error.
        """
        return self.api.rollback(self.matcher, n)


def _check_addr(addr: int) -> None:
    if not isinstance(addr, int) or not addr or (addr & 0x3) != 0:
        raise ValueError("Invalid address")


class CbisonFactory:
    """
    Wrapper around a CBISON factory. Allows grammar validation, matcher creation,
    and batch token mask computation.
    """

    def __init__(self, addr: int) -> None:
        """
        Initializes the factory wrapper from a raw memory address.

        Raises:
            ValueError: if the address is invalid or the magic/version mismatch.
        """
        _check_addr(addr)
        handle = struct_cbison_factory.from_address(addr)
        if handle.magic != 0x1bb53ed3:
            raise ValueError("Invalid magic")
        if handle.version_major != 1 or handle.version_minor < 0:
            raise ValueError("Invalid version")
        self.handle: struct_cbison_factory = handle

    def __del__(self) -> None:
        """
        Frees the factory when garbage collected.
        """
        if self.handle:
            self.handle.free_factory(self.handle)
            self.handle = None  # type: ignore

    @property
    def n_vocab(self) -> int:
        """
        Returns the vocabulary size used by this factory.
        """
        return self.handle.n_vocab

    @property
    def mask_byte_len(self) -> int:
        """
        Returns the size of a token mask for a single sampling in bytes.
        """
        return self.handle.mask_byte_len

    def new_matcher(self, grammar_type: str,
                    grammar: str | bytes) -> CbisonMatcher:
        """
        Creates a new matcher for the given grammar.

        Args:
            grammar_type: Type of grammar (e.g., "json", "regex").
            grammar: Grammar string or bytes.

        Returns:
            A new CbisonMatcher.
        """
        if isinstance(grammar, str):
            grammar = grammar.encode("utf-8")
        elif not isinstance(grammar, bytes):
            raise TypeError("grammar must be str or bytes")

        m = self.handle.new_matcher(self.handle, grammar_type.encode("utf-8"),
                                    grammar)

        return CbisonMatcher(self.handle, m)

    def validate_grammar(self, grammar_type: str,
                         grammar: str | bytes) -> tuple[bool, str]:
        """
        Validates a grammar string without creating a matcher.

        Args:
            grammar_type: Type of grammar.
            grammar: Grammar string or bytes.

        Returns:
            Tuple (ok, message), where ok is True iff grammar is valid;
                message contains the error if not ok, or any possible warnings if ok.
        """
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
        """
        Allocates a NumPy array for holding a batch of token masks.

        Args:
            batch: Number of matchers.

        Returns:
            A (batch, mask_len) NumPy array of int32 zeros.
        """
        import numpy as np
        n_elts = self.mask_byte_len // 4
        return np.zeros((batch, n_elts), dtype=np.int32)

    def compute_masks_numpy(self, matchers: list[tuple[CbisonMatcher, int]],
                            bitmask: 'NDArray[np.int32]') -> int:
        """
        Computes token masks for a batch of matchers into a NumPy array.

        Args:
            matchers: List of (matcher, row index) tuples.
            bitmask: A (batch, mask_len) C-contiguous int32 NumPy array.

        Returns:
            0 on success, -1 on error.
        """
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
