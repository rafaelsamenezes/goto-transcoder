#include <assert.h>
int foo() {
  assert(0);
}

int main() {
  foo();
  return 0;
}
