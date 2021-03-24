#include "test.h"

int;

int g0;
int f0(){g0=200; return g0;}
int g1;
int f1(){g1=100; return f0()-g1;}

int g2;

int g3, g4;
int g5; int g6;

int g7=-1, g8=2, *g9, g10=3;
int g11={7};

int main()
{
	ASSERT(1, ({int a=1; a;}));
	ASSERT(2, ({int z=2; z;}));
	ASSERT(30, ({int a=1, b=2, c=3; a=b=c; a*10;}));
	ASSERT(123, ({int x = 2 >= 0, y = (x * x) + (x / x), z = -x * (4 - 5) + y; x*100 + y*10 + z;}));
	ASSERT(72, ({int val001_num=3*4*6, val002_div=val001_num/6, val003_power=val002_div*val002_div; val003_power/2;}));

	ASSERT(3, ({int; 3;}));
	ASSERT(2, ({g0=g1=0; g0=f1()-50; g1/g0;}));

	ASSERT(10, ({int *x=&g2; g2=7; *x=g2+3; g2;}));

	ASSERT(21, ({g3=3; g4=7; g3*g4;}));
	ASSERT(18, ({g5=7; g6=11; g5+g6;}));

	ASSERT(15, ({g9 = &g10; *g9*((g7==-1)*g8+g10);}));
	ASSERT(7, g11);

	ASSERT(30, ({char x, y; x=y=29; x+(x==y);}));
	ASSERT(31, ({char x=31; char *p=&x; *p;}));

	return 0;
}
