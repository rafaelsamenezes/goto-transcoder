#include <assert.h>

int main() {
  int a[10];
  a[2] = 42;
  assert (a[2] != 42);
}
