#include <assert.h>

int foo() {
  return 32;
}

int main() {
  int a = foo();
  assert(a < 10);
  return 0;
}
