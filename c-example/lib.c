#include "lib.h"

int do_a_thing(float a, float b) {
  float x = a * b;
  float z = b * b;

  if (x + z > 100) {
    return x;
  } else if (x + z > 10) {
    printf("yeah");
    return z;
  } else {
    int k = x / z;
    return k;
  }

  while (x == 2) {
    z = 4;
    return z;
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
