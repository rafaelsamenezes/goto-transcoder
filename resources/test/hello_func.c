#include <assert.h>

int foo() {
  return 32;
}

int main() {
  int a = foo();
  assert(a < 100);
  return 0;
}
