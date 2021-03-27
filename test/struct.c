#include "test.h"

struct G0 {
	char x;
};

int main()
{
	ASSERT(1, ({struct {char x;} x; sizeof(x);}));
	ASSERT(16, ({struct {int *x; char y;} x; sizeof(x);}));
	ASSERT(1, ({struct S {char x;}; struct S x; sizeof(x);}));
	ASSERT(16, ({struct S {int *x; char y;}; struct S x; sizeof(x);}));

	ASSERT(1, ({struct G0 x; sizeof(x);}));
	ASSERT(8, ({struct G0 {int *x;}; struct G0 x; sizeof(x);}));

	return 0;
}
