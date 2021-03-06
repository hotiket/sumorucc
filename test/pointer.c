#include "test.h"

int *g0;

int g1[3][4];
int init_g1(){int i,j; for(i=0;i<3;i=i+1)for(j=0;j<4;j=j+1)g1[i][j]=i*10+j; return 0;}

int g2[2]={7, 5, };
int g3[3]={2,};
int g4[4][3][2]={{{1},}, {{2,3},}, {{4,5}, {6},}};
int g5[4][3][2]={{1}, {2,3}, {4,5,6}};
int g6[4][3][2]={1,0,0,0,0,0, 2,3,0,0,0,0, 4,5,6};

char g7=-1, g8[2][2]={3,2,-1};

int main()
{
	ASSERT(5, ({int x=5; *&*&x;}));
	ASSERT(10, ({int x=0, y=0; *(&y+1) = 10; x;}));
	ASSERT(20, ({int x=0, y=0; *(&x-2+1) = 20; y;}));
	ASSERT(2, ({int x=0, y=0, z=0; &x-&z;}));
	ASSERT(4, ({int x=1, *y=&x; *y=4; x;}));
	ASSERT(3, ({int x=2, *y=&x; *y+1;}));
	ASSERT(21, ({int x, *y=&x, **z=&y; **z=21; x;}));
	ASSERT(7, ({int; 7;}));

	ASSERT(12, ({int a=5; int x[3]; int b=7; x[0]=1; x[1]=2; x[2]=3; int i=0; x[i]*10 + (a==5) + (b==7);}));
	ASSERT(22, ({int a=5; int x[3]; int b=7; x[0]=1; x[1]=2; x[2]=3; int i=1; x[i]*10 + (a==5) + (b==7);}));
	ASSERT(32, ({int a=5; int x[3]; int b=7; x[0]=1; x[1]=2; x[2]=3; int i=2; x[i]*10 + (a==5) + (b==7);}));
	ASSERT(3, ({int x[2]; 0[x]=5; 1[x]=3; (x[1-1]-(5-4)[x]-1)[x];}));

	ASSERT(12, ({int a=5; int x[2][2]; int b=7; x[0][0]=1; x[0][1]=2; x[1][0]=3; x[1][1]=4; x[0][0]*10 + (a==5) + (b==7);}));
	ASSERT(22, ({int a=5; int x[2][2]; int b=7; x[0][0]=1; x[0][1]=2; x[1][0]=3; x[1][1]=4; x[0][1]*10 + (a==5) + (b==7);}));
	ASSERT(32, ({int a=5; int x[2][2]; int b=7; x[0][0]=1; x[0][1]=2; x[1][0]=3; x[1][1]=4; x[1][0]*10 + (a==5) + (b==7);}));
	ASSERT(42, ({int a=5; int x[2][2]; int b=7; x[0][0]=1; x[0][1]=2; x[1][0]=3; x[1][1]=4; x[1][1]*10 + (a==5) + (b==7);}));
	ASSERT(1, ({int x[2][2], *p=&x[0][0]; x[0][0]=1; x[0][1]=2; x[1][0]=3; x[1][1]=4; p[0];}));
	ASSERT(2, ({int x[2][2], *p=&x[0][0]; x[0][0]=1; x[0][1]=2; x[1][0]=3; x[1][1]=4; p[1];}));
	ASSERT(3, ({int x[2][2], *p=&x[0][0]; x[0][0]=1; x[0][1]=2; x[1][0]=3; x[1][1]=4; p[2];}));
	ASSERT(4, ({int x[2][2], *p=&x[0][0]; x[0][0]=1; x[0][1]=2; x[1][0]=3; x[1][1]=4; p[3];}));
	ASSERT(11, ({int x[2][3][4]; x[1][2][3] = 11; int a=2; (a+1)[a[(x[1][2][3]-10)[x]]];}));

	ASSERT(3, ({int x=100; g0=&x; *g0=3; x;}));

	init_g1();

	ASSERT(0, g1[0][0]);
	ASSERT(13, g1[1][3]);
	ASSERT(22, g1[2][2]);

	ASSERT(7, ({int x={7}; x;}));
	ASSERT(2, ({int x[2]={7, 5, }; (x[0]==7)+(x[1]==5);}));
	ASSERT(3, ({int x[3]={7, 5, 3}; (x[0]==7)+(x[1]==5)+(x[2]==3);}));
	ASSERT(3, ({int x[3]={2}; (x[0]==2)+(x[1]==0)+(x[2]==0);}));
	ASSERT(3, ({int x[3]={2,}; (x[0]==2)+(x[1]==0)+(x[2]==0);}));
	ASSERT(9, ({int x[3][3]={{1,2,3},{4,5,},}; (x[0][0]==1)+(x[0][1]==2)+(x[0][2]==3)+(x[1][0]==4)+(x[1][1]==5)+(x[1][2]==0)+(x[2][0]==0)+(x[2][1]==0)+(x[2][2]==0);}));
	ASSERT(9, ({int x[3][3]={1,2,3,4,5}; (x[0][0]==1)+(x[0][1]==2)+(x[0][2]==3)+(x[1][0]==4)+(x[1][1]==5)+(x[1][2]==0)+(x[2][0]==0)+(x[2][1]==0)+(x[2][2]==0);}));
	ASSERT(9, ({int x[4][3][2]={{{1},}, {{2,3},}, {{4,5}, {6},}}; (x[0][0][0]==1)+(x[0][2][1]==0)+(x[1][0][0]==2)+(x[1][0][1]==3)+(x[2][0][0]==4)+(x[2][0][1]==5)+(x[2][1][0]==6)+(x[2][2][1]==0)+(x[3][2][1]==0);}));
	ASSERT(9, ({int x[4][3][2]={{1}, {2,3}, {4,5,6}}; (x[0][0][0]==1)+(x[0][2][1]==0)+(x[1][0][0]==2)+(x[1][0][1]==3)+(x[2][0][0]==4)+(x[2][0][1]==5)+(x[2][1][0]==6)+(x[2][2][1]==0)+(x[3][2][1]==0);}));
	ASSERT(9, ({int x[4][3][2]={1,0,0,0,0,0, 2,3,0,0,0,0, 4,5,6}; (x[0][0][0]==1)+(x[0][2][1]==0)+(x[1][0][0]==2)+(x[1][0][1]==3)+(x[2][0][0]==4)+(x[2][0][1]==5)+(x[2][1][0]==6)+(x[2][2][1]==0)+(x[3][2][1]==0);}));

	ASSERT(2, (g2[0]==7)+(g2[1]==5));
	ASSERT(3, (g3[0]==2)+(g3[1]==0)+(g3[2]==0));
	ASSERT(9, (g4[0][0][0]==1)+(g4[0][2][1]==0)+(g4[1][0][0]==2)+(g4[1][0][1]==3)+(g4[2][0][0]==4)+(g4[2][0][1]==5)+(g4[2][1][0]==6)+(g4[2][2][1]==0)+(g4[3][2][1]==0));
	ASSERT(9, (g5[0][0][0]==1)+(g5[0][2][1]==0)+(g5[1][0][0]==2)+(g5[1][0][1]==3)+(g5[2][0][0]==4)+(g5[2][0][1]==5)+(g5[2][1][0]==6)+(g5[2][2][1]==0)+(g5[3][2][1]==0));
	ASSERT(9, (g6[0][0][0]==1)+(g6[0][2][1]==0)+(g6[1][0][0]==2)+(g6[1][0][1]==3)+(g6[2][0][0]==4)+(g6[2][0][1]==5)+(g6[2][1][0]==6)+(g6[2][2][1]==0)+(g6[3][2][1]==0));

	ASSERT(4, ({char a=-1, v[2][2]={3,2,-1}; (v[0][0]==3)+(v[0][1]==2)+(v[1][0]==a)+(v[1][1]==0);}));
	ASSERT(4, (g8[0][0]==3)+(g8[0][1]==2)+(g8[1][0]==g7)+(g8[1][1]==0));
	ASSERT(7, ({char x=1; int v[2]={3,7}; v[x];}));

	ASSERT(0, ({struct {struct {int y[2];} x[2];} v[2][2]; int i,j,k,l; for(i=0;i<2;i=i+1) for(j=0;j<2;j=j+1) for(k=0;k<2;k=k+1) for(l=0;l<2;l=l+1) v[i][j].x[k].y[l]=i*1000+j*100+k*10+l; v[0][0].x[0].y[0];}));
	ASSERT(101, ({struct {struct {int y[2];} x[2];} v[2][2]; int i,j,k,l; for(i=0;i<2;i=i+1) for(j=0;j<2;j=j+1) for(k=0;k<2;k=k+1) for(l=0;l<2;l=l+1) v[i][j].x[k].y[l]=i*1000+j*100+k*10+l; v[0][1].x[0].y[1];}));
	ASSERT(1000, ({struct {struct {int y[2];} x[2];} v[2][2]; int i,j,k,l; for(i=0;i<2;i=i+1) for(j=0;j<2;j=j+1) for(k=0;k<2;k=k+1) for(l=0;l<2;l=l+1) v[i][j].x[k].y[l]=i*1000+j*100+k*10+l; v[1][0].x[0].y[0];}));
	ASSERT(1111, ({struct {struct {int y[2];} x[2];} v[2][2]; int i,j,k,l; for(i=0;i<2;i=i+1) for(j=0;j<2;j=j+1) for(k=0;k<2;k=k+1) for(l=0;l<2;l=l+1) v[i][j].x[k].y[l]=i*1000+j*100+k*10+l; v[1][1].x[1].y[1];}));

	ASSERT(7, ({struct {struct X{int i;} *p[2];} a[2]; struct X x,y; x.i=3; y.i=5; a[0].p[0]=&x; a[0].p[1]=&y; a[0].p[0]->i=7; x.i;}));
	ASSERT(11, ({struct {struct X{int i;} *p[2];} a[2]; struct X x,y; x.i=3; y.i=5; a[0].p[0]=&x; a[0].p[1]=&y; a[0].p[1]->i=11; y.i;}));
	ASSERT(13, ({struct {struct X{int i;} *p[2];} a[2]; struct X x,y; x.i=3; y.i=5; (a+1)->p[0]=&x; a[0].p[1]=&y; a[1].p[0]->i=13; x.i;}));
	ASSERT(17, ({struct {struct X{int i;} *p[2];} a[2]; struct X x,y; x.i=3; y.i=5; a[0].p[0]=&x; a->p[1]=&y; a[0].p[1]->i=17; y.i;}));

	return 0;
}
