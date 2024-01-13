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
