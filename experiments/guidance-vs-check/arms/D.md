
## The pre-write check

A `codelaw hook --pre` hook judges every write before it reaches disk and
refuses one that adds an error; the refusal says which rule and how to fix it.
