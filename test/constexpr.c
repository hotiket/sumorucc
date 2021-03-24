#include "test.h"

int g0[(1==1) + (10!=-10) + (2<4) + (1<=5) + (0>-1) + (-2>=-7)];
int g1[1 + (0==1) + (10!=10) + (4<2) + (5<=1) + (-1>0) + (-7>=-2)];

int main()
{
	ASSERT(40, ({int a[1+1][5*(4/(1+2))]; sizeof a[1];}));
	ASSERT(48, sizeof g0);
	ASSERT(8, sizeof g1);

	return 0;
}
