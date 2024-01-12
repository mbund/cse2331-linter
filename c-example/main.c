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
  int x = 3;

#ifdef __ASMNAME
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

  return 0;
}

int x = 3;
int y;
