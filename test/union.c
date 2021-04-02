#include "test.h"

union G0 {
	char x;
};

int main()
{
	ASSERT(1, ({union {char x;} x; sizeof(x);}));
	ASSERT(8, ({union {int *x; char y;} x; sizeof(x);}));
	ASSERT(1, ({union S {char x;}; union S x; sizeof(x);}));
	ASSERT(8, ({union S {int *x; char y;}; union S x; sizeof(x);}));
	ASSERT(16, ({union S {int *x; char y[9];}; union S x; sizeof(x);}));

	ASSERT(1, ({union G0 x; sizeof(x);}));
	ASSERT(8, ({union G0 {int *x;}; union G0 x; sizeof(x);}));

	// 0x7f5a0c22 = 2136607778
	// 0x22 = 34
	// 0x0c = 12
	// 0x5a = 90
	// 0x7f = 127
	ASSERT(34, ({union {int i; char c[4];} x; x.i=2136607778; x.c[0];}));
	ASSERT(12, ({union {int i; char c[4];} x; x.i=2136607778; x.c[1];}));
	ASSERT(90, ({union {int i; char c[4];} x; x.i=2136607778; x.c[2];}));
	ASSERT(127, ({union {int i; char c[4];} x; x.i=2136607778; x.c[3];}));

	ASSERT(3, ({union { char a; union {char x; int y;} inner;} x; x.a=3; x.a;}));
	ASSERT(5, ({union { char a; union {char x; int y;} inner;} x; x.inner.x=5; x.inner.x;}));
	ASSERT(7, ({union { char a; union {char x; int y;} inner;} x; x.inner.y=7; x.inner.y;}));

	ASSERT(5, ({union {union X{int i;} *xp;} a,*p=&a; union X x; x.i=5; a.xp=&x; a.xp->i;}));
	ASSERT(7, ({union {union X{int i;} *xp;} a,*p=&a; union X x; x.i=5; a.xp=&x; a.xp->i=7; x.i;}));
	ASSERT(11, ({union {union X{int i;} *xp;} a,*p=&a; union X x; x.i=5; a.xp=&x; p->xp->i=11; x.i;}));

	ASSERT(3, ({union {int i; char c;} x,y,z; x.i=3; z=y=x; y.i;}));
	ASSERT(5, ({union {int i; char c;} x,y,z; x.c=5; z=y=x; y.c;}));
	ASSERT(3, ({union {int i; char c;} x,y,z; x.i=3; z=y=x; z.i;}));
	ASSERT(5, ({union {int i; char c;} x,y,z; x.c=5; z=y=x; z.c;}));

	ASSERT(7, ({union {union inner {int i; char c;} i;} x; union inner y; x.i.i=7; y=x.i; y.i;}));
	ASSERT(11, ({union {union inner {int i; char c;} i;} x; union inner y; x.i.c=11; y=x.i; y.c;}));

	return 0;
}
