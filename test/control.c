#include "test.h"

int early_return_0(){int n=5; return n; n=3; return n;}
int early_return_1(){return 3; return 100;}

int if_without_else_0(){int a = 1; if (a == 0) return 0; return a + 1;}
int if_without_else_1(){int x = 1; if (x == 1) if (2 == 2) return 1; return 0;}
int if_without_else_2(){int t=20, u=0, v=10; if (t) v = v + 2; if (u) v = 0; return v*2;}

int infinite_loop_0(){for(;;) return 42; return 0;}
int infinite_loop_1(){int a=1; for(a=2; ; a=3) {a=4; return a;} return 0;}

int stmt_expr(){return ({11; return 13; 17;});}

int main()
{
	ASSERT(5, early_return_0());
	ASSERT(3, early_return_1());

	ASSERT(2, if_without_else_0());
	ASSERT(1, if_without_else_1());
	ASSERT(24, if_without_else_2());

	ASSERT(1, ({int x; if (0 == 1) x=0; else x=1; x;}));
	ASSERT(1, ({int x; int a=5; if (a) a = a*2; else a = 0; if (a != 10) x=0; else if (a == 10) x=1; else x=2; x;}));
	ASSERT(8, ({int x; int a,b,c,d,e; a=b=c=d=e=0;if(a==0){b=a+1;if(b==1){c=b+1;if(c==1){x=0;}else{d=c*2; e=d*2;}}x=e;} x;}));

	ASSERT(4, ({int x; int a=1, b; {b = a + 1; {x=b*b;}} x;}));
	ASSERT(0, ({int a=0; {{{{{{{{{{{{{{1;}}}}}}}}}}}}}} a;}));

	ASSERT(1, ({;;;;; 1;}));
	ASSERT(100, ({int a=100; if (a==0); else; a;}));

	ASSERT(55, ({int sum=0, i; for(i=1; i<=10; i=i+1){sum=sum+i;} sum;}));
	ASSERT(42, infinite_loop_0());
	ASSERT(4, infinite_loop_1());

	ASSERT(55, ({int sum=0, i=1; while (i<=10) { sum=sum+i; i=i+1; } sum;}));
	ASSERT(36, ({int a=3, b, c; while(0) a=0; b=c=0; while(a>0){b=1; while(b<=3){c=c+a*b; b=b+1;} a=a-1;} c;}));

	ASSERT(1, ({int x=1; {int x=2;} x;}));
	ASSERT(7, ({int x=7; {int x; {x=5;}} x;}));
	ASSERT(5, ({int x=11; {int x=13;} {x=5;} x;}));
	ASSERT(8, ({int x=1, y=2; {int x=2; y=y+x;} if (x==1) {y=y*2;} y;}));

	ASSERT(10, ({ ({ int x=5; x*2; }); }) );
	ASSERT(13, stmt_expr());
	ASSERT(15, ({ ({ ({ int x=3; x; }) + ({ int x=5; x; }) + ({ 7; }); }); }) );

	return 0;
}
