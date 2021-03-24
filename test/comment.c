#include "test.h"

int main()
{
	ASSERT(3, /**/3);
	ASSERT(5, ({/* 3
	// */ 5;
	}));
	ASSERT(11, ({
	// 3;
	11;
	}));
	ASSERT(7, ({int x=7, *y=&x; 49 / * y;})); // */
	ASSERT(131, ({char *x="// A", *y="/* B"; x[3] + y[3];}));

	return 0;
}
