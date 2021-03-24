#include "test.h"

int g0[2][4];

int main()
{
	ASSERT(8, sizeof 1);
	ASSERT(8, sizeof sizeof 1);
	ASSERT(8, ({int x; sizeof x;}));
	ASSERT(8, ({int x; sizeof&x;}));
	ASSERT(8, ({int x=5; sizeof(5+(x*x)/8);}));
	ASSERT(40, ({int x[5]; sizeof x;}));
	ASSERT(48, ({int x[3][2]; sizeof x;}));
	ASSERT(16, ({int x[4][2]; sizeof x[2];}));
	ASSERT(65, sizeof g0+1);
	ASSERT(2, ({int x=2; sizeof(x=1); x;}));

	ASSERT(1, ({char c; sizeof c;}));
	ASSERT(10, ({char str[10]; sizeof str;}));
	ASSERT(8, ({char *p; sizeof p;}));

	return 0;
}
