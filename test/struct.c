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

	ASSERT(3, ({struct {int x; int y;} x; int three=3; x.x=three; x.y=7; x.x;}));
	ASSERT(7, ({struct {int x; int y;} x; int three=3; x.x=three; x.y=7; x.y;}));
	ASSERT(11, ({struct {int x; int y;} x; x.x=11; x.y=13; int a=x.x; a;}));
	ASSERT(13, ({struct {int x; int y;} x; x.x=11; x.y=13; int a=x.y; a;}));
	ASSERT(17, ({struct {int x;} x, *p=&x; x.x=0; (*p).x=17; x.x;}));
	ASSERT(65, ({struct {char s[3];} x; x.s[0]=65; x.s[1]=66; x.s[2]=0; x.s[0];}));
	ASSERT(66, ({struct {char s[3];} x; x.s[0]=65; x.s[1]=66; x.s[2]=0; x.s[1];}));
	ASSERT(0, ({struct {char s[3];} x; x.s[0]=65; x.s[1]=66; x.s[2]=0; x.s[2];}));

	struct {
		char x;
		int y;
		struct {
			char x;
			int y;
		} inner;
	} x;
	x.x = 3;
	x.y = 5;
	x.inner.x = 7;
	x.inner.y = 11;

	ASSERT(3, x.x);
	ASSERT(5, x.y);
	ASSERT(7, x.inner.x);
	ASSERT(11, x.inner.y);

	ASSERT(5, ({struct {struct X{int i;} *xp;} a,*p=&a; struct X x; x.i=5; a.xp=&x; a.xp->i;}));
	ASSERT(7, ({struct {struct X{int i;} *xp;} a,*p=&a; struct X x; x.i=5; a.xp=&x; a.xp->i=7; x.i;}));
	ASSERT(11, ({struct {struct X{int i;} *xp;} a,*p=&a; struct X x; x.i=5; a.xp=&x; p->xp->i=11; x.i;}));

	ASSERT(3, ({struct {int i; char c[2];} x,y,z; x.i=3; x.c[0]=5; x.c[1]=7; z=y=x; y.i;}));
	ASSERT(5, ({struct {int i; char c[2];} x,y,z; x.i=3; x.c[0]=5; x.c[1]=7; z=y=x; y.c[0];}));
	ASSERT(7, ({struct {int i; char c[2];} x,y,z; x.i=3; x.c[0]=5; x.c[1]=7; z=y=x; y.c[1];}));
	ASSERT(3, ({struct {int i; char c[2];} x,y,z; x.i=3; x.c[0]=5; x.c[1]=7; z=y=x; z.i;}));
	ASSERT(5, ({struct {int i; char c[2];} x,y,z; x.i=3; x.c[0]=5; x.c[1]=7; z=y=x; z.c[0];}));
	ASSERT(7, ({struct {int i; char c[2];} x,y,z; x.i=3; x.c[0]=5; x.c[1]=7; z=y=x; z.c[1];}));

	ASSERT(11, ({struct {struct inner {int i;} i;} x; struct inner y; x.i.i=11; y=x.i; y.i;}));

	return 0;
}
