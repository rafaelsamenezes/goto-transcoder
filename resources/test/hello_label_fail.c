#include <assert.h>
int main() {
  int a;
 qwe:  a = 2;
   assert(0);
 goto qwe;
 return 2;
}
