#!/bin/bash

TEST_FN_FILE="tmp_fn.o"

cat <<EOF | cc -xc -c -o "$TEST_FN_FILE" -
int ret3(){return 3;}
int ret5(){return 5;}
int power(int x){return x*x;}
int modulo(int x, int n){return x%n;}
int add6_weight(int x1, int x2, int x3, int x4, int x5, int x6){return x1*1+x2*2+x3*3+x4*4+x5*5+x6*6;}
EOF

assert() {
	expected="$1"
	input="$2"

	target/debug/rcc "$input" >tmp.s
	cc -o tmp tmp.s "$TEST_FN_FILE"
	./tmp
	actual="$?"

	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but got $actual"
		exit 1
	fi
}

cargo build

assert 0 "int main(){return 0;}"
assert 42 "int main(){return 42;}"
assert 21 "int main(){return 5+20-4;}"
assert 41 "int main(){return  12 +  34 - 5 ;}"
assert 0 "int main(){return 10-  5 -5;}"

assert 63 "int main(){return 9 * 7;}"
assert 9 "int main(){return 4+3*2-1;}"
assert 2 "int main(){return 20/10;}"
assert 4 "int main(){return 4+3/2-1;}"
assert 5 "int main(){return  1 + 12 * 4  / 8 - 2 ;}"

assert 7 "int main(){return (4+3)*(2-1);}"
assert 8 "int main(){return  ( 1 + 12) * 4  / (8 - 2) ;}"
assert 5 "int main(){return  1 + (((12 * 4)  / 8) - 2) ;}"

assert 10 "int main(){return -10+20;}"
assert 60 "int main(){return + 15*4;}"
assert 1 "int main(){return - (3 * 2) + 7;}"
assert 1 "int main(){return +(4 / 2) - 1;}"

assert 1 "int main(){return 0 == 0;}"
assert 0 "int main(){return 0 == 1;}"
assert 1 "int main(){return 5+3 == 2*4;}"
assert 1 "int main(){return 5+3 == 2*4 == 8-7;}"
assert 1 "int main(){return 2 == (4/2+3 == 5) + 1;}"
assert 1 "int main(){return 13 != 17;}"
assert 0 "int main(){return 19 != 19;}"
assert 1 "int main(){return 5*2+3 != 7+5*2 == 1;}"

assert 1 "int main(){return 0 < 1;}"
assert 0 "int main(){return 1 < 1;}"
assert 0 "int main(){return 2 < 1;}"
assert 0 "int main(){return 0 > 1;}"
assert 0 "int main(){return 1 > 1;}"
assert 1 "int main(){return 2 > 1;}"
assert 1 "int main(){return 0 <= 1;}"
assert 1 "int main(){return 1 <= 1;}"
assert 0 "int main(){return 2 <= 1;}"
assert 0 "int main(){return 0 >= 1;}"
assert 1 "int main(){return 1 >= 1;}"
assert 1 "int main(){return 2 >= 1;}"
assert 0 "int main(){return (1+2*3 < 6-5/4) <= 1 != (10 >= 3) + 10 > 9;}"

assert 1 "int main(){int a=1; return a;}"
assert 2 "int main(){int z=2; return z;}"
assert 30 "int main(){int a=1, b=2, c=3; a=b=c; return a*10;}"
assert 123 "int main(){int x = 2 >= 0, y = (x * x) + (x / x), z = -x * (4 - 5) + y; return x*100 + y*10 + z;}"

assert 72 "int main(){int val001_num=3*4*6, val002_div=val001_num/6, val003_power=val002_div*val002_div; return val003_power/2;}"

assert 5 "int main(){int n=5; return n; n=3; return n;}"
assert 3 "int main(){return 3; 100;}"

assert 2 "int main(){int a = 1; if (a == 0) return 0; return a + 1;}"
assert 1 "int main(){int x = 1; if (x == 1) if (2 == 2) return 1; return 0;}"
assert 24 "int main(){int t=20, u=0, v=10; if (t) v = v + 2; if (u) v = 0; return v*2;}"
assert 1 "int main(){if (0 == 1) return 0; else return 1; return 2;}"
assert 1 "int main(){int a=5; if (a) a = a*2; else a = 0; if (a != 10) return 0; else if (a == 10) return 1; else return 2;}"
assert 8 "int main(){int a, b, c, d, e; a=b=c=d=e=0;if(a==0){b=a+1;if(b==1){c=b+1;if(c==1){return 0;}else{d=c*2; e=d*2;}}return e;}}"

assert 4 "int main(){int a=1, b; {b = a + 1; {return b * b;}} {return a;}}"
assert 0 "int main(){int a=0; {{{{{{{{{{{{{{1;}}}}}}}}}}}}}} return a;}"

assert 1 "int main(){;;;;; return 1;}"
assert 100 "int main(){int a=100; if (a==0); else; return a;}"

assert 55 "int main(){int sum=0, i; for(i=1; i<=10; i=i+1){sum=sum+i;} return sum;}"
assert 42 "int main(){for(;;) return 42; return 0;}"
assert 4 "int main(){int a=1; for(a=2; ; a=3){a=4; return a;} return 0;}"

assert 55 "int main(){int sum=0, i=1; while (i<=10) { sum=sum+i; i=i+1; } return sum;}"
assert 36 "int main(){int a=3, b, c; while(0) a=0; b=c=0; while(a>0){b=1; while(b<=3){c=c+a*b; b=b+1;} a=a-1;} return c;}"

assert 5 "int main(){int x=5; return *&*&x;}"
assert 10 "int main(){int x=0, y=0; *(&y+1) = 10; return x;}"
assert 20 "int main(){int x=0, y=0; *(&x-2+1) = 20; return y;}"
assert 2 "int main(){int x=0, y=0, z=0; return &x-&z;}"
assert 4 "int main(){int x=1, *y=&x; *y=4; return x;}"
assert 3 "int main(){int x=2, *y=&x; return *y+1;}"
assert 21 "int main(){int x, *y=&x, **z=&y; **z=21; return x;}"
assert 7 "int main(){int; return 7;}"

assert 3 "int main(){return ret3();}"
assert 75 "int main(){int i; int ret=0; for (i=0; i<ret5(); i=i+1) ret = ret + ret3() * ret5(); return ret;}"
assert 49 "int main(){return power(modulo(27-10, 5*5-5*3));}"
assert 91 "int main(){return add6_weight(2-1, 3-1, 4-1, 5-1, 6-1, 7-1);}"
assert 102 "int main(){return add6_weight(5, 1, 3, 8, 6, 4);}"

assert 7 "int f(){if (1) {return 7;} else {return 5;}} int main(){if (0) {return 3;} else {return f();}}"
assert 11 "int f(){int x=13; if (1) {return x-2;} else {return x+4;}} int main(){int x=f(); if (0) {return f();} else {return x;}}"
assert 100 "int f(int x){return x;} int main(){return f(100);}"
assert 120 "int factorial(int x){if (x==1) return 1; return x*factorial(x-1);} int main(){return factorial(5);}"
assert 10 "int sub(int x, int y){return x-y;} int main(){int x=sub(2, 5); int y=sub(8, 1); return sub(y, x);}"

echo OK
