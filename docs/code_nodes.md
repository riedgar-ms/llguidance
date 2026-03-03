# Code Notes

This is a rough collection of notes taken about the code. There is minimal organisation, and in future they should probably be integrated into some of the other documents in future.

At the 50,000 ft level, `llguidance` takes a context free grammar and a prefix which conforms to that grammar, and answers the question "Which of the available LLM tokens can be added to the prefix while still conforming to the grammar?" The result is a _token mask_ which is then used to restrict generation by the LLM. There are obviously faster and slower ways to do things; `llguidance` contains many optimisations, so that it can 'beat' a forward pass of the model.

## Things about Grammars

Context free grammars (CFGs) are a superset of _regular grammars_, the latter being things which can be matched by a regular expression.  I believe that the practical difference is that CFGs allow for nesting/recursion while regular grammars do not - this is why you cannot parse HTML with a single regular expression (things breakdown when elements like `span`, `strong` and `em` are nesting inside one another). When writing out Lark rules, I _think_ this means that a regular grammar will have no cycles within the rules (there must be some relaxation of this around repeats, but I am way out of my theoretical depth here).

In the academic literature about grammars, you will find that they talk a lot about the 'tokens' within the grammar. These are *not* the tokens of an LLM. 