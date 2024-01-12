#include <stdio.h>

#include "lib.c"
#include "lib.h"

// hello there

// my comment
// thanos carpet
int foo(int x) { return double_twelve_times(x); }

#define MY_COMMENT

int a = 3;

int main() {
  int z;
  int x = 3;

#ifdef A
  if (x == 3) {
    // this is a comment
    printf("Hello");
  } else if (x == 4) {
    printf("Hi");
  } else {
    printf("no");
  }
#endif

  foo(x);

#ifdef A
  printf("X");
#endif

  while (true)
    printf("a");

  for (int i = 0; i < 10;) {
    printf("a");
    break;
    continue;
  }

  switch (x) {
  case 1:
    printf("Hi");
    break;
  case 4:
  case 2: {
    printf("Ho");

    break;
  }
  default:
    printf("default");
    break;
  }

  return 0;
}

int x = 3;
int y;
