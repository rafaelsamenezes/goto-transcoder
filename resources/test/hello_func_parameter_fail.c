#include <assert.h>

int id_foo(int i) {
  assert(i > 24);
  return i;
}

int main() {
  id_foo(30);
  int a = 42;
  assert(a != id_foo(a));
  return 0;
}
