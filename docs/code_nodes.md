# Code Notes

This is a rough collection of notes taken about the code. There is minimal organisation, and in future they should probably be integrated into some of the other documents in future.

At the 50,000 ft level, `llguidance` takes a context free grammar and a prefix which conforms to that grammar, and answers the question "Which of the available LLM tokens can be added to the prefix while still conforming to the grammar?" The result is a _token mask_ which is then used to restrict generation by the LLM. There are obviously faster and slower ways to do things; `llguidance` contains many optimisations, so that it can 'beat' a forward pass of the model.

## Things about Grammars

Context free grammars (CFGs) are a superset of _regular grammars_, the latter being things which can be matched by a regular expression.  I believe that the practical difference is that CFGs allow for nesting/recursion while regular grammars do not - this is why you cannot parse HTML with a single regular expression (things breakdown when elements like `span`, `strong` and `em` are nesting inside one another). When writing out Lark rules, I _think_ this means that a regular grammar will have no cycles within the rules (there must be some relaxation of this around repeats, but I am way out of my theoretical depth here).

In the academic textbooks about grammars in Computer Science, you will find that they talk a lot about the 'tokens' within the grammar. These are *not* the tokens of an LLM. Where a grammar textbook would refer to tokens, we will use the word _lexeme_, and reserve 'token' for the LLM.

Parsers follow a grammar, and take in lexemes one at a time, appending them to their internal state. If a lexeme does not match any of the grammar's rules (given the current state), then the parser will reject it.

Parsers can be in a few different high-level states.
TODO: _Get the exact names which `llguidance` uses_.
For example, consider the regex `red|green|blue`.
If we have already added the letters `r` and `e` then the parser will be in a valid state, but it will not be completed - not until we add `d` to complete the word.
At that point, the parser cannot accept any more letters.
Next, consider `\w*`.
The parser will be happy even with an empty string (it will be complete at any time), but given any valid string, the parser will also always accept more letters (or numbers).
Finally consider `.*A`, where we can add as many characters as we want, but have to end with `A`.
This means that the parser will always accept new characters until we provide the `A`, but that it won't be completed until we do so.

## Optimisations

As alluded above, `llguidance` makes use of a number of optimisations, since running a full parse for each candidate token is incredibly slow.

### Toktries

Toktries are fundamentally about "If `t` is not a valid extension of the current LLM generation, then neither are `to`, `the` or `tree` since they all begin with that letter."
If a large number of tokens share a common prefix, and that prefix is not currently acceptable, then we don't need to check all the individual tokens, saving us a lot of time.
When the set of allowable tokens is highly constrained, this insight allows us to find that set very quickly.

The Toktrie library arranges the tokens into a tree structure, with the property that all the children of a node start with that prefix.
Concretely, `t` might have children `ta` and `th`, with `ta` in turn having children `tap` and `tar`.
When building its token mask, `llguidance` will walk down the tree until it hits a node which can't be appended to the parser.
It then knows that it doesn't need to go any further, and can consider the next sibling node.

To build the tree, Toktrie needs the full list of tokens.
Extracting the list from a Hugging Face model is straightforward - look for the `tokenizer.json` file.
LlamaCpp models require a more vigorous approach.

### Slicers

_I am a little more hazy on this_. Slicers are an optimisation for when the generation is relatively unconstrained.
Consider a JSON string: once we know we're inside the string, any token which doesn't contain `"` (which would terminate the string), or `\` or a special character (which can require special handling) is acceptable - and we know that before we start.
This means we can pre-compute _slices_ of tokens which will always be acceptable in that situation.
We can then focus computation time on the remaining tokens _which may still be acceptable_ - for example if there is a token `",` we can add it to the string, but it will terminate the string and take us to the next part of the object.
That is, assuming there is more of the object to be generated - if the string is the last property in said object then `",` would not be valid but `"}` would be.
Hence the need to expend more computational resources on these tokens.

As a further optimisation, slicers can be nested as being of various lengths.
There may be a length constraint on the string, so we can make slices of lengths 2, 4 and 6.
If there are only three characters left in the length constraint, then we don't need to bother with the slices of length 4 and 6.