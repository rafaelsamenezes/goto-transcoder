#include <assert.h>
struct obj {
  int a;
  int b;
};

int main() {
  struct obj qwe;
  qwe.a = 0;
  assert(qwe.a); 
}
