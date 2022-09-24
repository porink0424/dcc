#include <stdio.h>

void print(int x)
{
  printf("%d\n", x);
}

// 4つのint型を連続で確保し、配列のアドレスをpに代入する
void alloc4(int **p, int i1, int i2, int i3, int i4)
{
  int arr[4] = {i1, i2, i3, i4};
  *p = arr;
}