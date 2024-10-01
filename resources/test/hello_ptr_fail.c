#include <assert.h>

int main() {
  int a;
  int *ptr = &a;
  assert (*ptr != a);
}
