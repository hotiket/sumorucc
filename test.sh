#!/bin/bash

assert() {
	expected="$1"
	input="$2"

	target/debug/rcc "$input" >tmp.s
	cc -o tmp tmp.s
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

assert 0 "return 0;"
assert 42 "return 42;"
assert 21 "return 5+20-4;"
assert 41 "return  12 +  34 - 5 ;"
assert 0 "return 10-  5 -5;"

assert 63 "return 9 * 7;"
assert 9 "return 4+3*2-1;"
assert 2 "return 20/10;"
assert 4 "return 4+3/2-1;"
assert 5 "return  1 + 12 * 4  / 8 - 2 ;"

assert 7 "return (4+3)*(2-1);"
assert 8 "return  ( 1 + 12) * 4  / (8 - 2) ;"
assert 5 "return  1 + (((12 * 4)  / 8) - 2) ;"

assert 10 "return -10+20;"
assert 60 "return + 15*4;"
assert 1 "return - (3 * 2) + 7;"
assert 1 "return +(4 / 2) - 1;"

assert 1 "return 0 == 0;"
assert 0 "return 0 == 1;"
assert 1 "return 5+3 == 2*4;"
assert 1 "return 5+3 == 2*4 == 8-7;"
assert 1 "return 2 == (4/2+3 == 5) + 1;"
assert 1 "return 13 != 17;"
assert 0 "return 19 != 19;"
assert 1 "return 5*2+3 != 7+5*2 == 1;"

assert 1 "return 0 < 1;"
assert 0 "return 1 < 1;"
assert 0 "return 2 < 1;"
assert 0 "return 0 > 1;"
assert 0 "return 1 > 1;"
assert 1 "return 2 > 1;"
assert 1 "return 0 <= 1;"
assert 1 "return 1 <= 1;"
assert 0 "return 2 <= 1;"
assert 0 "return 0 >= 1;"
assert 1 "return 1 >= 1;"
assert 1 "return 2 >= 1;"
assert 0 "return (1+2*3 < 6-5/4) <= 1 != (10 >= 3) + 10 > 9;"

assert 1 "a=1; return a;"
assert 2 "z=2; return z;"
assert 30 "a=1; b=2; c=3; a=b=c; return a*10;"
assert 123 "x = 2 >= 0; y = (x * x) + (x / x); z = -x * (4 - 5) + y; return x*100 + y*10 + z;"

assert 72 "val001_num=3*4*6; val002_div=val001_num/6; val003_power=val002_div*val002_div; return val003_power/2;"

assert 5 "n0 = 5; return n0; n1 = 3; return n1;"
assert 3 "return 3; 100;"

assert 2 "a = 1; if (a == 0) return 0; return a + 1;"
assert 1 "x = 1; if (x == 1) if (2 == 2) return 1; return 0;"
assert 24 "t = 20; u = 0; v = 10; if (t) v = v + 2; if (u) v = 0; return v*2;"

echo OK
