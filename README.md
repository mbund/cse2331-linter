# CSE 2331 Rule Linter

Lint for some extra rules for C

- [x] No global variables
- [x] Top level functions must have a comment explaining what they do
- [x] 10 "meaningful" lines of code per function
  - Declarations and comments do not count
  - DEBUG blocks do not count
  - If statements count (and else if)
  - Else statemetns do not count
  - Opening and closing curly brackets do not count
- [ ] `DEBUG` macro
  - A debug block is guarded by `#ifdef DEBUG` and `#endif`
  - There can only be print messages starting with the function name or `ERROR: <function name>` (⚠ not implemented)
  - No code may modify any variables (⚠ not implemented)
- [x] Identifiers are all either `lower_snake_case` or `camelCase`
- [x] Macros must be `UPPER_SNAKE_CASE`

## Example

Take the following C code (`example.c` in the repo) as an example:

```c
#include <stdio.h>

#define pi 3.141592653589
#define TAU (2 * pi)

unsigned int globalOneThousand = 1000;

// Do some math
double calculate(unsigned long long x) {
  unsigned long long final_value;

  // make sure x is even
  if (x % 2 == 0) {
    final_value = x;
  } else {
    final_value = x + 1;
  }

  // multiply final value by 3/2
  final_value = final_value / 2;
  final_value = final_value * 3;

  // round final value down to nearest 100
  while (final_value % 100 != 0) {
    final_value--;
  }

  // multiply by tau for some reason?
  double actualFinalValue = final_value * TAU;

#ifdef DEBUG
  printf("The final value is %llu\n", final_value);
#endif

  printf("The actual final value is %f\n",
    actualFinalValue);

  return actualFinalValue;
}

int main() {
  double value = calculate(37);
  printf("The value is %f\n", value);

  return 0;
}
```

Then, the linter outputs the following :rocket:

```
example.c:3:9 Macro is not SCREAMING_SNAKE_CASE `#define pi 3.141592653589`
example.c:6:1 Global variable `unsigned int globalOneThousand = 1000;`
example.c:6:14 Camel case identifier contributes to case inconsistency `globalOneThousand`
example.c:9:8 Function has more than 10 lines (11) `double calculate(unsigned long long x) {`
  1) example.c:13:6 Counted if condition for 1 line `  if (x % 2 == 0) {`
  2) example.c:14:5 Counted expression for 1 line `    final_value = x;`
  3) example.c:16:5 Counted expression for 1 line `    final_value = x + 1;`
  4) example.c:20:3 Counted expression for 1 line `  final_value = final_value / 2;`
  5) example.c:21:3 Counted expression for 1 line `  final_value = final_value * 3;`
  6) example.c:24:9 Counted while condition for 1 line `  while (final_value % 100 != 0) {`
  7) example.c:25:5 Counted expression for 1 line `    final_value--;`
  8) example.c:29:10 Counted definition for 1 line `  double actualFinalValue = final_value * TAU;`
  9) example.c:35:3 Counted expression for 2 lines `  printf("The actual final value is %f\n",`
  10) example.c:38:10 Counted return for 1 line `  return actualFinalValue;`
example.c:10:22 Snake case identifier contributes to case inconsistency `final_value`
example.c:29:10 Camel case identifier contributes to case inconsistency `actualFinalValue`
example.c:41:5 Missing comment directly above function `int main() {`
```
