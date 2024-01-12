# CSE 2331 Rule Linter

Lint for some extra rules for C

- No global variables
- Top level functions must have a comment explaining what they do
- 10 "meaningful" lines of code per function
  - Declarations and comments do not count
  - DEBUG blocks do not count
  - If statements count (and else if)
  - Else statemetns do not count
  - Opening and closing curly brackets do not count
- `DEBUG` macro
  - A debug block is guarded by `#ifdef DEBUG` and `#endif`
  - There can only be print messages starting with the function name or `ERROR: <function name>`
  - No code may modify any variables
