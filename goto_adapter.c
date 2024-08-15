/* Goto Adapter
Copyright (C) 2024 rafael

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// This project is intended to be a generic parser for the irep used by BMCs
// such as ESBMC and CBMC. The format itself is not stable and it consists in:
// 1. An string ID
// 2. Sub-string sub
// 3. Named sub-string string->string

/**********/
/* BASICS */
/**********/

#include <stddef.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>
#include <stdio.h>

// Vector structure

typedef struct vector vector;

#ifdef __ESBMC
#define VECTOR_INIT_LENGTH 10000
#else
#define VECTOR_INIT_LENGTH 20
#endif
#define VECTOR_FACTOR 2

struct vector {
  char *_buf;
  size_t length;
  size_t _capacity;
  size_t _unit_length;
};

vector vector_init(const size_t unit_length) {
  vector v;
  v._unit_length = unit_length;
  v.length = 0;
  v._capacity = VECTOR_INIT_LENGTH;
  v._buf = malloc(unit_length * VECTOR_INIT_LENGTH);

  return v;
}

void vector_destroy(vector * const v) {
  if (v->_buf != NULL)
    free(v->_buf);
}

void vector_reserve(vector * const v, const size_t quantity) {
  assert(v->_capacity < quantity); // We should be increasing
  v->_capacity = quantity;
  v->_buf = realloc(v->_buf, v->_capacity * v->_unit_length);
}

void vector_push_back(vector * const v, const void * const elem) {
  if (v->length == v->_capacity)
    vector_reserve(v, v->_capacity * VECTOR_FACTOR);

  void *addr =  &((char*)(v->_buf))[v->length++ * v->_unit_length];
  memcpy(addr, elem, v->_unit_length);
}

char *vector_at(vector *const v, const size_t i) {
  assert(i < v->length);
  return (char*)(v->_buf) + (i * v->_unit_length);
}

void vector_test() {
  size_t length = 25;
  char arr[length];
  for (char i = 0; i < length; i++)
    arr[i] = i;

  printf("\t- Testing vectors\n");
  vector v = vector_init(sizeof(char));

  for (int i = 0; i < length; i++)
    vector_push_back(&v, &arr[i]);

  assert(v.length == length);
  for (int i = 0; i < length; i++)
      assert(*vector_at(&v, i) == i);

  vector_destroy(&v);
}

/***************/
/* ENTRY-POINT */
/***************/

int main() {
  printf("Running tests\n");
  vector_test();
  printf("Success\n");
  return 0;
}
