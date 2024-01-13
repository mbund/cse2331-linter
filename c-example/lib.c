#include "lib.h"

int do_a_thing(float param_1, float param_2) {
  float among_as = param_1 * param_2;
  float among_us = param_2 * param_2;
  int y;

  if (among_as + among_us > 100) {
    return among_as;
  } else if (among_as + among_us > 10) {
    printf("yeah");
    return among_us;
  } else {
    int k = among_as / among_us;
    y = 3;
    return k * y;
  }

  do {
    printf("Hello");
  } while (true);

  while (among_as == 2) {
    among_us = 4;
    return among_us;
  }

  return 0;
}

// double double_twelve_times(double x) {
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   x = x * 2;
//   return x;
// }
