#include "test.h"

int f0(){if (1) {return 7;} else {return 5;}}
int f1(){int x=13; if (1) {return x-2;} else {return x+4;}}
int f2(int x){return x;}
int factorial(int x){if (x==1) return 1; return x*factorial(x-1);}
int sub(int x, int y){return x-y;}

int fchar0(char a, char b, char c){return a-b-c;}
int fchar1(char a, char b, int c){return c-a*10-b*10;}

int fst(int *p){return p[0];}
int snd(char *p){return p[1];}
int mul_3(int *p){*p=*p*3; return 0;}

int main()
{
	ASSERT(3, ret3());
	ASSERT(75, ({int i; int ret=0; for (i=0; i<ret5(); i=i+1) ret = ret + ret3() * ret5(); ret;}));
	ASSERT(49, power(modulo(27-10, 5*5-5*3)));
	ASSERT(91, add6_weight(2-1, 3-1, 4-1, 5-1, 6-1, 7-1));
	ASSERT(102, add6_weight(5, 1, 3, 8, 6, 4));

	ASSERT(7, f0());
	ASSERT(11, ({int x=f1(); x;}));
	ASSERT(100, f2(100));
	ASSERT(120, factorial(5));
	ASSERT(10, ({int x=sub(2, 5); int y=sub(8, 1); sub(y, x);}));

	ASSERT(1, fchar0(7, 3, 3));
	ASSERT(100, fchar1(10, 20, 400));

	ASSERT(57, ({int v[2]={-9,9}; fst(v) + snd("ABC");}));
	ASSERT(21, ({int x=7; mul_3(&x); x;}));

	return 0;
}
