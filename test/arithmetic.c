#include "test.h"

int main()
{
	ASSERT(0, 0);
	ASSERT(42, 42);
	ASSERT(21, 5+20-4);
	ASSERT(41,  12 +  34 - 5 );
	ASSERT(0, 10-  5 -5);

	ASSERT(63, 9 * 7);
	ASSERT(9, 4+3*2-1);
	ASSERT(2, 20/10);
	ASSERT(4, 4+3/2-1);
	ASSERT(5,  1 + 12 * 4  / 8 - 2 );

	ASSERT(7, (4+3)*(2-1));
	ASSERT(8,  ( 1 + 12) * 4  / (8 - 2) );
	ASSERT(5,  1 + (((12 * 4)  / 8) - 2) );

	ASSERT(10, -10+20);
	ASSERT(60, + 15*4);
	ASSERT(1, - (3 * 2) + 7);
	ASSERT(1, +(4 / 2) - 1);

	ASSERT(1, 0 == 0);
	ASSERT(0, 0 == 1);
	ASSERT(1, 5+3 == 2*4);
	ASSERT(1, 5+3 == 2*4 == 8-7);
	ASSERT(1, 2 == (4/2+3 == 5) + 1);
	ASSERT(1, 13 != 17);
	ASSERT(0, 19 != 19);
	ASSERT(1, 5*2+3 != 7+5*2 == 1);

	ASSERT(1, 0 < 1);
	ASSERT(0, 1 < 1);
	ASSERT(0, 2 < 1);
	ASSERT(0, 0 > 1);
	ASSERT(0, 1 > 1);
	ASSERT(1, 2 > 1);
	ASSERT(1, 0 <= 1);
	ASSERT(1, 1 <= 1);
	ASSERT(0, 2 <= 1);
	ASSERT(0, 0 >= 1);
	ASSERT(1, 1 >= 1);
	ASSERT(1, 2 >= 1);
	ASSERT(0, (1+2*3 < 6-5/4) <= 1 != (10 >= 3) + 10 > 9);

	return 0;
}
