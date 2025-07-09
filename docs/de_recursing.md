# De-Recursing Grammars

This is a cookbook of examples to help in removing recursion where possible from grammars (see [Syntax](./syntax.md) for more details).
The examples below will generally already be left-recursive.

## Simple lists

```lark
item_list : item
    | item_list item
```
can become
```lark
item_list : item+
```

## Lists with Delimiters

```lark
sep_list : item
    | item_list SEP item
```