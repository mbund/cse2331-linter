#include "lib.h"

int do_things() {
  int some_value;
  some_value = 0;

  for (int i = 0; i < 10; i++) {
    printf("Doing thing %d\n", i);
    continue;
  }

  while (some_value) {
    printf("This will never print\n");
    break;
  }

  do {
    printf("This will print once\n");
  } while (some_value);

  if (true) {
    printf("This will always print\n");
  } else if (some_value) {
    printf("A\n");
  } else if (some_value == 48) {
    printf("A\n");
  } else {
    printf("B\n");
  }

  switch (some_value) {
  case 0:
    printf("Case 0\n");
    break;
  case 1: {
    printf("Case 1\n");
    break;
  }
  case 2:
    printf("Case 2\n");
    break;
  default:
    printf("Default Case (1)\n");
    break;
  }

  switch (some_value) {
  default: {
    printf("Default Case (2)\n");
    break;
  }
  }

  return 20;
}
