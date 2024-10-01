#include <assert.h>
struct obj {
  int a;
  int b;
};

int main() {
  struct obj qwe;
  qwe.a = 1;
  assert(qwe.a); 
}
