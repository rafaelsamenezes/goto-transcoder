#include <assert.h>
struct obj {
  int a;
  int b;
};

int main() {
  struct obj qwe = {.a=1,.b=2};
  assert(qwe.a); 
}
