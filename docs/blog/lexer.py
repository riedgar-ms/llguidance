class HashCons:

    def __init__(self) -> None:
        self.by_val = {}
        self.by_id = []

    def lookup(self, id: int):
        return self.by_id[id]

    def insert(self, val) -> int:
        if isinstance(val, list): val = tuple(val)
        r = self.by_val.get(val, None)
        if r is None:
            r = len(self.by_id)
            self.by_id += [val]
            self.by_val[val] = r
        return r


byte = int
LexemeIdx = int


class Regex:
    NO_MATCH: 'Regex'

    def deriv(self, b: byte) -> 'Regex':
        ...


LexerState = list[tuple[LexemeIdx, Regex]]

lexer_states = HashCons()
DEAD_STATE = lexer_states.insert(())

lexer_table = {}


def lexer_transition(state: int, b: byte) -> int:
    r = lexer_table.get((state, b), None)
    if r is None:
        src: LexerState = lexer_states.lookup(state)
        dst: LexerState = [(idx, s.deriv(b)) for idx, s in src
                           if s.deriv(b) != Regex.NO_MATCH]
        r = lexer_states.insert(dst)
        lexer_table[(state, b)] = r
    return r
