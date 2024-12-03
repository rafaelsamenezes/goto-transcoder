#include <assert.h>
struct obj {
  unsigned arr[4];
};

int main() {
  struct obj qwe = { .arr={1,2,3,4}};
  assert(qwe.arr[2] == 2);
}
